use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::infra::registry::SlotRegistry;
use git_claw::workflow::{list_worktrees, start_worktree, StartOptions};
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
fn test_list_empty_worktrees() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active worktrees found."));
}

#[test]
fn test_list_active_worktrees_with_ports_and_commits() {
    let (repo_dir, _wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "alpha",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start alpha");

    start_worktree(StartOptions {
        branch_type: BranchType::Bugfix,
        name: "beta",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start beta");

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("SLOT"))
        .stdout(predicate::str::contains("BRANCH"))
        .stdout(predicate::str::contains("DIRECTORY"))
        .stdout(predicate::str::contains("PORTS"))
        .stdout(predicate::str::contains("COMMIT"))
        .stdout(predicate::str::contains("feature/alpha"))
        .stdout(predicate::str::contains("bugfix/beta"))
        .stdout(predicate::str::contains("PORT_WEB:3001"))
        .stdout(predicate::str::contains("PORT_WEB:3002"));
}

#[test]
fn test_list_auto_gc_orphaned_slots() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "orphan",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start orphan");

    let wt_path = wt_parent.path().join("orphan");
    assert!(wt_path.exists());

    // Manually delete directory
    fs::remove_dir_all(&wt_path).expect("delete wt dir");

    // Modify allocated_at to bypass 30s grace period for test
    let registry_path = repo_path.join(".git/claw/slots.json");
    let mut registry = SlotRegistry::load_or_empty(&registry_path).expect("load registry");
    registry.slots[0].allocated_at = "2020-01-01T00:00:00Z".to_string();
    registry.save_atomic(&registry_path).expect("save registry");

    // Run list_worktrees
    list_worktrees(Some(repo_path)).expect("list worktrees");

    // Verify registry is cleaned
    let updated_registry = SlotRegistry::load_or_empty(&registry_path).expect("reload");
    assert!(updated_registry.slots.is_empty());
}
