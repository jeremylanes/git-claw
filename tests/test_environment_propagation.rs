//! Integration test suite for Ticket 18: Automatic Environment Detection and Propagation.
//!
//! Validates automatic detection during `init_workflow` and runtime propagation during
//! `start_worktree` for `pyenv`, `uv`, `venv`, Node.js version files, and clean repository scenarios.

use assert_cmd::Command;
use git_claw::core::branch::BranchType;
use git_claw::core::config::Config;
use git_claw::workflow::{run_init_workflow, start_worktree, InitOptions, StartOptions};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Sets up an isolated temporary Git repository with an initial commit on `main`.
///
/// :returns: A tuple of `(repo_dir, wt_parent)` temporary directories.
///
/// # Examples
///
/// ```ignore
/// let (repo, wt_parent) = setup_repo();
/// assert!(repo.path().join(".git").exists());
/// ```
fn setup_repo() -> (tempfile::TempDir, tempfile::TempDir) {
    let repo_dir = tempdir().expect("create repo tempdir");
    let wt_parent = tempdir().expect("create worktree parent tempdir");
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

    fs::write(repo_path.join("README.md"), "# Test Init").expect("write readme");

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

/// Sets the `worktree_root` parameter in `.git-claw.toml` to a designated test directory.
///
/// :param repo_path: Path to the repository root directory.
/// :param wt_parent: Path to the temporary worktree parent directory.
///
/// # Examples
///
/// ```ignore
/// set_worktree_root(repo.path(), wt_parent.path());
/// ```
fn set_worktree_root(repo_path: &Path, wt_parent: &Path) {
    let config_path = repo_path.join(".git-claw.toml");
    let content = fs::read_to_string(&config_path).expect("read config");
    let updated = content.replace(
        "[project]\nmain_branch = \"main\"",
        &format!(
            "[project]\nmain_branch = \"main\"\nworktree_root = \"{}\"",
            wt_parent.display()
        ),
    );
    fs::write(&config_path, updated).expect("write config");
}

/// Verifies Scenario 1: Pyenv repository with `.python-version` containing `tune`.
///
/// Checks detection in `init_workflow`, propagation into worktree, and `PYENV_VERSION` injection.
///
/// # Examples
///
/// ```ignore
/// test_scenario_1_pyenv_detection_and_propagation();
/// ```
#[test]
fn test_scenario_1_pyenv_detection_and_propagation() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    fs::write(repo_path.join(".python-version"), "tune\n").expect("write .python-version");

    let init_res = run_init_workflow(InitOptions {
        yes: true,
        repo_root: Some(repo_path),
    });
    assert!(init_res.is_ok(), "init_workflow should succeed");

    let config_path = repo_path.join(".git-claw.toml");
    let raw = fs::read_to_string(&config_path).expect("read .git-claw.toml");
    assert!(raw.contains("[environment]"));
    assert!(raw.contains("type = \"pyenv\""));
    assert!(raw.contains("name = \"tune\""));
    assert!(raw.contains("copy_files = [\".python-version\"]"));

    let config = Config::load_or_default(&config_path, "test-repo").expect("parse config");
    assert_eq!(config.environment.r#type.as_deref(), Some("pyenv"));
    assert_eq!(config.environment.name.as_deref(), Some("tune"));
    assert_eq!(
        config.environment.copy_files,
        vec![".python-version".to_string()]
    );

    set_worktree_root(repo_path, wt_parent.path());

    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "pyenv-feat",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start_worktree succeeds");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("pyenv-feat");
    assert!(wt_path.exists());

    let py_ver = wt_path.join(".python-version");
    assert!(py_ver.exists());
    assert_eq!(fs::read_to_string(&py_ver).unwrap(), "tune\n");

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("read env");
    assert!(env_content.contains("PYENV_VERSION=tune"));
    assert!(env_content.contains("CLAW_SLOT_ID=1"));
    assert!(env_content.contains("CLAW_WORKTREE_NAME=pyenv-feat"));
}

/// Verifies Scenario 2: uv / virtualenv repository with `uv.lock` and `.venv` directory.
///
/// Checks detection in `init_workflow`, `.venv` symlinking into worktree, and `VIRTUAL_ENV` / `PATH` injection.
///
/// # Examples
///
/// ```ignore
/// test_scenario_2_uv_virtualenv_detection_and_symlink();
/// ```
#[test]
fn test_scenario_2_uv_virtualenv_detection_and_symlink() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    fs::write(repo_path.join("uv.lock"), "# uv lockfile\n").expect("write uv.lock");
    let primary_venv = repo_path.join(".venv");
    fs::create_dir_all(&primary_venv).expect("create .venv");
    fs::write(primary_venv.join("pyvenv.cfg"), "home = /usr/bin\n").expect("write pyvenv.cfg");

    let init_res = run_init_workflow(InitOptions {
        yes: true,
        repo_root: Some(repo_path),
    });
    assert!(init_res.is_ok(), "init_workflow should succeed");

    let config_path = repo_path.join(".git-claw.toml");
    let raw = fs::read_to_string(&config_path).expect("read .git-claw.toml");
    assert!(raw.contains("type = \"uv\""));
    assert!(raw.contains("[cache]"));
    assert!(raw.contains("\".venv\""));

    let config = Config::load_or_default(&config_path, "test-repo").expect("parse config");
    assert_eq!(config.environment.r#type.as_deref(), Some("uv"));
    assert!(config.cache.directories.contains(&".venv".to_string()));

    set_worktree_root(repo_path, wt_parent.path());

    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "uv-feat",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start_worktree succeeds");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("uv-feat");
    let wt_venv = wt_path.join(".venv");
    assert!(wt_venv.exists());
    let symlink_meta = fs::symlink_metadata(&wt_venv).expect("symlink metadata");
    assert!(symlink_meta.file_type().is_symlink());

    let target = fs::read_link(&wt_venv).expect("read symlink");
    assert_eq!(
        fs::canonicalize(target).unwrap(),
        fs::canonicalize(&primary_venv).unwrap()
    );
    assert!(wt_venv.join("pyvenv.cfg").exists());

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("read env");
    assert!(env_content.contains(&format!("VIRTUAL_ENV={}", wt_venv.display())));
    assert!(env_content.contains(&format!("PATH={}/bin:$PATH", wt_venv.display())));
}

/// Verifies Scenario 3: Explicit configuration without `.python-version` in the primary repository.
///
/// Checks that `start_worktree` generates `.python-version` with the configured environment name.
///
/// # Examples
///
/// ```ignore
/// test_scenario_3_explicit_config_generates_python_version();
/// ```
#[test]
fn test_scenario_3_explicit_config_generates_python_version() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    assert!(!repo_path.join(".python-version").exists());

    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[environment]
type = "pyenv"
name = "my-env"
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");

    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Bugfix,
        name: "fix-pyenv",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start_worktree succeeds");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("fix-pyenv");
    let py_ver = wt_path.join(".python-version");
    assert!(py_ver.exists());
    assert_eq!(fs::read_to_string(&py_ver).unwrap(), "my-env\n");

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("read env");
    assert!(env_content.contains("PYENV_VERSION=my-env"));
    assert!(!repo_path.join(".python-version").exists());
}

