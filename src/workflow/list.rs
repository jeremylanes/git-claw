//! Active worktree slots listing and dashboard rendering.

use crate::cli::output::{
    print_json, print_worktree_table, ListSummary, WorktreeListItem, WorktreeRow,
};
use crate::core::config::Config;
use crate::core::port::{calculate_effective_ports, filter_shiftable_ports};
use crate::infra::git::{
    git_head_commit, resolve_git_common_dir, resolve_primary_repo_root, resolve_repo_name,
};
use crate::infra::lock::SlotLock;
use crate::infra::registry::SlotRegistry;
use crate::workflow::error::WorkflowError;
use std::path::Path;

fn collect_worktree_entries(
    repo_root: Option<&Path>,
) -> Result<Vec<(WorktreeListItem, String)>, WorkflowError> {
    let toplevel = resolve_primary_repo_root(repo_root)?;
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

    let shiftable_ports = filter_shiftable_ports(&config.ports, &config.docker.shared_services);
    let mut entries = Vec::new();
    for slot in &registry.slots {
        let ports_map = calculate_effective_ports(&shiftable_ports, slot.id)?;
        let commit_str = git_head_commit(Path::new(&slot.path)).unwrap_or_else(|_| "-".to_string());
        let status = if Path::new(&slot.path).exists() {
            "active".to_string()
        } else {
            "missing".to_string()
        };

        entries.push((
            WorktreeListItem {
                slot_id: slot.id,
                name: slot.name.clone(),
                branch: slot.branch.clone(),
                path: slot.path.clone(),
                ports: ports_map,
                status,
            },
            commit_str,
        ));
    }

    drop(_lock);
    Ok(entries)
}

/// Inspects active worktree slots, runs auto-GC, and returns structured slot metadata.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::workflow::list::list_worktrees_summary;
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// let summary = list_worktrees_summary(Some(Path::new(".")))?;
/// assert!(summary.worktrees.is_empty() || summary.worktrees[0].slot_id >= 1);
/// # Ok(())
/// # }
/// ```
pub fn list_worktrees_summary(repo_root: Option<&Path>) -> Result<ListSummary, WorkflowError> {
    let entries = collect_worktree_entries(repo_root)?;
    let worktrees = entries.into_iter().map(|(item, _)| item).collect();
    Ok(ListSummary { worktrees })
}

/// Inspects active worktree slots, runs auto-GC, and emits either JSON or the dashboard table.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::workflow::list::list_worktrees_with_output;
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// list_worktrees_with_output(Some(Path::new(".")), true)?;
/// # Ok(())
/// # }
/// ```
pub fn list_worktrees_with_output(
    repo_root: Option<&Path>,
    json: bool,
) -> Result<(), WorkflowError> {
    let entries = collect_worktree_entries(repo_root)?;

    if json {
        let summary = ListSummary {
            worktrees: entries.into_iter().map(|(item, _)| item).collect(),
        };
        print_json(&summary).map_err(|e| WorkflowError::ExecutionFailed(e.to_string()))?;
        return Ok(());
    }

    let rows: Vec<WorktreeRow> = entries
        .into_iter()
        .map(|(item, commit)| {
            let ports_str = if item.ports.is_empty() {
                "-".to_string()
            } else {
                item.ports
                    .into_iter()
                    .map(|(name, port)| format!("{}:{}", name, port))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            WorktreeRow {
                slot: item.slot_id,
                branch: item.branch,
                path: item.path,
                ports: ports_str,
                commit,
            }
        })
        .collect();

    print_worktree_table(&rows);
    Ok(())
}

/// Inspects active worktree slots, runs auto-GC, and renders the status dashboard table.
pub fn list_worktrees(repo_root: Option<&Path>) -> Result<(), WorkflowError> {
    list_worktrees_with_output(repo_root, false)
}
