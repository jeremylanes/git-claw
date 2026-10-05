use assert_cmd::Command;
use git_claw::workflow::{tag_release, TagOptions};
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn setup_repo() -> tempfile::TempDir {
    let repo_dir = tempdir().expect("repo dir");
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
    let config_content = r#"
[project]
main_branch = "main"
"#;
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

    repo_dir
}

#[test]
fn test_tag_release_clean_trunk() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    tag_release(TagOptions {
        version: "v1.0.0",
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Verify tag is created as annotated tag
    let tag_type = std::process::Command::new("git")
        .args(["cat-file", "-t", "v1.0.0"])
        .current_dir(repo_path)
        .output()
        .expect("cat-file");
    assert!(tag_type.status.success());
    assert_eq!(String::from_utf8_lossy(&tag_type.stdout).trim(), "tag");
}

#[test]
fn test_tag_release_dirty_trunk_fails() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    // Modify file without committing
    fs::write(repo_path.join("README.md"), "dirty changes").expect("dirty write");

    let res = tag_release(TagOptions {
        version: "v1.0.1",
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
}

#[test]
fn test_tag_release_invalid_semver_fails() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let res = tag_release(TagOptions {
        version: "invalid.version",
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
}

#[test]
fn test_cli_tag_command_e2e() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .args(["tag", "v2.1.0"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created annotated release tag 'v2.1.0'",
        ));

    // Verify tag exists
    let check = std::process::Command::new("git")
        .args(["tag", "-l", "v2.1.0"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert!(check.status.success());
    assert_eq!(String::from_utf8_lossy(&check.stdout).trim(), "v2.1.0");
}
