use git_claw::infra::error::WorktreeError;
use git_claw::infra::worktree::{create_worktree, delete_branch, remove_worktree};
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn setup_test_git_repo() -> tempfile::TempDir {
    let dir = tempdir().expect("tempdir");
    let path = dir.path();

    let init = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(path)
        .output()
        .expect("git init");
    assert!(init.status.success());

    // Configure user name and email for commits in temporary repo
    let _ = Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(path)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "tester@test.local"])
        .current_dir(path)
        .output();

    // Create an initial commit so main branch exists
    let readme = path.join("README.md");
    fs::write(&readme, "# Test Repo").expect("writes readme");

    let add = Command::new("git")
        .args(["add", "README.md"])
        .current_dir(path)
        .output()
        .expect("git add");
    assert!(add.status.success());

    let commit = Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .expect("git commit");
    assert!(commit.status.success());

    dir
}

#[test]
fn test_create_and_remove_worktree() {
    let repo_dir = setup_test_git_repo();
    let repo_path = repo_dir.path();
    let worktree_dir = tempdir().expect("worktree parent dir");
    let worktree_path = worktree_dir.path().join("worktree-feat");

    create_worktree(repo_path, &worktree_path, "feature/my-feat", "main")
        .expect("creates worktree");

    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Remove worktree
    remove_worktree(repo_path, &worktree_path, false).expect("removes worktree");
    assert!(!worktree_path.exists());

    // Delete branch
    delete_branch(repo_path, "feature/my-feat", false).expect("deletes branch");
}

#[test]
fn test_create_worktree_branch_already_exists() {
    let repo_dir = setup_test_git_repo();
    let repo_path = repo_dir.path();
    let worktree_dir = tempdir().expect("worktree parent dir");
    let worktree_path1 = worktree_dir.path().join("wt1");
    let worktree_path2 = worktree_dir.path().join("wt2");

    create_worktree(repo_path, &worktree_path1, "feature/dup", "main").expect("creates first");

    let err = create_worktree(repo_path, &worktree_path2, "feature/dup", "main").unwrap_err();
    assert!(matches!(err, WorktreeError::BranchAlreadyExists(_)));
    assert!(!worktree_path2.exists());
}
