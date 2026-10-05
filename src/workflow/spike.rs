//! Spike worktree drop and abandonment orchestration.

use crate::cli::output::print_success;
use crate::infra::git::{resolve_git_common_dir, resolve_toplevel};
use crate::infra::lock::SlotLock;
use crate::infra::registry::SlotRegistry;
use crate::infra::worktree::{delete_branch, remove_worktree};
use crate::workflow::error::WorkflowError;
use std::path::{Path, PathBuf};

pub struct DropSpikeOptions<'a> {
    pub name: &'a str,
    pub repo_root: Option<&'a Path>,
}

/// Drops a spike worktree without merging, force-removes the worktree and branch, and frees the slot.
pub fn drop_spike(options: DropSpikeOptions<'_>) -> Result<(), WorkflowError> {
    let toplevel = resolve_toplevel(options.repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;

    let registry_path = common_dir.join("claw/slots.json");
    let lock_path = common_dir.join("claw/slots.lock");

    let _lock = SlotLock::acquire(&lock_path)?;
    let mut registry = SlotRegistry::load_or_empty(&registry_path)?;

    let slot = registry
        .find_by_name(options.name)
        .cloned()
        .ok_or_else(|| WorkflowError::WorktreeNotFound(options.name.to_string()))?;

    let worktree_path = PathBuf::from(&slot.path);

    // Force remove worktree directory
    let _ = remove_worktree(&toplevel, &worktree_path, true);

    // Force delete the branch without merging (-D)
    let _ = delete_branch(&toplevel, &slot.branch, true);

    // Release Slot ID in registry
    registry.remove_by_id(slot.id);
    registry.save_atomic(&registry_path)?;

    print_success(format!(
        "Spike worktree '{}' (Slot {}) and branch '{}' dropped successfully without merging",
        slot.name, slot.id, slot.branch
    ));

    Ok(())
}
