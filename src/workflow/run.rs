//! Contextual command runner with environment injection.

use crate::infra::git::{resolve_git_common_dir, resolve_toplevel};
use crate::infra::registry::SlotRegistry;
use crate::workflow::error::WorkflowError;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RunOptions {
    pub name: Option<String>,
    pub command: Vec<String>,
    pub repo_root: Option<PathBuf>,
}

/// Parses a `.env.worktree` file into key-value pairs.
pub fn parse_env_file(path: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }
    map
}

/// Executes a command in the worktree directory with .env.worktree injected.
pub fn run_command(options: RunOptions) -> Result<i32, WorkflowError> {
    let toplevel = resolve_toplevel(options.repo_root.as_deref())?;
    let common_dir = resolve_git_common_dir(Some(&toplevel))?;

    let registry_path = common_dir.join("claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path)?;

    // Determine target worktree and final command args
    let (slot, cmd_args) = match options.name {
        Some(name_str) => {
            if let Some(s) = registry.find_by_name(&name_str) {
                // Name matches an existing worktree
                (s.clone(), options.command)
            } else {
                // Name does not match a worktree. Check if cwd is inside a worktree.
                let cwd = std::env::current_dir().unwrap_or_default();
                let canonical_cwd = cwd.canonicalize().unwrap_or(cwd);
                let found = registry.slots.iter().find(|s| {
                    let p = PathBuf::from(&s.path);
                    let canonical_p = p.canonicalize().unwrap_or(p);
                    canonical_cwd.starts_with(&canonical_p)
                });

                if let Some(s) = found {
                    let mut full_cmd = vec![name_str];
                    full_cmd.extend(options.command);
                    (s.clone(), full_cmd)
                } else {
                    return Err(WorkflowError::WorktreeNotFound(name_str));
                }
            }
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
                Some(s) => (s.clone(), options.command),
                None => {
                    return Err(WorkflowError::CannotInferWorktree(
                        canonical_cwd.to_string_lossy().to_string(),
                    ));
                }
            }
        }
    };

    if cmd_args.is_empty() {
        return Err(WorkflowError::NoCommandSpecified);
    }

    let worktree_path = PathBuf::from(&slot.path);
    let env_file_path = worktree_path.join(".env.worktree");
    let env_vars = parse_env_file(&env_file_path);

    let mut cmd = Command::new(&cmd_args[0]);
    cmd.args(&cmd_args[1..]);
    cmd.current_dir(&worktree_path);

    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let status = cmd.status().map_err(|e| {
        WorkflowError::CommandExecutionFailed(format!("failed to execute '{}': {}", cmd_args[0], e))
    })?;

    Ok(status.code().unwrap_or(1))
}
