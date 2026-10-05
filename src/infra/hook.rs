//! Lifecycle hook execution and policy enforcement.

use crate::cli::output::print_warning;
use crate::infra::error::HookError;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// Executes a shell command inside `working_dir` with `env_vars` injected.
pub fn execute_hook_command(
    command_str: &str,
    working_dir: &Path,
    env_vars: &BTreeMap<String, String>,
) -> Result<bool, HookError> {
    #[cfg(unix)]
    let mut cmd = Command::new("sh");
    #[cfg(unix)]
    cmd.args(["-c", command_str]);

    #[cfg(windows)]
    let mut cmd = Command::new("cmd");
    #[cfg(windows)]
    cmd.args(["/C", command_str]);

    cmd.current_dir(working_dir);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let status = cmd
        .status()
        .map_err(|e| HookError::ExecutionFailed(format!("Failed to spawn hook command: {}", e)))?;

    Ok(status.success())
}

/// Runs `post_start` hook: non-fatal warning on failure, does not abort worktree creation.
pub fn run_post_start_hook(
    hook: Option<&str>,
    worktree_path: &Path,
    env_vars: &BTreeMap<String, String>,
) -> Result<bool, HookError> {
    if let Some(cmd) = hook {
        let success = execute_hook_command(cmd, worktree_path, env_vars)?;
        if !success {
            print_warning(format!(
                "post_start hook '{}' exited with non-zero status; retaining worktree",
                cmd
            ));
        }
        return Ok(success);
    }
    Ok(true)
}

/// Runs `pre_finish` hook: fatal gatekeeper aborting execution unless `force` is true.
pub fn run_pre_finish_hook(
    hook: Option<&str>,
    worktree_path: &Path,
    env_vars: &BTreeMap<String, String>,
    force: bool,
) -> Result<bool, HookError> {
    if let Some(cmd) = hook {
        let success = execute_hook_command(cmd, worktree_path, env_vars)?;
        if !success {
            if force {
                print_warning(format!(
                    "pre_finish hook '{}' failed, but continuing due to --force",
                    cmd
                ));
                return Ok(false);
            } else {
                return Err(HookError::PreFinishFailed {
                    hook: cmd.to_string(),
                    code: 1,
                });
            }
        }
        return Ok(true);
    }
    Ok(true)
}

/// Runs `post_finish` hook: non-fatal warning on failure.
pub fn run_post_finish_hook(
    hook: Option<&str>,
    repo_root: &Path,
    env_vars: &BTreeMap<String, String>,
) -> Result<bool, HookError> {
    if let Some(cmd) = hook {
        let success = execute_hook_command(cmd, repo_root, env_vars)?;
        if !success {
            print_warning(format!(
                "post_finish hook '{}' exited with non-zero status",
                cmd
            ));
        }
        return Ok(success);
    }
    Ok(true)
}