/// Verifies Scenario 4: Node.js version declarations (`.nvmrc` and `.node-version`).
///
/// Checks that both files are detected by `init_workflow` and propagated into the worktree.
///
/// # Examples
///
/// ```ignore
/// test_scenario_4_nodejs_version_files_propagation();
/// ```
#[test]
fn test_scenario_4_nodejs_version_files_propagation() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    fs::write(repo_path.join(".nvmrc"), "v20.11.0\n").expect("write .nvmrc");
    fs::write(repo_path.join(".node-version"), "20.11.0\n").expect("write .node-version");

    let init_res = run_init_workflow(InitOptions {
        yes: true,
        repo_root: Some(repo_path),
    });
    assert!(init_res.is_ok(), "init_workflow succeeds");

    let config_path = repo_path.join(".git-claw.toml");
    let raw = fs::read_to_string(&config_path).expect("read config");
    assert!(raw.contains("\".node-version\""));
    assert!(raw.contains("\".nvmrc\""));

    set_worktree_root(repo_path, wt_parent.path());

    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "node-feat",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start_worktree succeeds");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("node-feat");
    let wt_nvmrc = wt_path.join(".nvmrc");
    assert!(wt_nvmrc.exists());
    assert_eq!(fs::read_to_string(&wt_nvmrc).unwrap(), "v20.11.0\n");

    let wt_node = wt_path.join(".node-version");
    assert!(wt_node.exists());
    assert_eq!(fs::read_to_string(&wt_node).unwrap(), "20.11.0\n");

    assert!(!wt_path.join(".python-version").exists());
}

