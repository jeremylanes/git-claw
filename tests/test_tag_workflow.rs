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
        version: Some("v1.0.0"),
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
        version: Some("v1.0.1"),
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
}

#[test]
fn test_tag_release_invalid_semver_fails() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let res = tag_release(TagOptions {
        version: Some("invalid.version"),
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
        version: Some("v0.3.0"),
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
        version: Some("v0.3.0"),
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

#[test]
fn test_update_pyproject_toml_version_helper() {
    use git_claw::workflow::update_pyproject_toml_version;

    // Test PEP 621 [project] table
    let pep621 = r#"[project]
name = "my-app" # project name
version = "0.1.0" # version tag
description = "Sample app"

[project.scripts]
run = "my_app:run"
"#;
    let updated_pep621 = update_pyproject_toml_version(pep621, "0.2.0").expect("should update");
    assert!(updated_pep621.contains("version = \"0.2.0\" # version tag"));
    assert_eq!(
        update_pyproject_toml_version(&updated_pep621, "0.2.0"),
        None
    );

    // Test Poetry [tool.poetry] table
    let poetry = r#"[tool.poetry]
name = "my-poetry-app"
version = "1.0.0"

[tool.poetry.dependencies]
python = "^3.11"
requests = { version = "2.31.0" }
"#;
    let updated_poetry = update_pyproject_toml_version(poetry, "1.1.0").expect("should update");
    assert!(updated_poetry.contains("version = \"1.1.0\""));
    // Dependency version should remain untouched
    assert!(updated_poetry.contains("requests = { version = \"2.31.0\" }"));

    // Test single quotes
    let single_quotes = "[project]\nversion = '0.5.0'\n";
    let updated_single =
        update_pyproject_toml_version(single_quotes, "0.6.0").expect("should update");
    assert_eq!(updated_single, "[project]\nversion = '0.6.0'\n");

    // Non-project TOML returns None
    let other_toml = "[build-system]\nrequires = [\"hatchling\"]\n";
    assert_eq!(update_pyproject_toml_version(other_toml, "0.2.0"), None);
}

#[test]
fn test_tag_release_updates_and_commits_pyproject_toml() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let pyproject_content = r#"[project]
name = "sample-python"
version = "0.1.0"
description = "A sample python project"
"#;
    fs::write(repo_path.join("pyproject.toml"), pyproject_content).expect("write pyproject.toml");

    let _ = std::process::Command::new("git")
        .args(["add", "pyproject.toml"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add pyproject.toml"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: Some("v0.2.0"),
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Confirms pyproject.toml now contains version = "0.2.0"
    let updated_pyproject =
        fs::read_to_string(repo_path.join("pyproject.toml")).expect("read pyproject.toml");
    assert!(updated_pyproject.contains("version = \"0.2.0\""));

    // HEAD commit message is chore: release v0.2.0
    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "chore: release v0.2.0"
    );

    // Tag v0.2.0 points to HEAD
    let head_rev = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .expect("rev-parse HEAD");
    let tag_rev = std::process::Command::new("git")
        .args(["rev-parse", "v0.2.0^{commit}"])
        .current_dir(repo_path)
        .output()
        .expect("rev-parse tag");
    assert_eq!(
        String::from_utf8_lossy(&head_rev.stdout).trim(),
        String::from_utf8_lossy(&tag_rev.stdout).trim()
    );
}

#[test]
fn test_tag_release_updates_and_commits_project_toml() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let project_content = r#"[project]
name = "sample-legacy"
version = "1.0.0"
"#;
    fs::write(repo_path.join("project.toml"), project_content).expect("write project.toml");

    let _ = std::process::Command::new("git")
        .args(["add", "project.toml"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add project.toml"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: Some("1.1.0"),
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Confirms project.toml now contains version = "1.1.0"
    let updated_project =
        fs::read_to_string(repo_path.join("project.toml")).expect("read project.toml");
    assert!(updated_project.contains("version = \"1.1.0\""));

    // HEAD commit message is chore: release 1.1.0
    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "chore: release 1.1.0"
    );
}

#[test]
fn test_tag_release_with_pyproject_and_uv_lock() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let pyproject_content = r#"[project]
name = "uv-project"
version = "0.1.0"
"#;
    let uv_lock_content = "# uv lockfile v1\nversion = 1\n";
    fs::write(repo_path.join("pyproject.toml"), pyproject_content).expect("write pyproject.toml");
    fs::write(repo_path.join("uv.lock"), uv_lock_content).expect("write uv.lock");

    let _ = std::process::Command::new("git")
        .args(["add", "pyproject.toml", "uv.lock"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Initial python with uv.lock"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: Some("v0.2.0"),
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    let updated_pyproject =
        fs::read_to_string(repo_path.join("pyproject.toml")).expect("read pyproject.toml");
    assert!(updated_pyproject.contains("version = \"0.2.0\""));

    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "chore: release v0.2.0"
    );
}

#[test]
fn test_tag_release_skips_commit_if_pyproject_version_matches() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let pyproject_content = r#"[project]
name = "already-released"
version = "0.2.0"
"#;
    fs::write(repo_path.join("pyproject.toml"), pyproject_content).expect("write pyproject.toml");

    let _ = std::process::Command::new("git")
        .args(["add", "pyproject.toml"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Initial release v0.2.0"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: Some("v0.2.0"),
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Commit message should remain initial commit
    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "Initial release v0.2.0"
    );

    // Tag was still created
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", "v0.2.0"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert_eq!(String::from_utf8_lossy(&tag_check.stdout).trim(), "v0.2.0");
}

#[test]
fn test_tag_release_auto_infer_minor_and_changelog() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    // Create previous tag v0.2.0
    let _ = std::process::Command::new("git")
        .args(["tag", "-a", "v0.2.0", "-m", "Release v0.2.0"])
        .current_dir(repo_path)
        .output();

    // Add a feature commit
    fs::write(repo_path.join("feature.txt"), "new feature").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "feature.txt"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args([
            "commit",
            "-m",
            "feat(core): add conventional commit support",
        ])
        .current_dir(repo_path)
        .output();

    // Tag without version -> auto inferred
    tag_release(TagOptions {
        version: None,
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Inferred minor bump: v0.2.0 -> v0.3.0
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", "v0.3.0"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert_eq!(String::from_utf8_lossy(&tag_check.stdout).trim(), "v0.3.0");

    // Verify CHANGELOG.md was created
    let changelog = fs::read_to_string(repo_path.join("CHANGELOG.md")).expect("read changelog");
    assert!(changelog.contains("## [v0.3.0] - "));
    assert!(changelog.contains("### Features"));
    assert!(changelog.contains("- **core**: add conventional commit support"));

    // Verify HEAD commit message
    let log_msg = std::process::Command::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo_path)
        .output()
        .expect("git log");
    assert_eq!(
        String::from_utf8_lossy(&log_msg.stdout).trim(),
        "chore: release v0.3.0"
    );
}

#[test]
fn test_tag_release_auto_infer_major_breaking() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let _ = std::process::Command::new("git")
        .args(["tag", "-a", "v0.2.0", "-m", "Release v0.2.0"])
        .current_dir(repo_path)
        .output();

    // Add a breaking change commit with !
    fs::write(repo_path.join("breaking.txt"), "break").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "breaking.txt"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "fix!: drop deprecated API"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: None,
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Major bump: v0.2.0 -> v1.0.0
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", "v1.0.0"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert_eq!(String::from_utf8_lossy(&tag_check.stdout).trim(), "v1.0.0");

    let changelog = fs::read_to_string(repo_path.join("CHANGELOG.md")).expect("read changelog");
    assert!(changelog.contains("## [v1.0.0] - "));
    assert!(changelog.contains("### Breaking Changes"));
    assert!(changelog.contains("- drop deprecated API"));
}

#[test]
fn test_tag_release_auto_infer_patch() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let _ = std::process::Command::new("git")
        .args(["tag", "-a", "v0.2.0", "-m", "Release v0.2.0"])
        .current_dir(repo_path)
        .output();

    fs::write(repo_path.join("fix.txt"), "fix").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "fix.txt"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "fix: resolve memory leak"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: None,
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Patch bump: v0.2.0 -> v0.2.1
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", "v0.2.1"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert_eq!(String::from_utf8_lossy(&tag_check.stdout).trim(), "v0.2.1");

    let changelog = fs::read_to_string(repo_path.join("CHANGELOG.md")).expect("read changelog");
    assert!(changelog.contains("## [v0.2.1] - "));
    assert!(changelog.contains("### Bug Fixes"));
    assert!(changelog.contains("- resolve memory leak"));
}

