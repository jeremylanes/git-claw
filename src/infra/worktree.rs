//! Git worktree lifecycle runner primitives.

use crate::infra::error::WorktreeError;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Creates a new worktree at `worktree_path` checked out on a newly created `branch` based on `base_branch`.
/// Cleans up any partially created directory if `git worktree add` fails.
pub fn create_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    branch: &str,
    base_branch: &str,
) -> Result<(), WorktreeError> {
    if worktree_path.exists() {
        return Err(WorktreeError::DirectoryAlreadyExists(
            worktree_path.to_string_lossy().to_string(),
        ));
    }

    if let Some(parent) = worktree_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args([
        "worktree",
        "add",
        "-b",
        branch,
        &worktree_path.to_string_lossy(),
        base_branch,
    ]);

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Attempt rollback of created directory if it exists
        if worktree_path.exists() {
            let _ = fs::remove_dir_all(worktree_path);
        }

        // Clean up git metadata
        let mut prune_cmd = Command::new("git");
        prune_cmd.current_dir(repo_root);
        prune_cmd.args(["worktree", "prune"]);
        let _ = prune_cmd.output();

        if stderr.contains("already exists") {
            return Err(WorktreeError::BranchAlreadyExists(branch.to_string()));
        }

        return Err(WorktreeError::CreationFailed {
            path: worktree_path.to_string_lossy().to_string(),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}

/// Removes a worktree via `git worktree remove` (optionally `--force`) and prunes worktree metadata.
pub fn remove_worktree(
    repo_root: &Path,
    worktree_path: &Path,
    force: bool,
) -> Result<(), WorktreeError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.arg("worktree");
    cmd.arg("remove");
    if force {
        cmd.arg("--force");
    }
    cmd.arg(worktree_path);

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WorktreeError::RemovalFailed {
            path: worktree_path.to_string_lossy().to_string(),
            message: stderr.trim().to_string(),
        });
    }

    // Prune stale metadata
    let mut prune = Command::new("git");
    prune.current_dir(repo_root);
    prune.args(["worktree", "prune"]);
    let _ = prune.output();

    Ok(())
}

/// Deletes a local Git branch (`-d` or `-D` if force).
pub fn delete_branch(repo_root: &Path, branch: &str, force: bool) -> Result<(), WorktreeError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.arg("branch");
    if force {
        cmd.arg("-D");
    } else {
        cmd.arg("-d");
    }
    cmd.arg(branch);

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WorktreeError::RemovalFailed {
            path: branch.to_string(),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}
