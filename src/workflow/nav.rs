//! Navigation and editor integration workflows (`git claw cd` and `git claw open`).

use crate::core::config::Config;
use crate::infra::git::{resolve_git_common_dir, resolve_primary_repo_root, resolve_repo_name};
use crate::infra::registry::SlotRegistry;
use crate::workflow::error::WorkflowError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Options for navigating to a worktree or primary repository root.
///
/// :param name: Target branch, slot name, or root alias ("main", "master", "root").
/// :param repo_root: Optional repository root path.
///
/// # Example
///
/// ```
/// use git_claw::workflow::CdOptions;
///
/// let options = CdOptions {
///     name: Some("main"),
///     repo_root: None,
/// };
/// assert_eq!(options.name, Some("main"));
/// ```
pub struct CdOptions<'a> {
    pub name: Option<&'a str>,
    pub repo_root: Option<&'a Path>,
}

/// Options for opening a worktree in an editor.
///
/// :param name: Target worktree name or alias.
/// :param editor: Optional editor binary or command name.
/// :param repo_root: Optional repository root path.
///
/// # Example
///
/// ```
/// use git_claw::workflow::OpenOptions;
///
/// let options = OpenOptions {
///     name: Some("feature-x"),
///     editor: Some("nano"),
///     repo_root: None,
/// };
/// assert_eq!(options.editor, Some("nano"));
/// ```
pub struct OpenOptions<'a> {
    pub name: Option<&'a str>,
    pub editor: Option<&'a str>,
    pub repo_root: Option<&'a Path>,
}

/// Resolves the worktree path from name or current working directory.
///
/// If `name` is `"main"`, `"master"`, `"root"`, or matches the configured main branch,
/// returns the primary repository root directory.
/// If `name` is omitted and the shell is outside any active worktree, falls back to the
/// primary repository root directory.
///
/// :param name: Optional target worktree name or primary alias.
/// :param repo_root: Optional repository root path override.
/// :return: Resolved [`PathBuf`] on success, or [`WorkflowError`] on failure.
fn resolve_target_worktree_path(
    name: Option<&str>,
    repo_root: Option<&Path>,
) -> Result<PathBuf, WorkflowError> {
    let toplevel = resolve_primary_repo_root(repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;

    let config_path = toplevel.join(".git-claw.toml");
    let repo_name = resolve_repo_name(&toplevel);
    let config = Config::load_or_default(&config_path, &repo_name)?;

    let registry_path = common_dir.join("claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path)?;

    match name {
        Some(target_name) => {
            if target_name == "main"
                || target_name == "master"
                || target_name == "root"
                || target_name == config.main_branch()
            {
                return Ok(toplevel);
            }
            let slot = registry
                .find_by_name(target_name)
                .ok_or_else(|| WorkflowError::WorktreeNotFound(target_name.to_string()))?;
            Ok(PathBuf::from(&slot.path))
        }
        None => {
            let cwd = repo_root
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
            let found = registry.slots.iter().find(|s| {
                let p = PathBuf::from(&s.path);
                let canonical_p = p.canonicalize().unwrap_or(p);
                canonical_cwd.starts_with(&canonical_p)
            });

            match found {
                Some(s) => Ok(PathBuf::from(&s.path)),
                None => Ok(toplevel),
            }
        }
    }
}

/// Resolves and prints the raw worktree path to stdout for shell integration (`cd $(git claw cd <name>)`).
///
/// :param options: CD navigation options containing target name and optional repo root.
/// :return: Resolved [`PathBuf`] on success, or [`WorkflowError`] on failure.
///
/// # Example
///
/// ```no_run
/// use git_claw::workflow::{cd_worktree, CdOptions};
///
/// let path = cd_worktree(CdOptions {
///     name: Some("main"),
///     repo_root: None,
/// });
/// ```
pub fn cd_worktree(options: CdOptions<'_>) -> Result<PathBuf, WorkflowError> {
    let path = resolve_target_worktree_path(options.name, options.repo_root)?;
    println!("{}", path.display());
    Ok(path)
}

/// Launches the configured editor targeting the worktree directory.
///
/// :param options: Editor launch options.
/// :return: Unit result on success, or [`WorkflowError`] on failure.
///
/// # Example
///
/// ```no_run
/// use git_claw::workflow::{open_worktree, OpenOptions};
///
/// let res = open_worktree(OpenOptions {
///     name: Some("main"),
///     editor: Some("true"),
///     repo_root: None,
/// });
/// ```
pub fn open_worktree(options: OpenOptions<'_>) -> Result<(), WorkflowError> {
    let path = resolve_target_worktree_path(options.name, options.repo_root)?;

    let editor_cmd = match options.editor {
        Some(e) => e.to_string(),
        None => std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| "code".to_string()),
    };

    let status = Command::new(&editor_cmd).arg(&path).status().map_err(|e| {
        WorkflowError::CommandExecutionFailed(format!(
            "failed to launch editor '{}': {}",
            editor_cmd, e
        ))
    })?;

    if !status.success() {
        return Err(WorkflowError::CommandExecutionFailed(format!(
            "editor '{}' exited with non-zero status",
            editor_cmd
        )));
    }

    Ok(())
}
