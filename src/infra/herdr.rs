//! Herdr terminal multiplexer integration.
//!
//! Provides automatic detection and activation of worktrees in Herdr workspaces.

use crate::cli::output::{print_success, print_warning};
use std::path::Path;
use std::process::Command;

/// Checks whether Herdr auto-activation is enabled and safe to run.
///
/// Returns `false` if:
/// - `HERDR_ENV` is unset or not equal to `"1"`
/// - `CLAW_DISABLE_HERDR` is set
/// - Running inside a test harness (`CARGO_MANIFEST_DIR` or `RUST_TEST_THREADS` is set)
///
/// # Example
///
/// ```
/// use git_claw::infra::herdr::is_herdr_active;
///
/// // Returns false in tests to prevent polluting developer sessions
/// assert!(!is_herdr_active());
/// ```
pub fn is_herdr_active() -> bool {
    if std::env::var("HERDR_ENV").as_deref() != Ok("1") {
        return false;
    }
    if std::env::var("CLAW_DISABLE_HERDR").is_ok() {
        return false;
    }
    if std::env::var("CARGO_MANIFEST_DIR").is_ok() || std::env::var("RUST_TEST_THREADS").is_ok() {
        return false;
    }
    true
}

/// Opens a newly created worktree branch in Herdr multiplexer.
///
/// Returns `false` without executing if Herdr is inactive or if the repository
/// path is within a temporary directory (e.g., during tests).
///
/// # Example
///
/// ```
/// use git_claw::infra::herdr::open_worktree_in_herdr;
/// use std::path::Path;
///
/// // Temporary directory paths are rejected immediately
/// assert!(!open_worktree_in_herdr(Path::new("/tmp/test-repo"), "feature/sample"));
/// ```
pub fn open_worktree_in_herdr(repo_root: &Path, branch: &str) -> bool {
    if !is_herdr_active() {
        return false;
    }

    let repo_str = repo_root.to_string_lossy();
    if repo_root.starts_with(std::env::temp_dir())
        || repo_str.contains("/tmp/")
        || repo_str.starts_with("/tmp")
    {
        return false;
    }

    let cwd = repo_root.to_str().unwrap_or(".");
    match Command::new("herdr")
        .args(["worktree", "open", "--cwd", cwd, "--branch", branch])
        .status()
    {
        Ok(status) if status.success() => {
            print_success(format!("Opened worktree in Herdr: {}", branch));
            true
        }
        Ok(status) => {
            print_warning(format!(
                "Herdr worktree open exited with code: {:?}",
                status.code()
            ));
            false
        }
        Err(err) => {
            print_warning(format!("Could not execute herdr: {}", err));
            false
        }
    }
}
