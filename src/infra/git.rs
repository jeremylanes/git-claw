//! Git CLI command wrappers for repository discovery, merge, and path resolution.

use crate::infra::error::GitError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resolves the Git common directory (`git rev-parse --git-common-dir`)
/// for repository-wide state like `claw/slots.json` and `claw/slots.lock`.
pub fn resolve_git_common_dir(repo_root: Option<&Path>) -> Result<PathBuf, GitError> {
    let mut cmd = Command::new("git");
    if let Some(root) = repo_root {
        cmd.current_dir(root);
    }
    cmd.args(["rev-parse", "--git-common-dir"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotAGitRepository);
        }
        return Err(GitError::CommandFailed {
            command: "git rev-parse --git-common-dir".to_string(),
            message: stderr.trim().to_string(),
        });
    }

    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = PathBuf::from(path_str);
    let resolved = if path.is_absolute() {
        path
    } else {
        let base = repo_root
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        base.join(path)
    };

    Ok(resolved.canonicalize().unwrap_or(resolved))
}

/// Resolves the top-level repository working tree directory (`git rev-parse --show-toplevel`).
pub fn resolve_toplevel(repo_root: Option<&Path>) -> Result<PathBuf, GitError> {
    let mut cmd = Command::new("git");
    if let Some(root) = repo_root {
        cmd.current_dir(root);
    }
    cmd.args(["rev-parse", "--show-toplevel"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotAGitRepository);
        }
        return Err(GitError::CommandFailed {
            command: "git rev-parse --show-toplevel".to_string(),
            message: stderr.trim().to_string(),
        });
    }

    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let path = PathBuf::from(path_str);
    Ok(path.canonicalize().unwrap_or(path))
}

/// Extracts repository name from top-level directory.
pub fn resolve_repo_name(toplevel: &Path) -> String {
    toplevel
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("repo")
        .to_string()
}

/// Runs `git worktree prune` to clean internal Git worktree tracking.
pub fn git_worktree_prune(repo_root: Option<&Path>) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    if let Some(root) = repo_root {
        cmd.current_dir(root);
    }
    cmd.args(["worktree", "prune"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: "git worktree prune".to_string(),
            message: stderr.trim().to_string(),
        });
    }
    Ok(())
}

/// Checks whether the Git repository working tree has any unstaged or staged modifications.
pub fn git_is_clean(repo_root: &Path) -> Result<bool, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["status", "--porcelain"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: "git status --porcelain".to_string(),
            message: stderr.trim().to_string(),
        });
    }

    Ok(output.stdout.is_empty())
}

/// Merges `branch` into the currently checked-out branch.
pub fn git_merge(repo_root: &Path, branch: &str) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["merge", branch]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git merge {}", branch),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}

/// Switches to the specified branch.
pub fn git_checkout(repo_root: &Path, branch: &str) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["checkout", branch]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git checkout {}", branch),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}

/// Resolves the abbreviated HEAD commit hash for a worktree.
pub fn git_head_commit(worktree_path: &Path) -> Result<String, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(worktree_path);
    cmd.args(["rev-parse", "--short", "HEAD"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: "git rev-parse --short HEAD".to_string(),
            message: stderr.trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Creates an annotated Git tag on `target_ref`.
pub fn git_create_annotated_tag(
    repo_root: &Path,
    tag: &str,
    target_ref: &str,
    message: &str,
) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["tag", "-a", tag, target_ref, "-m", message]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git tag -a {} {} -m {}", tag, target_ref, message),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}
