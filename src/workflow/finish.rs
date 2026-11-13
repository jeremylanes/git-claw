//! Finish worktree lifecycle orchestration.

use crate::cli::output::{print_success, print_warning, FinishSummary};
use crate::core::config::Config;
use crate::infra::git::{
    git_checkout, git_delete_remote_branch, git_get_remotes, git_is_clean, git_merge,
    git_remote_branch_exists, resolve_git_common_dir, resolve_primary_repo_root, resolve_repo_name,
};
use crate::infra::hook::{run_post_finish_hook_with_output, run_pre_finish_hook_with_output};
use crate::infra::lock::SlotLock;
use crate::infra::registry::SlotRegistry;
use crate::infra::worktree::{delete_branch, remove_worktree};
use crate::workflow::error::WorkflowError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Options for finishing and merging a worktree.
///
/// # Example
///
/// ```
/// use git_claw::workflow::FinishOptions;
///
/// let options = FinishOptions {
///     name: Some("login"),
///     force: false,
///     keep_remote: false,
///     repo_root: None,
/// };
/// assert_eq!(options.name, Some("login"));
/// assert!(!options.keep_remote);
/// ```
#[derive(Debug, Clone, Default)]
pub struct FinishOptions<'a> {
    pub name: Option<&'a str>,
    pub force: bool,
    pub keep_remote: bool,
    pub repo_root: Option<&'a Path>,
}

/// Orchestrates finishing and merging a worktree, returning structured completion details.
///
/// When `quiet_stdout` is `true`, informational banners are suppressed and hook stdout
/// is redirected to stderr so stdout stays clean for `--json`.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::workflow::finish::{finish_worktree_summary, FinishOptions};
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// let summary = finish_worktree_summary(
///     FinishOptions {
///         name: Some("login"),
///         force: false,
///         keep_remote: false,
///         repo_root: Some(Path::new(".")),
///     },
///     true,
/// )?;
/// assert_eq!(summary.name, "login");
/// # Ok(())
/// # }
/// ```
pub fn finish_worktree_summary(
    options: FinishOptions<'_>,
    quiet_stdout: bool,
) -> Result<FinishSummary, WorkflowError> {
    let toplevel = resolve_primary_repo_root(options.repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    let registry_path = common_dir.join("claw/slots.json");
    let lock_path = common_dir.join("claw/slots.lock");

    let _lock = SlotLock::acquire(&lock_path)?;
    let mut registry = SlotRegistry::load_or_empty(&registry_path)?;

    // Infer worktree name if not provided
    let target_name = match options.name {
        Some(n) => n.to_string(),
        None => {
            let cwd = options
                .repo_root
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
            let found = registry.slots.iter().find(|s| {
                let p = PathBuf::from(&s.path);
                let canonical_p = p.canonicalize().unwrap_or(p);
                canonical_cwd == canonical_p || canonical_cwd.starts_with(&canonical_p)
            });

            match found {
                Some(s) => s.name.clone(),
                None => {
                    return Err(WorkflowError::CannotInferWorktree(
                        canonical_cwd.to_string_lossy().to_string(),
                    ));
                }
            }
        }
    };

    let slot = registry
        .find_by_name(&target_name)
        .cloned()
        .ok_or_else(|| WorkflowError::WorktreeNotFound(target_name.clone()))?;

    let worktree_path = PathBuf::from(&slot.path);

    // Prepare environment variables for hooks
    let mut hook_env = BTreeMap::new();
    hook_env.insert("CLAW_SLOT_ID".to_string(), slot.id.to_string());
    hook_env.insert("CLAW_WORKTREE_NAME".to_string(), slot.name.clone());
    hook_env.insert("CLAW_WORKTREE_PATH".to_string(), slot.path.clone());

    // 1. Run pre_finish hook (abort on failure unless force is true)
    run_pre_finish_hook_with_output(
        config.hooks.pre_finish.as_deref(),
        &worktree_path,
        &hook_env,
        options.force,
        quiet_stdout,
    )?;

    // 2. Verify primary working tree is clean before merging
    if !git_is_clean(&toplevel)? {
        return Err(WorkflowError::DirtyWorkingTree);
    }

    // 3. Checkout main and merge worktree branch
    git_checkout(&toplevel, config.main_branch())?;
    git_merge(&toplevel, &slot.branch)?;

    // 4. Force remove worktree directory (ignoring untracked generated files)
    remove_worktree(&toplevel, &worktree_path, true)?;

    // 5. Delete the merged branch
    delete_branch(&toplevel, &slot.branch, false)?;

    // Teardown remote branch if published and not opted out
    let mut remote_deleted = false;
    if !options.keep_remote {
        let remote = config.remote().unwrap_or("origin");
        let remotes = git_get_remotes(&toplevel).unwrap_or_default();
        if remotes.iter().any(|r| r == remote) {
            match git_remote_branch_exists(&toplevel, remote, &slot.branch) {
                Ok(true) => {
                    if let Err(e) = git_delete_remote_branch(&toplevel, remote, &slot.branch) {
                        print_warning(format!(
                            "Failed to delete remote branch '{}' from '{}': {}",
                            slot.branch, remote, e
                        ));
                    } else {
                        remote_deleted = true;
                        if !quiet_stdout {
                            print_success(format!(
                                "Remote branch '{}' deleted from '{}'",
                                slot.branch, remote
                            ));
                        }
                    }
                }
                Ok(false) => {}
                Err(e) => {
                    print_warning(format!(
                        "Could not check remote branch '{}' on '{}': {}",
                        slot.branch, remote, e
                    ));
                }
            }
        }
    }

    // 6. Release Slot ID in registry
    registry.remove_by_id(slot.id);
    registry.save_atomic(&registry_path)?;

    // Release lock before running post-finish hook
    drop(_lock);

    // 7. Run post_finish hook
    let _ = run_post_finish_hook_with_output(
        config.hooks.post_finish.as_deref(),
        &toplevel,
        &hook_env,
        quiet_stdout,
    );

    // Record target primary root for shell auto-cd integration
    let _ = std::fs::write(
        "/tmp/.git_claw_target_cd",
        toplevel.to_string_lossy().as_bytes(),
    );

    if !quiet_stdout {
        print_success(format!(
            "Worktree '{}' (Slot {}) successfully merged into '{}' and cleaned up",
            slot.name,
            slot.id,
            config.main_branch()
        ));
    }

    Ok(FinishSummary {
        name: slot.name,
        branch: slot.branch,
        target_branch: config.main_branch().to_string(),
        slot_id: Some(slot.id),
        primary_root: toplevel.to_string_lossy().to_string(),
        remote_deleted,
    })
}

/// Orchestrates finishing and merging a worktree following the AD-8 teardown order.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::workflow::{finish_worktree, FinishOptions};
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// finish_worktree(FinishOptions {
///     name: Some("login"),
///     force: false,
///     keep_remote: false,
///     repo_root: Some(Path::new(".")),
/// })?;
/// # Ok(())
/// # }
/// ```
pub fn finish_worktree(options: FinishOptions<'_>) -> Result<(), WorkflowError> {
    finish_worktree_summary(options, false).map(|_| ())
}
