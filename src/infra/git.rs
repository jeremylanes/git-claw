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

/// Checks if a branch exists locally in the repository.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_branch_exists;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// let exists = git_branch_exists(Path::new("."), "main")?;
/// assert!(exists);
/// # Ok(())
/// # }
/// ```
pub fn git_branch_exists(repo_root: &Path, branch: &str) -> Result<bool, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    let ref_spec = if branch.starts_with("refs/heads/") {
        branch.to_string()
    } else {
        format!("refs/heads/{}", branch)
    };
    cmd.args(["rev-parse", "--verify", &ref_spec]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    Ok(output.status.success())
}

/// Returns the list of configured Git remotes in the repository.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_get_remotes;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// let remotes = git_get_remotes(Path::new("."))?;
/// assert!(remotes.contains(&"origin".to_string()));
/// # Ok(())
/// # }
/// ```
pub fn git_get_remotes(repo_root: &Path) -> Result<Vec<String>, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.arg("remote");

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: "git remote".to_string(),
            message: stderr.trim().to_string(),
        });
    }

    let remotes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(remotes)
}

/// Pushes the branch to the specified remote repository and configures upstream tracking.
///
/// Executes `git push -u <remote> <branch>`.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_push_upstream;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// git_push_upstream(Path::new("."), "origin", "feature/login")?;
/// # Ok(())
/// # }
/// ```
pub fn git_push_upstream(repo_root: &Path, remote: &str, branch: &str) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["push", "-u", remote, branch]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git push -u {} {}", remote, branch),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}

/// Checks whether a branch exists on the specified remote repository.
///
/// Executes `git ls-remote --heads <remote> <branch>`.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_remote_branch_exists;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// let exists = git_remote_branch_exists(Path::new("."), "origin", "feature/login")?;
/// assert!(exists);
/// # Ok(())
/// # }
/// ```
pub fn git_remote_branch_exists(
    repo_root: &Path,
    remote: &str,
    branch: &str,
) -> Result<bool, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["ls-remote", "--heads", remote, branch]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git ls-remote --heads {} {}", remote, branch),
            message: stderr.trim().to_string(),
        });
    }

    let expected_ref = if branch.starts_with("refs/heads/") {
        branch.to_string()
    } else {
        format!("refs/heads/{}", branch)
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let exists = stdout
        .lines()
        .any(|line| line.split_whitespace().nth(1) == Some(&expected_ref));

    Ok(exists)
}

/// Deletes a branch from the specified remote repository.
///
/// Executes `git push <remote> --delete <branch>`.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_delete_remote_branch;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// git_delete_remote_branch(Path::new("."), "origin", "feature/login")?;
/// # Ok(())
/// # }
/// ```
pub fn git_delete_remote_branch(
    repo_root: &Path,
    remote: &str,
    branch: &str,
) -> Result<(), GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    let branch_clean = branch.strip_prefix("refs/heads/").unwrap_or(branch);
    cmd.args(["push", remote, "--delete", branch_clean]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git push {} --delete {}", remote, branch_clean),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}

/// Adds specified files to the Git index and records a commit with the given message.
///
/// Executes `git add <files...>` followed by `git commit -m <message>`.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory.
/// * `files` - List of file paths relative to `repo_root` to stage and commit.
/// * `message` - Commit message text.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_commit_files;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// git_commit_files(Path::new("."), &["Cargo.toml"], "chore: release 0.2.0")?;
/// # Ok(())
/// # }
/// ```
pub fn git_commit_files(repo_root: &Path, files: &[&str], message: &str) -> Result<(), GitError> {
    if files.is_empty() {
        return Ok(());
    }

    let mut add_cmd = Command::new("git");
    add_cmd.current_dir(repo_root);
    add_cmd.arg("add");
    add_cmd.args(files);

    let add_output = add_cmd.output().map_err(GitError::ProcessFailed)?;
    if !add_output.status.success() {
        let stderr = String::from_utf8_lossy(&add_output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git add {}", files.join(" ")),
            message: stderr.trim().to_string(),
        });
    }

    let mut commit_cmd = Command::new("git");
    commit_cmd.current_dir(repo_root);
    commit_cmd.args(["commit", "-m", message]);

    let commit_output = commit_cmd.output().map_err(GitError::ProcessFailed)?;
    if !commit_output.status.success() {
        let stderr = String::from_utf8_lossy(&commit_output.stderr);
        return Err(GitError::CommandFailed {
            command: format!("git commit -m {}", message),
            message: stderr.trim().to_string(),
        });
    }

    Ok(())
}

/// Returns the latest tag reachable from the current commit, or `None` if no tags exist.
///
/// Executes `git describe --tags --abbrev=0`.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_get_latest_tag;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// let latest = git_get_latest_tag(Path::new("."))?;
/// if let Some(tag) = latest {
///     println!("Latest tag: {}", tag);
/// }
/// # Ok(())
/// # }
/// ```
pub fn git_get_latest_tag(repo_root: &Path) -> Result<Option<String>, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    cmd.args(["describe", "--tags", "--abbrev=0"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if output.status.success() {
        let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if tag.is_empty() {
            Ok(None)
        } else {
            Ok(Some(tag))
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotAGitRepository);
        }
        Ok(None)
    }
}

/// Retrieves all commit messages since the given tag (or all commits up to HEAD if `tag` is `None`).
///
/// Executes `git log <tag>..HEAD --format=%B%x1e` or `git log HEAD --format=%B%x1e`.
/// Splits the output on ASCII 0x1E records, trims whitespace, and filters empty entries.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory.
/// * `tag` - Optional starting tag reference.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use git_claw::infra::git::git_get_commits_since;
///
/// # fn run() -> Result<(), git_claw::infra::error::GitError> {
/// let commits = git_get_commits_since(Path::new("."), Some("v1.0.0"))?;
/// for c in commits {
///     println!("Commit: {}", c);
/// }
/// # Ok(())
/// # }
/// ```
pub fn git_get_commits_since(repo_root: &Path, tag: Option<&str>) -> Result<Vec<String>, GitError> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root);
    let range = match tag {
        Some(t) => format!("{}..HEAD", t),
        None => "HEAD".to_string(),
    };
    cmd.args(["log", &range, "--format=%B%x1e"]);

    let output = cmd.output().map_err(GitError::ProcessFailed)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err(GitError::NotAGitRepository);
        }
        if stderr.contains("does not have any commits yet") {
            return Ok(Vec::new());
        }
        return Err(GitError::CommandFailed {
            command: format!("git log {}", range),
            message: stderr.trim().to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let commits = stdout
        .split('\x1e')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(commits)
}
