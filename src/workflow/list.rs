//! Active worktree slots listing and dashboard rendering.

use crate::cli::output::{print_worktree_table, WorktreeRow};
use crate::core::config::Config;
use crate::core::port::calculate_effective_ports;
use crate::infra::git::{
    git_head_commit, resolve_git_common_dir, resolve_repo_name, resolve_toplevel,
};
use crate::infra::lock::SlotLock;
use crate::infra::registry::SlotRegistry;
use crate::workflow::error::WorkflowError;
use std::path::Path;

/// Inspects active worktree slots, runs auto-GC, and renders the status dashboard table.
pub fn list_worktrees(repo_root: Option<&Path>) -> Result<(), WorkflowError> {
    let toplevel = resolve_toplevel(repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    let registry_path = common_dir.join("claw/slots.json");
    let lock_path = common_dir.join("claw/slots.lock");

    let _lock = SlotLock::acquire(&lock_path)?;
    let mut registry = SlotRegistry::load_or_empty(&registry_path)?;

    // Pre-flight auto-GC of stale/orphaned worktree entries
    let _ = registry.auto_gc(&registry_path, Some(&toplevel), true);

    let mut rows = Vec::new();
    for slot in &registry.slots {
        let ports_map = calculate_effective_ports(&config.ports, slot.id)?;
        let ports_str = if ports_map.is_empty() {
            "-".to_string()
        } else {
            ports_map
                .into_iter()
                .map(|(name, port)| format!("{}:{}", name, port))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let commit_str = git_head_commit(Path::new(&slot.path)).unwrap_or_else(|_| "-".to_string());

        rows.push(WorktreeRow {
            slot: slot.id,
            branch: slot.branch.clone(),
            path: slot.path.clone(),
            ports: ports_str,
            commit: commit_str,
        });
    }

    drop(_lock);

    print_worktree_table(&rows);

    Ok(())
}
