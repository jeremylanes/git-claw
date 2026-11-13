//! Start worktree lifecycle orchestration.

use crate::cli::output::{print_success, StartSummary};
use crate::core::branch::BranchType;
use crate::core::config::{Config, EnvironmentConfig};
use crate::core::port::{calculate_effective_ports, filter_shiftable_ports};
use crate::core::slot::allocate_lowest_slot;
use crate::infra::docker::generate_docker_compose_override;
use crate::infra::env_file::{
    copy_and_merge_untracked_files, format_slot_test_db_name, sanitize_compose_project_name,
    write_env_worktree,
};
use crate::infra::fs::link_shared_cache_directories;
use crate::infra::git::{resolve_git_common_dir, resolve_primary_repo_root, resolve_repo_name};
use crate::infra::hook::run_post_start_hook_with_output;
use crate::infra::lock::SlotLock;
use crate::infra::registry::{SlotRecord, SlotRegistry};
use crate::infra::worktree::create_worktree;
use crate::workflow::error::WorkflowError;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct StartOptions<'a> {
    pub branch_type: BranchType,
    pub name: &'a str,
    pub isolated: bool,
    pub repo_root: Option<&'a Path>,
}

/// Orchestrates starting a new worktree for `branch_type` and `name`, returning structured details.
///
/// When `quiet_stdout` is `true`, informational banners are suppressed and hook/Herdr stdout
/// is redirected to stderr so caller stdout stays clean for `--json`.
///
/// # Examples
///
/// ```no_run
/// use git_claw::core::branch::BranchType;
/// use git_claw::workflow::start::{start_worktree_summary, StartOptions};
///
/// # fn run() -> Result<(), git_claw::workflow::WorkflowError> {
/// let summary = start_worktree_summary(
///     StartOptions {
///         branch_type: BranchType::Feature,
///         name: "auth",
///         isolated: false,
///         repo_root: None,
///     },
///     true,
/// )?;
/// assert_eq!(summary.name, "auth");
/// # Ok(())
/// # }
/// ```
pub fn start_worktree_summary(
    options: StartOptions<'_>,
    quiet_stdout: bool,
) -> Result<StartSummary, WorkflowError> {
    let toplevel = resolve_primary_repo_root(options.repo_root)?;
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
    let shiftable_ports = filter_shiftable_ports(&config.ports, &config.docker.shared_services);
    let effective_ports = calculate_effective_ports(&shiftable_ports, slot_id)?;

    // Create the Git worktree with the new branch
    create_worktree(
        &toplevel,
        &worktree_path,
        &branch_name,
        config.main_branch(),
    )?;

    // Propagate runtime environment files
    let _ = propagate_environment_files(&toplevel, &worktree_path, &config.environment);

    // Link shared cache directories (including .venv if configured or if env type is venv/uv)
    let mut effective_cache = config.cache.clone();
    if !options.isolated
        && !effective_cache.directories.iter().any(|d| d == ".venv")
        && (config.environment.r#type.as_deref() == Some("venv")
            || config.environment.r#type.as_deref() == Some("uv"))
        && toplevel.join(".venv").exists()
    {
        effective_cache.directories.push(".venv".to_string());
    }

    let _ = link_shared_cache_directories(
        &toplevel,
        &worktree_path,
        &effective_cache,
        options.isolated,
    );

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
    let _ = generate_docker_compose_override(
        &toplevel,
        &worktree_path,
        &config.docker,
        Some(slot_id),
        Some(&repo_name),
    );

    // Write .env.worktree
    write_env_worktree(
        &worktree_path,
        &repo_name,
        options.name,
        slot_id,
        &effective_ports,
        Some(&config.environment),
    )?;

    let test_database = Some(format_slot_test_db_name(&repo_name, slot_id));

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
    let _ = run_post_start_hook_with_output(
        config.hooks.post_start.as_deref(),
        &worktree_path,
        &hook_env,
        quiet_stdout,
    );

    crate::infra::herdr::open_worktree_in_herdr_quiet(&toplevel, &branch_name, quiet_stdout);

    if !quiet_stdout {
        print_success(format!(
            "Worktree '{}' started successfully (Slot: {}, Branch: {}, Path: {})",
            options.name,
            slot_id,
            branch_name,
            worktree_path.display()
        ));
    }

    Ok(StartSummary {
        slot_id,
        name: options.name.to_string(),
        branch: branch_name,
        path: worktree_path.to_string_lossy().to_string(),
        ports: effective_ports,
        test_database,
    })
}

/// Orchestrates starting a new worktree for `branch_type` and `name`.
pub fn start_worktree(options: StartOptions<'_>) -> Result<u32, WorkflowError> {
    start_worktree_summary(options, false).map(|s| s.slot_id)
}

/// Propagates environment configuration files from the primary repository to the worktree.
///
/// Handles declared files in `environment.copy_files`, standard runtime files (`.python-version`,
/// `.node-version`, `.nvmrc`, `.envrc`), and writes `.python-version` from `environment.name`
/// if absent.
///
/// # Examples
///
/// ```
/// use git_claw::workflow::start::propagate_environment_files;
/// use git_claw::core::config::EnvironmentConfig;
/// use tempfile::tempdir;
/// use std::fs;
///
/// let top = tempdir().unwrap();
/// let wt = tempdir().unwrap();
/// fs::write(top.path().join(".python-version"), "3.12.0\n").unwrap();
///
/// let env = EnvironmentConfig::default();
/// propagate_environment_files(top.path(), wt.path(), &env).unwrap();
/// assert_eq!(fs::read_to_string(wt.path().join(".python-version")).unwrap(), "3.12.0\n");
/// ```
pub fn propagate_environment_files(
    toplevel: &Path,
    worktree_path: &Path,
    env_config: &EnvironmentConfig,
) -> std::io::Result<()> {
    // 1. Copy files declared in environment.copy_files
    for file in &env_config.copy_files {
        let file = file.trim();
        if file.is_empty() {
            continue;
        }
        let src = toplevel.join(file);
        let dst = worktree_path.join(file);
        if src.is_file() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst)?;
        }
    }

    // 2. Check standard version files: .python-version, .node-version, .nvmrc, .envrc
    for file in &[".python-version", ".node-version", ".nvmrc", ".envrc"] {
        let src = toplevel.join(file);
        let dst = worktree_path.join(file);
        if src.is_file() && !dst.exists() {
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src, &dst)?;
        }
    }

    // 3. Fallback: if environment.name is set and .python-version does not exist in worktree, write it
    let py_version_dst = worktree_path.join(".python-version");
    if !py_version_dst.exists() {
        if let Some(ref name) = env_config.name {
            let content = if name.ends_with('\n') {
                name.clone()
            } else {
                format!("{}\n", name)
            };
            fs::write(&py_version_dst, content)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_propagate_environment_files_declared_and_standard() {
        let top = tempdir().unwrap();
        let wt = tempdir().unwrap();

        fs::write(top.path().join(".python-version"), "tune\n").unwrap();
        fs::write(top.path().join(".nvmrc"), "20\n").unwrap();
        fs::write(top.path().join("custom.env"), "FOO=BAR\n").unwrap();

        let env = EnvironmentConfig {
            r#type: Some("pyenv".to_string()),
            name: Some("tune".to_string()),
            copy_files: vec!["custom.env".to_string()],
        };

        propagate_environment_files(top.path(), wt.path(), &env).unwrap();

        assert_eq!(
            fs::read_to_string(wt.path().join(".python-version")).unwrap(),
            "tune\n"
        );
        assert_eq!(
            fs::read_to_string(wt.path().join(".nvmrc")).unwrap(),
            "20\n"
        );
        assert_eq!(
            fs::read_to_string(wt.path().join("custom.env")).unwrap(),
            "FOO=BAR\n"
        );
    }

    #[test]
    fn test_propagate_environment_fallback_name() {
        let top = tempdir().unwrap();
        let wt = tempdir().unwrap();

        let env = EnvironmentConfig {
            r#type: Some("pyenv".to_string()),
            name: Some("my-virtualenv".to_string()),
            copy_files: vec![],
        };

        propagate_environment_files(top.path(), wt.path(), &env).unwrap();

        assert_eq!(
            fs::read_to_string(wt.path().join(".python-version")).unwrap(),
            "my-virtualenv\n"
        );
    }
}