#[test]
fn test_tag_release_auto_infer_no_previous_tags_fresh_repo() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let cargo_toml_content = r#"[package]
name = "fresh-project"
version = "0.1.0"
edition = "2021"
"#;
    fs::write(repo_path.join("Cargo.toml"), cargo_toml_content).expect("write Cargo.toml");
    let _ = std::process::Command::new("git")
        .args(["add", "Cargo.toml"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "feat: initial feature set"])
        .current_dir(repo_path)
        .output();

    tag_release(TagOptions {
        version: None,
        repo_root: Some(repo_path),
    })
    .expect("tag release");

    // Fresh repo base 0.1.0 with feat -> bumped minor -> v0.2.0
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", "v0.2.0"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert_eq!(String::from_utf8_lossy(&tag_check.stdout).trim(), "v0.2.0");

    let updated_cargo_toml =
        fs::read_to_string(repo_path.join("Cargo.toml")).expect("read Cargo.toml");
    assert!(updated_cargo_toml.contains("version = \"0.2.0\""));
}

#[test]
fn test_tag_release_auto_infer_no_commits_fails() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let _ = std::process::Command::new("git")
        .args(["tag", "-a", "v0.1.0", "-m", "Release v0.1.0"])
        .current_dir(repo_path)
        .output();

    // No new commits since v0.1.0
    let res = tag_release(TagOptions {
        version: None,
        repo_root: Some(repo_path),
    });
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("No commits found since last release tag"));
}

#[test]
fn test_cli_tag_command_auto_e2e() {
    let repo_dir = setup_repo();
    let repo_path = repo_dir.path();

    let _ = std::process::Command::new("git")
        .args(["tag", "-a", "v0.2.0", "-m", "Release v0.2.0"])
        .current_dir(repo_path)
        .output();

    fs::write(repo_path.join("code.txt"), "feature").expect("write file");
    let _ = std::process::Command::new("git")
        .args(["add", "code.txt"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "feat(api): expose health check"])
        .current_dir(repo_path)
        .output();

    let mut cmd = Command::cargo_bin("git-claw").expect("binary");
    cmd.current_dir(repo_path)
        .arg("tag")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Inferred bump: Minor -> version 'v0.3.0'",
        ))
        .stdout(predicate::str::contains("Updated CHANGELOG.md"))
        .stdout(predicate::str::contains(
            "Created annotated release tag 'v0.3.0'",
        ));

    // Verify tag exists
    let check = std::process::Command::new("git")
        .args(["tag", "-l", "v0.3.0"])
        .current_dir(repo_path)
        .output()
        .expect("check tag");
    assert_eq!(String::from_utf8_lossy(&check.stdout).trim(), "v0.3.0");
}
