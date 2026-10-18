//! Remote worktree publish workflow (`git claw [type] publish [name]` and `git claw publish [name]`).
//!
//! Pushes worktree branches to remote Git repositories with upstream tracking.

use crate::cli::output::print_success;
use crate::core::branch::BranchType;
use crate::core::config::Config;
use crate::infra::git::{
    git_branch_exists, git_get_remotes, git_push_upstream, resolve_git_common_dir,
    resolve_primary_repo_root, resolve_repo_name,
};
use crate::infra::registry::{SlotRecord, SlotRegistry};
use crate::workflow::error::WorkflowError;
use std::path::{Path, PathBuf};

/// Options for the worktree publish workflow.
///
/// # Example
///
/// ```
/// use git_claw::workflow::PublishOptions;
/// use git_claw::core::branch::BranchType;
///
/// let options = PublishOptions {
///     branch_type: Some(BranchType::Feature),
///     name: Some("login"),
///     remote: Some("origin"),
///     repo_root: None,
/// };
/// assert_eq!(options.name, Some("login"));
/// ```
#[derive(Debug, Clone, Default)]
pub struct PublishOptions<'a> {
    pub branch_type: Option<BranchType>,
    pub name: Option<&'a str>,
    pub remote: Option<&'a str>,
    pub repo_root: Option<&'a Path>,
}

/// Resolves the active worktree slot from the current working directory or explicit repo root.
///
/// # Example
///
/// ```
/// use git_claw::infra::registry::SlotRegistry;
/// use git_claw::workflow::publish::resolve_active_worktree_slot;
///
/// let registry = SlotRegistry::empty();
/// let result = resolve_active_worktree_slot(&registry, None);
/// assert!(result.is_err());
/// ```
pub fn resolve_active_worktree_slot<'a>(
    registry: &'a SlotRegistry,
    repo_root: Option<&Path>,
) -> Result<&'a SlotRecord, WorkflowError> {
    let cwd = if let Some(root) = repo_root {
        let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if registry.slots.iter().any(|s| {
            let p = PathBuf::from(&s.path);
            let cp = p.canonicalize().unwrap_or(p);
            canonical.starts_with(&cp)
        }) {
            canonical
        } else {
            std::env::current_dir().unwrap_or_default()
        }
    } else {
        std::env::current_dir().unwrap_or_default()
    };
    let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
    let found = registry.slots.iter().find(|s| {
        let p = PathBuf::from(&s.path);
        let canonical_p = p.canonicalize().unwrap_or(p);
        canonical_cwd.starts_with(&canonical_p)
    });

    found.ok_or_else(|| {
        WorkflowError::CannotInferWorktree(canonical_cwd.to_string_lossy().to_string())
    })
}

/// Orchestrates publishing a worktree branch to a remote Git repository with upstream tracking.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::core::branch::BranchType;
/// use git_claw::workflow::{publish_worktree, PublishOptions};
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// publish_worktree(PublishOptions {
///     branch_type: Some(BranchType::Feature),
///     name: Some("login"),
///     remote: Some("origin"),
///     repo_root: Some(Path::new(".")),
/// })?;
/// # Ok(())
/// # }
/// ```
pub fn publish_worktree(options: PublishOptions<'_>) -> Result<(), WorkflowError> {
    let toplevel = resolve_primary_repo_root(options.repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    let registry_path = common_dir.join("claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path)?;

    let slot = match options.name {
        Some(target_name) => {
            let found = registry.slots.iter().find(|s| {
                if let Some(bt) = options.branch_type {
                    (s.name == target_name
                        || s.branch == target_name
                        || s.branch == format!("{}/{}", bt.as_str(), target_name))
                        && s.branch_type == bt.as_str()
                } else {
                    s.name == target_name || s.branch == target_name
                }
            });
            found.ok_or_else(|| WorkflowError::WorktreeNotFound(target_name.to_string()))?
        }
        None => {
            let slot = resolve_active_worktree_slot(&registry, options.repo_root)?;
            if let Some(bt) = options.branch_type {
                if slot.branch_type != bt.as_str() {
                    let cwd = std::env::current_dir().unwrap_or_default();
                    return Err(WorkflowError::CannotInferWorktree(
                        cwd.to_string_lossy().to_string(),
                    ));
                }
            }
            slot
        }
    };

    let worktree_path = PathBuf::from(&slot.path);
    if !worktree_path.exists() {
        return Err(WorkflowError::WorktreeNotFound(slot.name.clone()));
    }

    if !git_branch_exists(&toplevel, &slot.branch)? {
        return Err(WorkflowError::WorktreeNotFound(slot.name.clone()));
    }

    let remotes = git_get_remotes(&toplevel)?;
    if remotes.is_empty() {
        return Err(WorkflowError::NoRemoteConfigured);
    }

    let remote = options
        .remote
        .map(|r| r.to_string())
        .or_else(|| config.remote().map(|r| r.to_string()))
        .unwrap_or_else(|| "origin".to_string());

    git_push_upstream(&toplevel, &remote, &slot.branch)?;

    print_success(format!(
        "Branch '{}' successfully published to '{}' with upstream tracking",
        slot.branch, remote
    ));

    Ok(())
}
