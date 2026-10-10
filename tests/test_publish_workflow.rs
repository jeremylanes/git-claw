use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::workflow::{publish_worktree, start_worktree, PublishOptions, StartOptions, WorkflowError};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn setup_repo_with_remote() -> (tempfile::TempDir, tempfile::TempDir, tempfile::TempDir) {
    let remote_dir = tempdir().expect("remote dir");
    let repo_dir = tempdir().expect("repo dir");
    let wt_parent = tempdir().expect("wt parent dir");

    // Initialize bare remote repo
    let init_remote = std::process::Command::new("git")
        .args(["init", "--bare", "-b", "main"])
        .current_dir(remote_dir.path())
        .output()
        .expect("git init bare remote");
    assert!(init_remote.status.success());

    // Initialize local repo
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

    // Add origin remote
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

    // Push initial main to origin
    let push_main = std::process::Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(repo_path)
        .output()
        .expect("git push main");
    assert!(push_main.status.success());

    (remote_dir, repo_dir, wt_parent)
}

#[test]
fn test_publish_active_worktree_inference() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "login",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start login worktree");

    let wt_path = wt_parent.path().join("login");
    assert!(wt_path.exists());

    // Commit file in worktree
    fs::write(wt_path.join("login.rs"), "// login logic").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "login.rs"])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add login logic"])
        .current_dir(&wt_path)
        .output();

    // Run `git-claw feature publish` without name from inside worktree
    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(&wt_path)
        .args(["feature", "publish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch 'feature/login' successfully published to 'origin'"));

    // Verify remote bare repo has the branch
    let verify_remote = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/login"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch");
    assert!(verify_remote.status.success());

    // Verify upstream tracking was configured on the local branch
    let verify_upstream = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "feature/login@{upstream}"])
        .current_dir(&wt_path)
        .output()
        .expect("verify upstream tracking");
    assert!(verify_upstream.status.success());
    assert_eq!(
        String::from_utf8_lossy(&verify_upstream.stdout).trim(),
        "origin/feature/login"
    );
}

#[test]
fn test_publish_by_explicit_name() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "auth",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start auth worktree");

    let wt_path = wt_parent.path().join("auth");
    fs::write(wt_path.join("auth.rs"), "// auth").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add auth"])
        .current_dir(&wt_path)
        .output();

    // Run `git-claw feature publish auth` from repo root (outside worktree)
    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["feature", "publish", "auth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch 'feature/auth' successfully published to 'origin'"));

    // Verify remote bare repo has the branch
    let verify_remote = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/feature/auth"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch");
    assert!(verify_remote.status.success());
}

#[test]
fn test_publish_top_level_shorthand() {
    let (remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    start_worktree(StartOptions {
        branch_type: BranchType::Bugfix,
        name: "fix-bug",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start fix-bug worktree");

    let wt_path = wt_parent.path().join("fix-bug");
    fs::write(wt_path.join("fix.rs"), "// fix").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Fix bug"])
        .current_dir(&wt_path)
        .output();

    // Shorthand top-level: `git-claw publish fix-bug`
    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["publish", "fix-bug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch 'bugfix/fix-bug' successfully published to 'origin'"));

    let verify_remote = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/bugfix/fix-bug"])
        .current_dir(remote_dir.path())
        .output()
        .expect("verify remote branch");
    assert!(verify_remote.status.success());
}

#[test]
fn test_publish_custom_remote() {
    let (_remote_dir, repo_dir, wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    // Create a second bare remote: upstream
    let upstream_dir = tempdir().expect("upstream dir");
    let init_upstream = std::process::Command::new("git")
        .args(["init", "--bare", "-b", "main"])
        .current_dir(upstream_dir.path())
        .output()
        .expect("git init bare upstream");
    assert!(init_upstream.status.success());

    let _ = std::process::Command::new("git")
        .args(["remote", "add", "upstream", &upstream_dir.path().to_string_lossy()])
        .current_dir(repo_path)
        .output();

    start_worktree(StartOptions {
        branch_type: BranchType::Hotfix,
        name: "patch-sec",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start patch-sec worktree");

    let wt_path = wt_parent.path().join("patch-sec");
    fs::write(wt_path.join("patch.rs"), "// patch").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&wt_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Apply patch"])
        .current_dir(&wt_path)
        .output();

    // Publish targeting custom remote `upstream`
    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["hotfix", "publish", "patch-sec", "--remote", "upstream"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Branch 'hotfix/patch-sec' successfully published to 'upstream'"));

    let verify_upstream = std::process::Command::new("git")
        .args(["rev-parse", "--verify", "refs/heads/hotfix/patch-sec"])
        .current_dir(upstream_dir.path())
        .output()
        .expect("verify upstream remote branch");
    assert!(verify_upstream.status.success());
}

#[test]
fn test_publish_unknown_worktree_fails() {
    let (_remote_dir, repo_dir, _wt_parent) = setup_repo_with_remote();
    let repo_path = repo_dir.path();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["feature", "publish", "nonexistent-branch"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found in active slots registry"));
}

#[test]
fn test_publish_no_remote_configured_fails() {
    let repo_dir = tempdir().expect("repo dir");
    let wt_parent = tempdir().expect("wt parent dir");
    let repo_path = repo_dir.path();

    // Repo without any remotes
    let _ = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(repo_path)
        .output();
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
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");
    let _ = std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Initial"])
        .current_dir(repo_path)
        .output();

    start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "isolated-work",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");

    let res = publish_worktree(PublishOptions {
        branch_type: Some(BranchType::Feature),
        name: Some("isolated-work"),
        remote: None,
        repo_root: Some(repo_path),
    });

    assert!(matches!(res, Err(WorkflowError::NoRemoteConfigured)));
}
