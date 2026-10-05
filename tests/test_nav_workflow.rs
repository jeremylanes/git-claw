use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::workflow::{
    cd_worktree, open_worktree, start_worktree, CdOptions, OpenOptions, StartOptions,
};
use std::fs;
use tempfile::tempdir;

fn setup_repo() -> (tempfile::TempDir, tempfile::TempDir) {
    let repo_dir = tempdir().expect("repo dir");
    let wt_parent = tempdir().expect("wt parent dir");

    let repo_path = repo_dir.path();
    let init = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git init");
    assert!(init.status.success());

    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Tester"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "tester@test.local"])
        .current_dir(repo_path)
        .output();

    fs::write(repo_path.join("README.md"), "# Test").expect("write readme");
    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[ports]
web = 3000
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");

    let add = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    assert!(add.status.success());

    let commit = std::process::Command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");
    assert!(commit.status.success());

    (repo_dir, wt_parent)
}

#[test]
fn test_cd_worktree_explicit_name() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "nav-test",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start nav-test");

    let expected_path = wt_parent.path().join("nav-test");
    let resolved = cd_worktree(CdOptions {
        name: Some("nav-test"),
        repo_root: Some(repo_path),
    })
    .expect("cd worktree");

    assert_eq!(resolved, expected_path);
}

#[test]
fn test_cli_cd_subcommand() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "cli-cd",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start cli-cd");

    let expected_path = wt_parent.path().join("cli-cd");

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    let assert = cmd
        .current_dir(repo_path)
        .args(["cd", "cli-cd"])
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8");
    assert_eq!(stdout.trim(), expected_path.to_string_lossy().trim());
}

#[test]
fn test_open_worktree_editor() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "editor-test",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start editor-test");

    // Success with 'true' editor
    let res = open_worktree(OpenOptions {
        name: Some("editor-test"),
        editor: Some("true"),
        repo_root: Some(repo_path),
    });
    assert!(res.is_ok());

    // Failure with 'false' editor
    let res_fail = open_worktree(OpenOptions {
        name: Some("editor-test"),
        editor: Some("false"),
        repo_root: Some(repo_path),
    });
    assert!(res_fail.is_err());
}

#[test]
fn test_cd_unknown_worktree_fails() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["cd", "nonexistent"])
        .assert()
        .failure()
        .code(1);
}
