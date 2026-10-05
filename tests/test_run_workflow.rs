use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::workflow::{run_command, start_worktree, RunOptions, StartOptions};
use predicates::prelude::*;
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
fn test_run_command_explicit_name_env_injection() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "runner",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start runner");

    let code = run_command(RunOptions {
        name: Some("runner".to_string()),
        command: vec![
            "sh".to_string(),
            "-c".to_string(),
            "test \"$CLAW_SLOT_ID\" = \"1\" && test \"$PORT_WEB\" = \"3001\"".to_string(),
        ],
        repo_root: Some(repo_path.to_path_buf()),
    })
    .expect("run command");

    assert_eq!(code, 0);
}

#[test]
fn test_run_command_cli_forwarding() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "cli-run",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start cli-run");

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args([
            "run",
            "cli-run",
            "sh",
            "-c",
            "echo WORKTREE_ENV=$CLAW_WORKTREE_NAME",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("WORKTREE_ENV=cli-run"));
}

#[test]
fn test_run_command_exit_code_forwarded() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "code-test",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start code-test");

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["run", "code-test", "sh", "-c", "exit 42"])
        .assert()
        .failure()
        .code(42);
}

#[test]
fn test_run_command_unknown_worktree_fails() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["run", "nonexistent", "echo", "hi"])
        .assert()
        .failure()
        .code(1);
}
