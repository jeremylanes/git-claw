//! Navigation and editor integration workflows (`git claw cd` and `git claw open`).

use crate::infra::git::{resolve_git_common_dir, resolve_toplevel};
use crate::infra::registry::SlotRegistry;
use crate::workflow::error::WorkflowError;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CdOptions<'a> {
    pub name: Option<&'a str>,
    pub repo_root: Option<&'a Path>,
}

pub struct OpenOptions<'a> {
    pub name: Option<&'a str>,
    pub editor: Option<&'a str>,
    pub repo_root: Option<&'a Path>,
}

/// Resolves the worktree path from name or current working directory.
fn resolve_target_worktree_path(
    name: Option<&str>,
    repo_root: Option<&Path>,
) -> Result<PathBuf, WorkflowError> {
    let toplevel = resolve_toplevel(repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;

    let registry_path = common_dir.join("claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path)?;

    match name {
        Some(target_name) => {
            let slot = registry
                .find_by_name(target_name)
                .ok_or_else(|| WorkflowError::WorktreeNotFound(target_name.to_string()))?;
            Ok(PathBuf::from(&slot.path))
        }
        None => {
            let cwd = std::env::current_dir().unwrap_or_default();
            let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
            let found = registry.slots.iter().find(|s| {
                let p = PathBuf::from(&s.path);
                let canonical_p = p.canonicalize().unwrap_or(p);
                canonical_cwd.starts_with(&canonical_p)
            });

            match found {
                Some(s) => Ok(PathBuf::from(&s.path)),
                None => Err(WorkflowError::CannotInferWorktree(
                    canonical_cwd.to_string_lossy().to_string(),
                )),
            }
        }
    }
}

/// Resolves and prints the raw worktree path to stdout for shell integration (`cd $(git claw cd <name>)`).
pub fn cd_worktree(options: CdOptions<'_>) -> Result<PathBuf, WorkflowError> {
    let path = resolve_target_worktree_path(options.name, options.repo_root)?;
    println!("{}", path.display());
    Ok(path)
}

/// Launches the configured editor targeting the worktree directory.
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
