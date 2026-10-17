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

#[test]
fn test_tag_release_updates_and_commits_cargo_toml() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let cargo_toml_content = r#"[package]
name = "my-test-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
"#;
    fs::write(repo_path.join("Cargo.toml"), cargo_toml_content).expect("write Cargo.toml");

    let add = std::process::Command::new("git")
        .args(["add", "Cargo.toml"])
        .current_dir(repo_path)
        .output()
        .expect("git add");
    assert!(add.status.success());
    let commit = std::process::Command::new("git")
        .args(["commit", "-m", "Add Cargo.toml"])
        .current_dir(repo_path)
        .output()
        .expect("git commit");
    assert!(commit.status.success());

    tag_release(TagOptions {
        version: "v0.3.0",
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Confirms Cargo.toml now contains version = "0.3.0"
    let updated_cargo_toml =
        fs::read_to_string(repo_path.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(updated_cargo_toml.contains("version = \"0.3.0\""));

    // HEAD commit message is chore: release v0.3.0
    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "chore: release v0.3.0"
    );

    // Tag v0.3.0 points to HEAD
    let head_rev = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("rev-parse HEAD");
    let tag_rev = std::process::Command::new("git")
        .args(["rev-parse", "v0.3.0^{commit}"])
        .current_dir(repo_path)
        .output()
        .expect("rev-parse tag");
    assert_eq!(
        String::from_utf8_lossy(&head_rev.stdout).trim(),
        String::from_utf8_lossy(&tag_rev.stdout).trim()
    );
}

#[test]
fn test_tag_release_skips_commit_if_version_matches() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let cargo_toml_content = r#"[package]
name = "my-test-crate"
version = "0.3.0"
edition = "2021"
"#;
    fs::write(repo_path.join("Cargo.toml"), cargo_toml_content).expect("write Cargo.toml");

    let _ = std::process::Command::new("git")
        .args(["add", "Cargo.toml"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Initial Cargo.toml"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: "v0.3.0",
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Commit message should still be the initial commit, not chore: release v0.3.0
    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "Initial Cargo.toml"
    );
}

#[test]
fn test_update_cargo_toml_version_helper() {
    use git_claw::workflow::update_cargo_toml_version;

    let toml = r#"[package]
name = "demo" # package name
version = "0.1.0" # initial version
edition = "2021"

[dependencies]
foo = { version = "0.1.0" }
"#;

    let updated = update_cargo_toml_version(toml, "0.2.0").expect("should update");
    assert!(updated.contains("version = \"0.2.0\" # initial version"));
    // Dependency version should not be changed
    assert!(updated.contains("foo = { version = \"0.1.0\" }"));

    // When version already matches, returns None
    assert_eq!(update_cargo_toml_version(&updated, "0.2.0"), None);

    // Non-package TOML returns None
    let workspace_toml = "[workspace]\nmembers = [\"a\"]\n";
    assert_eq!(update_cargo_toml_version(workspace_toml, "0.2.0"), None);
}
