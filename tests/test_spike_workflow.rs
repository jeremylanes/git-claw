use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::infra::registry::SlotRegistry;
use git_claw::workflow::{drop_spike, start_worktree, DropSpikeOptions, StartOptions};
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
fn test_spike_start_and_drop_success() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // 1. Start spike worktree
    start_worktree(StartOptions {
        branch_type: BranchType::Spike,
        name: "experiment",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start spike");

    let wt_path = wt_parent.path().join("experiment");
    assert!(wt_path.exists());

    // 2. Commit file inside worktree
    fs::write(wt_path.join("experiment.txt"), "throwaway").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Spike commit"])
        .current_dir(&wt_path)
        .output();

    // 3. Drop spike worktree
    drop_spike(DropSpikeOptions {
        name: "experiment",
        repo_root: Some(repo_path),
    })
    .expect("drop spike");

    // 4. Verify worktree directory is removed
    assert!(!wt_path.exists());

    // 5. Verify registry slot is freed
    let registry_path = repo_path.join(".git/claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path).expect("load registry");
    assert!(registry.slots.is_empty());

    // 6. Verify branch spike/experiment is deleted
    let branch_check = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "spike/experiment"])
        .current_dir(repo_path)
        .output()
        .expect("branch check");
    assert!(!branch_check.status.success());

    // 7. Verify main branch did NOT get the commit or file
    assert!(!repo_path.join("experiment.txt").exists());
}

#[test]
fn test_spike_drop_unknown_worktree_fails() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let res = drop_spike(DropSpikeOptions {
        name: "nonexistent",
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
}

#[test]
fn test_cli_spike_drop_e2e() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // Start spike via CLI
    let mut cmd_start = Command::cargo_bin("git-claw").expect("binary");
    cmd_start
        .current_dir(repo_path)
        .args(["spike", "start", "cli-spike"])
        .assert()
        .success();

    let wt_path = wt_parent.path().join("cli-spike");
    assert!(wt_path.exists());

    // Drop spike via CLI
    let mut cmd_drop = Command::cargo_bin("git-claw").expect("binary");
    cmd_drop
        .current_dir(repo_path)
        .args(["spike", "drop", "cli-spike"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "dropped successfully without merging",
        ));

    assert!(!wt_path.exists());

    // Verify registry is empty
    let registry_path = repo_path.join(".git/claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path).expect("load registry");
    assert!(registry.slots.is_empty());
}
