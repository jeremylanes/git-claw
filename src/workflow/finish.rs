//! Finish worktree lifecycle orchestration.

use crate::cli::output::print_success;
use crate::core::config::Config;
use crate::infra::git::{
    git_checkout, git_is_clean, git_merge, resolve_git_common_dir, resolve_repo_name,
    resolve_toplevel,
};
use crate::infra::hook::{run_post_finish_hook, run_pre_finish_hook};
use crate::infra::lock::SlotLock;
use crate::infra::registry::SlotRegistry;
use crate::infra::worktree::{delete_branch, remove_worktree};
use crate::workflow::error::WorkflowError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct FinishOptions<'a> {
    pub name: Option<&'a str>,
    pub force: bool,
    pub repo_root: Option<&'a Path>,
}

/// Orchestrates finishing and merging a worktree following the AD-8 teardown order.
pub fn finish_worktree(options: FinishOptions<'_>) -> Result<(), WorkflowError> {
    let toplevel = resolve_toplevel(options.repo_root)?;
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
            let cwd = std::env::current_dir().unwrap_or_default();
            let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
            let found = registry.slots.iter().find(|s| {
                let p = PathBuf::from(&s.path);
                p.canonicalize().unwrap_or(p) == canonical_cwd
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
    run_pre_finish_hook(
        config.hooks.pre_finish.as_deref(),
        &worktree_path,
        &hook_env,
        options.force,
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

    // 6. Release Slot ID in registry
    registry.remove_by_id(slot.id);
    registry.save_atomic(&registry_path)?;

    // Release lock before running post-finish hook
    drop(_lock);

    // 7. Run post_finish hook
    let _ = run_post_finish_hook(config.hooks.post_finish.as_deref(), &toplevel, &hook_env);

    print_success(format!(
        "Worktree '{}' (Slot {}) successfully merged into '{}' and cleaned up",
        slot.name,
        slot.id,
        config.main_branch()
    ));

    Ok(())
}
