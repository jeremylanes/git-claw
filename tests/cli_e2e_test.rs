use assert_cmd::Command;
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

    fs::write(repo_path.join("README.md"), "# E2E Test").expect("write readme");
    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[ports]
web = 8080
api = 3000
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
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");
    assert!(commit.status.success());

    (repo_dir, wt_parent)
}

#[test]
fn test_e2e_complete_feature_lifecycle() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // 1. Start feature worktree
    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["feature", "start", "billing"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Worktree 'billing' started successfully",
        ))
        .stdout(predicate::str::contains("Slot: 1"));

    let wt_path = wt_parent.path().join("billing");
    assert!(wt_path.exists());
    assert!(wt_path.join(".env.worktree").exists());

    // 2. Query worktree path via 'cd'
    let mut cmd_cd = Command::cargo_bin("git-claw").expect("binary");
    let cd_assert = cmd_cd
        .current_dir(repo_path)
        .args(["cd", "billing"])
        .assert()
        .success();
    let cd_stdout = String::from_utf8(cd_assert.get_output().stdout.clone()).expect("utf8");
    assert_eq!(cd_stdout.trim(), wt_path.to_string_lossy().trim());

    // 3. Inspect dashboard via 'list'
    let mut cmd_list = Command::cargo_bin("git-claw").expect("binary");
    cmd_list
        .current_dir(repo_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("billing"))
        .stdout(predicate::str::contains("PORT_API:3001"))
        .stdout(predicate::str::contains("PORT_WEB:8081"));

    // 4. Run command inside worktree via 'run'
    let mut cmd_run = Command::cargo_bin("git-claw").expect("binary");
    cmd_run
        .current_dir(repo_path)
        .args([
            "run",
            "billing",
            "sh",
            "-c",
            "echo IN_WORKTREE=$CLAW_WORKTREE_NAME:$PORT_WEB > feature.txt",
        ])
        .assert()
        .success();

    let feature_file = wt_path.join("feature.txt");
    assert!(feature_file.exists());
    let feature_content = fs::read_to_string(&feature_file).expect("read feature.txt");
    assert_eq!(feature_content.trim(), "IN_WORKTREE=billing:8081");

    // 5. Commit change in worktree
    let _ = std::process::Command::new("git")
        .args(["add", "feature.txt"])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Implement billing feature"])
        .current_dir(&wt_path)
        .output();

    // 6. Finish worktree (runs tests, merges into main, deletes worktree and branch)
    let mut cmd_finish = Command::cargo_bin("git-claw").expect("binary");
    cmd_finish
        .current_dir(repo_path)
        .args(["finish", "billing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("successfully merged into 'main'"));

    // Assert worktree directory is cleaned up
    assert!(!wt_path.exists());

    // Assert main branch contains merged file
    assert!(repo_path.join("feature.txt").exists());
    let main_content = fs::read_to_string(repo_path.join("feature.txt")).expect("read merged");
    assert_eq!(main_content.trim(), "IN_WORKTREE=billing:8081");

    // 7. Verify list is now empty
    let mut cmd_list_after = Command::cargo_bin("git-claw").expect("binary");
    cmd_list_after
        .current_dir(repo_path)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No active worktrees found."));

    // 8. Create SemVer release tag
    let mut cmd_tag = Command::cargo_bin("git-claw").expect("binary");
    cmd_tag
        .current_dir(repo_path)
        .args(["tag", "v1.0.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created annotated release tag 'v1.0.0'",
        ));

    // Verify tag in git
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", "v1.0.0"])
        .current_dir(repo_path)
        .output()
        .expect("tag check");
    assert!(tag_check.status.success());
    assert_eq!(String::from_utf8_lossy(&tag_check.stdout).trim(), "v1.0.0");
}

#[test]
fn test_e2e_spike_abandonment_lifecycle() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // 1. Start spike
    let mut cmd_spike = Command::cargo_bin("git-claw").expect("binary");
    cmd_spike
        .current_dir(repo_path)
        .args(["spike", "start", "prototype"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Worktree 'prototype' started successfully",
        ));

    let wt_path = wt_parent.path().join("prototype");
    assert!(wt_path.exists());

    // 2. Commit experiment file inside spike
    fs::write(wt_path.join("experiment.txt"), "throwaway code").expect("write experiment");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Spike prototype"])
        .current_dir(&wt_path)
        .output();

    // 3. Drop spike
    let mut cmd_drop = Command::cargo_bin("git-claw").expect("binary");
    cmd_drop
        .current_dir(repo_path)
        .args(["spike", "drop", "prototype"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "dropped successfully without merging",
        ));

    // 4. Verify cleanup and trunk purity
    assert!(!wt_path.exists());
    assert!(!repo_path.join("experiment.txt").exists());

    let branch_check = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "spike/prototype"])
        .current_dir(repo_path)
        .output()
        .expect("branch check");
    assert!(!branch_check.status.success());
}