/// Verifies Scenario 5: Clean repository without any environment or version files.
///
/// Checks that worktree creation succeeds cleanly without warnings, no `.python-version` is created,
/// and `.env.worktree` contains only standard variables.
///
/// # Examples
///
/// ```ignore
/// test_scenario_5_clean_repo_no_environment();
/// ```
#[test]
fn test_scenario_5_clean_repo_no_environment() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let init_res = run_init_workflow(InitOptions {
        yes: true,
        repo_root: Some(repo_path),
    });
    assert!(init_res.is_ok(), "init_workflow succeeds");

    let config_path = repo_path.join(".git-claw.toml");
    let raw = fs::read_to_string(&config_path).expect("read config");
    assert!(!raw.contains("[environment]"));
    assert!(!raw.contains("[cache]"));

    set_worktree_root(repo_path, wt_parent.path());

    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "clean-feat",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start_worktree succeeds");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("clean-feat");
    assert!(wt_path.exists());

    assert!(!wt_path.join(".python-version").exists());
    assert!(!wt_path.join(".node-version").exists());
    assert!(!wt_path.join(".nvmrc").exists());
    assert!(!wt_path.join(".venv").exists());

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("read env");
    assert!(env_content.contains("CLAW_SLOT_ID=1"));
    assert!(env_content.contains("CLAW_WORKTREE_NAME=clean-feat"));
    assert!(env_content.contains("COMPOSE_PROJECT_NAME="));
    assert!(!env_content.contains("PYENV_VERSION="));
    assert!(!env_content.contains("VIRTUAL_ENV="));
}

/// Verifies isolated mode does not symlink `.venv` even if present in the primary repo.
///
/// :param repo_path: Path to the repository root directory.
///
/// # Examples
///
/// ```ignore
/// test_scenario_isolated_venv_skips_symlink();
/// ```
#[test]
fn test_scenario_isolated_venv_skips_symlink() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    let primary_venv = repo_path.join(".venv");
    fs::create_dir_all(&primary_venv).expect("create .venv");

    let config_content = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[cache]
directories = [".venv"]
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), config_content).expect("write config");

    let slot = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "isolated-feat",
        isolated: true,
        repo_root: Some(repo_path),
    })
    .expect("start_worktree succeeds");
    assert_eq!(slot, 1);

    let wt_path = wt_parent.path().join("isolated-feat");
    assert!(!wt_path.join(".venv").exists());

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("read env");
    assert!(!env_content.contains("VIRTUAL_ENV="));
}

/// Verifies end-to-end CLI flow for init and feature start with environment propagation.
///
/// # Examples
///
/// ```ignore
/// test_cli_environment_propagation_e2e();
/// ```
#[test]
fn test_cli_environment_propagation_e2e() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    fs::write(repo_path.join(".python-version"), "3.12.2\n").expect("write .python-version");

    let mut init_cmd = Command::cargo_bin("git-claw").expect("binary exists");
    init_cmd
        .current_dir(repo_path)
        .args(["init", "--yes"])
        .assert()
        .success();

    set_worktree_root(repo_path, wt_parent.path());

    let mut start_cmd = Command::cargo_bin("git-claw").expect("binary exists");
    start_cmd
        .current_dir(repo_path)
        .args(["feature", "start", "cli-env"])
        .assert()
        .success();

    let wt_path = wt_parent.path().join("cli-env");
    let wt_py_ver = wt_path.join(".python-version");
    assert!(wt_py_ver.exists());
    assert_eq!(fs::read_to_string(&wt_py_ver).unwrap(), "3.12.2\n");

    let env_content = fs::read_to_string(wt_path.join(".env.worktree")).expect("read env");
    assert!(env_content.contains("PYENV_VERSION=3.12.2"));
}
