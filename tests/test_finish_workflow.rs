use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::infra::registry::SlotRegistry;
use git_claw::workflow::{finish_worktree, start_worktree, FinishOptions, StartOptions};
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
fn test_finish_worktree_merges_and_cleans_up() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // Start worktree
    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "login",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("login");
    assert!(wt_path.exists());

    // Commit file in worktree
    fs::write(wt_path.join("login.txt"), "login code").expect("write login.txt");
    let add = std::process::Command::new("git")
        .args(["add", "login.txt"])
        .current_dir(&wt_path)
        .output()
        .expect("git add in wt");
    assert!(add.status.success());
    let commit = std::process::Command::new("git")
        .args(["commit", "-m", "Add login"])
        .current_dir(&wt_path)
        .output()
        .expect("git commit in wt");
    assert!(commit.status.success());

    // Create untracked IDE folder to simulate IDE activity
    let idea_dir = wt_path.join(".idea");
    fs::create_dir_all(&idea_dir).expect("create .idea");
    fs::write(idea_dir.join("workspace.xml"), "<xml/>").expect("write workspace.xml");

    // Finish worktree
    finish_worktree(FinishOptions {
        name: Some("login"),
        force: false,
        repo_root: Some(repo_path),
    })
    .expect("finish worktree");

    // Verify main branch has login.txt
    assert!(repo_path.join("login.txt").exists());
    let content = fs::read_to_string(repo_path.join("login.txt")).expect("read merged file");
    assert_eq!(content, "login code");

    // Verify worktree directory is removed
    assert!(!wt_path.exists());

    // Verify registry slot is freed
    let registry_path = repo_path.join(".git/claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path).expect("load registry");
    assert!(registry.slots.is_empty());
}

#[test]
fn test_finish_worktree_pre_hook_abort_and_force() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[hooks]
pre_finish = "exit 1"
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");
    let _ = std::process::Command::new("git")
        .args(["commit", "-am", "Update hooks config"])
        .current_dir(repo_path)
        .output();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "hook-test",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("hook-test");
    assert!(wt_path.exists());

    fs::write(wt_path.join("hook.txt"), "hook code").expect("write hook.txt");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Hook commit"])
        .current_dir(&wt_path)
        .output();

    // Finish without force should fail
    let res = finish_worktree(FinishOptions {
        name: Some("hook-test"),
        force: false,
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
    assert!(wt_path.exists());

    // Finish with force should succeed
    let res_force = finish_worktree(FinishOptions {
        name: Some("hook-test"),
        force: true,
        repo_root: Some(repo_path),
    });
    assert!(res_force.is_ok());
    assert!(!wt_path.exists());
    assert!(repo_path.join("hook.txt").exists());
}

#[test]
fn test_cli_finish_subcommand() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // Start worktree via CLI
    let mut cmd_start = Command::cargo_bin("git-claw").expect("binary");
    cmd_start
        .current_dir(repo_path)
        .args(["feature", "start", "cli-finish"])
        .assert()
        .success();

    let wt_path = wt_parent.path().join("cli-finish");
    assert!(wt_path.exists());

    // Finish worktree via CLI
    let mut cmd_finish = Command::cargo_bin("git-claw").expect("binary");
    cmd_finish
        .current_dir(repo_path)
        .args(["finish", "cli-finish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("successfully merged into 'main'"));

    assert!(!wt_path.exists());
}
