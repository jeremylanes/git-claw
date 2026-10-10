use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::infra::registry::SlotRegistry;
use git_claw::workflow::{
    finish_worktree, publish_worktree, start_worktree, FinishOptions, PublishOptions, StartOptions,
};
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
        keep_remote: false,
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
        keep_remote: false,
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
    assert!(wt_path.exists());

    // Finish with force should succeed
    let res_force = finish_worktree(FinishOptions {
        name: Some("hook-test"),
        force: true,
        keep_remote: false,
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

fn setup_repo_with_remote() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let remote_dir = tempdir().expect("remote dir");
    let repo_dir = tempdir().expect("repo dir");
    let wt_parent = tempdir().expect("wt parent dir");

    let init_remote = std::process::Command::new("git")
        .args(["init", "--bare", "-b", "main"])
        .current_dir(remote_dir.path())
        .output()
        .expect("git init bare remote");
    assert!(init_remote.status.success());

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

    let add_remote = std::process::Command::new("git")
        .args(["remote", "add", "origin", &remote_dir.path().to_string_lossy()])
        .current_dir(repo_path)
        .output()
        .expect("git remote add");
    assert!(add_remote.status.success());

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

    let push_main = std::process::Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git push main");
    assert!(push_main.status.success());

    (remote_dir, repo_dir, wt_parent)
}

#[test]
fn test_finish_deletes_published_remote_branch() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "rem-del",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("rem-del");
    fs::write(wt_path.join("code.txt"), "code").expect("write code");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add code"])
        .current_dir(&wt_path)
        .output();

    publish_worktree(PublishOptions {
        branch_type: Some(BranchType::Feature),
        name: Some("rem-del"),
        remote: None,
        repo_root: Some(repo_path),
    })
    .expect("publish worktree");

    let verify = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/rem-del"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch");
    assert!(verify.status.success());

    finish_worktree(FinishOptions {
        name: Some("rem-del"),
        force: false,
        keep_remote: false,
        repo_root: Some(repo_path),
    })
    .expect("finish worktree");

    let verify_after = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/rem-del"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch deleted");
    assert!(!verify_after.status.success());
    assert!(!wt_path.exists());
    assert!(repo_path.join("code.txt").exists());
}

#[test]
fn test_finish_retains_remote_branch_when_keep_remote_true() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "rem-keep",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("rem-keep");
    fs::write(wt_path.join("keep.txt"), "keep").expect("write keep");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Keep code"])
        .current_dir(&wt_path)
        .output();

    publish_worktree(PublishOptions {
        branch_type: Some(BranchType::Feature),
        name: Some("rem-keep"),
        remote: None,
        repo_root: Some(repo_path),
    })
    .expect("publish worktree");

    let verify = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/rem-keep"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch");
    assert!(verify.status.success());

    finish_worktree(FinishOptions {
        name: Some("rem-keep"),
        force: false,
        keep_remote: true,
        repo_root: Some(repo_path),
    })
    .expect("finish worktree");

    let verify_after = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/rem-keep"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch retained");
    assert!(verify_after.status.success());
    assert!(!wt_path.exists());
    assert!(repo_path.join("keep.txt").exists());
}

#[test]
fn test_cli_finish_remote_cleanup_and_keep_remote_flag() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "cli-rem",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("cli-rem");
    fs::write(wt_path.join("clirem.txt"), "clirem").expect("write clirem");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "CLI remote commit"])
        .current_dir(&wt_path)
        .output();

    publish_worktree(PublishOptions {
        branch_type: Some(BranchType::Feature),
        name: Some("cli-rem"),
        remote: None,
        repo_root: Some(repo_path),
    })
    .expect("publish worktree");

    let mut cmd_finish = Command::cargo_bin("git-claw").expect("binary");
    cmd_finish
        .current_dir(repo_path)
        .args(["finish", "cli-rem"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Remote branch 'feature/cli-rem' deleted from 'origin'",
        ));

    let verify_deleted = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/cli-rem"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch deleted");
    assert!(!verify_deleted.status.success());

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "cli-keep",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path2 = wt_parent.path().join("cli-keep");
    fs::write(wt_path2.join("clikeep.txt"), "clikeep").expect("write clikeep");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path2)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "CLI keep commit"])
        .current_dir(&wt_path2)
        .output();

    publish_worktree(PublishOptions {
        branch_type: Some(BranchType::Feature),
        name: Some("cli-keep"),
        remote: None,
        repo_root: Some(repo_path),
    })
    .expect("publish worktree");

    let mut cmd_finish_keep = Command::cargo_bin("git-claw").expect("binary");
    cmd_finish_keep
        .current_dir(repo_path)
        .args(["finish", "cli-keep", "--keep-remote"])
        .assert()
        .success();

    let verify_kept = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/cli-keep"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch kept");
    assert!(verify_kept.status.success());
}

#[test]
fn test_finish_remote_unreachable_is_non_fatal() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "rem-unreach",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let wt_path = wt_parent.path().join("rem-unreach");
    fs::write(wt_path.join("file.txt"), "content").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Remote unreachable test"])
        .current_dir(&wt_path)
        .output();

    // Corrupt remote directory so remote becomes unreachable
    let _ = fs::remove_dir_all(remote_dir.path());

    // Finish should succeed locally despite remote error
    let res = finish_worktree(FinishOptions {
        name: Some("rem-unreach"),
        force: false,
        keep_remote: false,
        repo_root: Some(repo_path),
    });
    assert!(res.is_ok());
    assert!(!wt_path.exists());
    assert!(repo_path.join("file.txt").exists());
}


