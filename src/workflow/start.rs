//! Start worktree lifecycle orchestration.

use crate::cli::output::print_success;
use crate::core::branch::BranchType;
use crate::core::config::Config;
use crate::core::port::{calculate_effective_ports, is_shared_service_port};
use crate::core::slot::allocate_lowest_slot;
use crate::infra::docker::generate_docker_compose_override;
use crate::infra::env_file::{
    copy_and_merge_untracked_files, sanitize_compose_project_name, write_env_worktree,
};
use crate::infra::fs::link_shared_cache_directories;
use crate::infra::git::{resolve_git_common_dir, resolve_repo_name, resolve_toplevel};
use crate::infra::hook::run_post_start_hook;
use crate::infra::lock::SlotLock;
use crate::infra::registry::{SlotRecord, SlotRegistry};
use crate::infra::worktree::create_worktree;
use crate::workflow::error::WorkflowError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct StartOptions<'a> {
    pub branch_type: BranchType,
    pub name: &'a str,
    pub isolated: bool,
    pub repo_root: Option<&'a Path>,
}

/// Orchestrates starting a new worktree for `branch_type` and `name`.
pub fn start_worktree(options: StartOptions<'_>) -> Result<u32, WorkflowError> {
    let toplevel = resolve_toplevel(options.repo_root)?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;
    let repo_name = resolve_repo_name(&toplevel);

    let config_path = toplevel.join(".git-claw.toml");
    let config = Config::load_or_default(&config_path, &repo_name)?;

    let branch_name = options.branch_type.format_branch_name(options.name)?;

    let raw_worktree_root = config.worktree_root(&repo_name);
    let worktree_root = if Path::new(&raw_worktree_root).is_absolute() {
        PathBuf::from(raw_worktree_root)
    } else {
        toplevel.join(raw_worktree_root)
    };
    let worktree_path = worktree_root.join(options.name);

    let lock_path = common_dir.join("claw/slots.lock");
    let registry_path = common_dir.join("claw/slots.json");

    // Acquire exclusive advisory lock before touching registry or worktree
    let _lock = SlotLock::acquire(&lock_path)?;

    let mut registry = SlotRegistry::load_or_empty(&registry_path)?;

    // Auto-GC stale worktrees
    let _ = registry.auto_gc(&registry_path, Some(&toplevel), true);

    if registry.find_by_name(options.name).is_some() {
        return Err(WorkflowError::DuplicateSlotName(options.name.to_string()));
    }

    let slot_id = allocate_lowest_slot(registry.occupied_ids());

    // Filter out ports associated with shared services so client connections to shared containers are never broken
    let shiftable_ports: BTreeMap<String, u16> = config
        .ports
        .iter()
        .filter(|(k, _)| !is_shared_service_port(k, &config.docker.shared_services))
        .map(|(k, &v)| (k.clone(), v))
        .collect();

    let effective_ports = calculate_effective_ports(&shiftable_ports, slot_id)?;

    // Create the Git worktree with the new branch
    create_worktree(
        &toplevel,
        &worktree_path,
        &branch_name,
        config.main_branch(),
    )?;

    // Write .env.worktree
    write_env_worktree(
        &worktree_path,
        &repo_name,
        options.name,
        slot_id,
        &effective_ports,
    )?;

    let compose_project = sanitize_compose_project_name(&repo_name, options.name, slot_id);

    // Copy declared untracked files and update port variables in-place
    let _ = copy_and_merge_untracked_files(
        &toplevel,
        &worktree_path,
        &config.files.copy,
        &shiftable_ports,
        slot_id,
        Some(&compose_project),
    );

    // Generate docker-compose.claw.override.yml if configured
    let _ = generate_docker_compose_override(&toplevel, &worktree_path, &config.docker);

    // Link shared cache directories
    let _ =
        link_shared_cache_directories(&toplevel, &worktree_path, &config.cache, options.isolated);

    // Record in registry and persist atomically
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    registry.slots.push(SlotRecord {
        id: slot_id,
        name: options.name.to_string(),
        branch_type: options.branch_type.to_string(),
        branch: branch_name.clone(),
        path: worktree_path.to_string_lossy().to_string(),
        allocated_at: now.to_string(),
    });
    registry.save_atomic(&registry_path)?;

    // Drop lock before running hook
    drop(_lock);

    // Prepare hook environment
    let mut hook_env = BTreeMap::new();
    hook_env.insert("CLAW_SLOT_ID".to_string(), slot_id.to_string());
    hook_env.insert("CLAW_WORKTREE_NAME".to_string(), options.name.to_string());
    hook_env.insert(
        "CLAW_WORKTREE_PATH".to_string(),
        worktree_path.to_string_lossy().to_string(),
    );
    for (k, v) in &effective_ports {
        hook_env.insert(k.clone(), v.to_string());
    }

    // Run post_start hook if configured
    let _ = run_post_start_hook(
        config.hooks.post_start.as_deref(),
        &worktree_path,
        &hook_env,
    );

    print_success(format!(
        "Worktree '{}' started successfully (Slot: {}, Branch: {}, Path: {})",
        options.name,
        slot_id,
        branch_name,
        worktree_path.display()
    ));

    Ok(slot_id)
}
