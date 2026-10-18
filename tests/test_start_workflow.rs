use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::infra::registry::SlotRegistry;
use git_claw::workflow::{start_worktree, StartOptions};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn setup_git_repo_with_config() -> (tempfile::TempDir, tempfile::TempDir) {
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
fn test_start_worktree_workflow() {
    let (repo_dir, wt_parent) = setup_git_repo_with_config();
    let repo_path = repo_dir.path();

    // 1. Start feature worktree
    let slot_1 = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "auth",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("starts first worktree");
    assert_eq!(slot_1, 1);

    let wt_path = wt_parent.path().join("auth");
    assert!(wt_path.exists());
    assert!(wt_path.join(".env.worktree").exists());

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("reads env");
    assert!(env_content.contains("CLAW_SLOT_ID=1"));
    assert!(env_content.contains("PORT_WEB=3001"));

    // Check registry
    let registry_path = repo_path.join(".git/claw/slots.json");
    let registry = SlotRegistry::load_or_empty(&registry_path).expect("loads registry");
    assert_eq!(registry.slots.len(), 1);
    assert_eq!(registry.slots[0].id, 1);
    assert_eq!(registry.slots[0].name, "auth");
    assert_eq!(registry.slots[0].branch, "feature/auth");

    // 2. Start second worktree
    let slot_2 = start_worktree(StartOptions {
        branch_type: BranchType::Bugfix,
        name: "fix-login",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("starts second worktree");
    assert_eq!(slot_2, 2);

    // 3. Duplicate name fails
    let dup_res = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "auth",
        isolated: false,
        repo_root: Some(repo_path),
    });
    assert!(dup_res.is_err());
}

#[test]
fn test_cli_feature_start_e2e() {
    let (repo_dir, wt_parent) = setup_git_repo_with_config();
    let repo_path = repo_dir.path();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.current_dir(repo_path);
    cmd.args(["feature", "start", "dashboard"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Worktree 'dashboard' started successfully",
        ))
        .stdout(predicate::str::contains("Slot: 1"));

    assert!(wt_parent.path().join("dashboard").exists());
}

#[test]
fn test_start_worktree_environment_propagation() {
    let (repo_dir, wt_parent) = setup_git_repo_with_config();
    let repo_path = repo_dir.path();

    // 1. Configure [environment] in .git-claw.toml
    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[environment]
type = "pyenv"
name = "tune"
copy_files = [".python-version"]

[cache]
directories = [".venv"]
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");

    // 2. Create environment files and shared .venv in repo root
    fs::write(repo_path.join(".python-version"), "tune\n").expect("write .python-version");
    fs::write(repo_path.join(".nvmrc"), "v20.10.0\n").expect("write .nvmrc");
    fs::create_dir_all(repo_path.join(".venv")).expect("create .venv");

    // 3. Start feature worktree
    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "env-test",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("starts worktree");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("env-test");
    assert!(wt_path.exists());

    // 4. Verify runtime files were propagated
    let py_ver = wt_path.join(".python-version");
    assert!(py_ver.exists(), ".python-version must exist in worktree");
    assert_eq!(fs::read_to_string(&py_ver).unwrap(), "tune\n");

    let nvmrc = wt_path.join(".nvmrc");
    assert!(nvmrc.exists(), ".nvmrc must exist in worktree");
    assert_eq!(fs::read_to_string(&nvmrc).unwrap(), "v20.10.0\n");

    // 5. Verify .venv symlink exists
    assert!(
        wt_path.join(".venv").exists(),
        ".venv must exist in worktree"
    );

    // 6. Verify .env.worktree has PYENV_VERSION and VIRTUAL_ENV / PATH
    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("reads env");
    assert!(env_content.contains("PYENV_VERSION=tune"));
    assert!(env_content.contains(&format!("VIRTUAL_ENV={}/.venv", wt_path.display())));
    assert!(env_content.contains(&format!("PATH={}/.venv/bin:$PATH", wt_path.display())));
}
