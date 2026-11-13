//! Lifecycle hook execution and policy enforcement.

use crate::cli::output::print_warning;
use crate::infra::error::HookError;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Executes a shell command inside `working_dir` with `env_vars` injected.
///
/// When `quiet_stdout` is `true`, child stdout and stderr are captured and forwarded
/// to stderr so parent stdout stays clean for structured output (such as `--json`).
///
/// # Examples
///
/// ```
/// use git_claw::infra::hook::execute_hook_command_with_output;
/// use std::collections::BTreeMap;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// let ok = execute_hook_command_with_output("echo ok", dir.path(), &BTreeMap::new(), true).unwrap();
/// assert!(ok);
/// ```
pub fn execute_hook_command_with_output(
    command_str: &str,
    working_dir: &Path,
    env_vars: &BTreeMap<String, String>,
    quiet_stdout: bool,
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

    if quiet_stdout {
        let output = cmd.output().map_err(|e| {
            HookError::ExecutionFailed(format!("Failed to spawn hook command: {}", e))
        })?;
        let mut stderr = std::io::stderr();
        if !output.stdout.is_empty() {
            let _ = stderr.write_all(&output.stdout);
        }
        if !output.stderr.is_empty() {
            let _ = stderr.write_all(&output.stderr);
        }
        return Ok(output.status.success());
    }

    let status = cmd
        .status()
        .map_err(|e| HookError::ExecutionFailed(format!("Failed to spawn hook command: {}", e)))?;

    Ok(status.success())
}

/// Executes a shell command inside `working_dir` with `env_vars` injected.
pub fn execute_hook_command(
    command_str: &str,
    working_dir: &Path,
    env_vars: &BTreeMap<String, String>,
) -> Result<bool, HookError> {
    execute_hook_command_with_output(command_str, working_dir, env_vars, false)
}

/// Runs `post_start` hook with configurable stdout suppression.
///
/// # Examples
///
/// ```
/// use git_claw::infra::hook::run_post_start_hook_with_output;
/// use std::collections::BTreeMap;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// assert!(run_post_start_hook_with_output(None, dir.path(), &BTreeMap::new(), true).unwrap());
/// ```
pub fn run_post_start_hook_with_output(
    hook: Option<&str>,
    worktree_path: &Path,
    env_vars: &BTreeMap<String, String>,
    quiet_stdout: bool,
) -> Result<bool, HookError> {
    if let Some(cmd) = hook {
        let success = execute_hook_command_with_output(cmd, worktree_path, env_vars, quiet_stdout)?;
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

/// Runs `post_start` hook: non-fatal warning on failure, does not abort worktree creation.
pub fn run_post_start_hook(
    hook: Option<&str>,
    worktree_path: &Path,
    env_vars: &BTreeMap<String, String>,
) -> Result<bool, HookError> {
    run_post_start_hook_with_output(hook, worktree_path, env_vars, false)
}

/// Runs `pre_finish` hook with configurable stdout suppression.
///
/// # Examples
///
/// ```
/// use git_claw::infra::hook::run_pre_finish_hook_with_output;
/// use std::collections::BTreeMap;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// assert!(run_pre_finish_hook_with_output(None, dir.path(), &BTreeMap::new(), false, true).unwrap());
/// ```
pub fn run_pre_finish_hook_with_output(
    hook: Option<&str>,
    worktree_path: &Path,
    env_vars: &BTreeMap<String, String>,
    force: bool,
    quiet_stdout: bool,
) -> Result<bool, HookError> {
    if let Some(cmd) = hook {
        let success = execute_hook_command_with_output(cmd, worktree_path, env_vars, quiet_stdout)?;
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

/// Runs `pre_finish` hook: fatal gatekeeper aborting execution unless `force` is true.
pub fn run_pre_finish_hook(
    hook: Option<&str>,
    worktree_path: &Path,
    env_vars: &BTreeMap<String, String>,
    force: bool,
) -> Result<bool, HookError> {
    run_pre_finish_hook_with_output(hook, worktree_path, env_vars, force, false)
}

/// Runs `post_finish` hook with configurable stdout suppression.
///
/// # Examples
///
/// ```
/// use git_claw::infra::hook::run_post_finish_hook_with_output;
/// use std::collections::BTreeMap;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// assert!(run_post_finish_hook_with_output(None, dir.path(), &BTreeMap::new(), true).unwrap());
/// ```
pub fn run_post_finish_hook_with_output(
    hook: Option<&str>,
    repo_root: &Path,
    env_vars: &BTreeMap<String, String>,
    quiet_stdout: bool,
) -> Result<bool, HookError> {
    if let Some(cmd) = hook {
        let success = execute_hook_command_with_output(cmd, repo_root, env_vars, quiet_stdout)?;
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

/// Runs `post_finish` hook: non-fatal warning on failure.
pub fn run_post_finish_hook(
    hook: Option<&str>,
    repo_root: &Path,
    env_vars: &BTreeMap<String, String>,
) -> Result<bool, HookError> {
    run_post_finish_hook_with_output(hook, repo_root, env_vars, false)
}
