use assert_cmd::Command;
use git_claw::core::config::Config;
use git_claw::workflow::{run_init_workflow, InitOptions};
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

    repo_dir
}

#[test]
fn test_init_workflow_scans_env_and_compose() {
    let repo = setup_repo();
    let repo_path = repo.path();

    // 1. Create untracked .env file with port variables
    let env_content = r#"
# App Configuration
DATABASE_URL=postgres://localhost:5432/mydb
APP_PORT=8000
DEBUG_PORT=9000
SECRET_KEY=supersecret
"#;
    fs::write(repo_path.join(".env"), env_content).expect("write .env");

    // 2. Create docker-compose.yml with shared service
    let compose_content = r#"
version: '3.8'

services:
  web:
    build: .
    container_name: my_app_web
    ports:
      - "8000:8000"
  postgres:
    image: postgres:15
    container_name: my_app_postgres

networks:
  default:
    name: tune_network
"#;
    fs::write(repo_path.join("docker-compose.yml"), compose_content).expect("write compose");

    // 3. Run init workflow
    let result = run_init_workflow(InitOptions {
        yes: true,
        repo_root: Some(repo_path),
    });
    assert!(result.is_ok(), "init workflow should succeed");

    let config_path = repo_path.join(".git-claw.toml");
    assert!(config_path.exists(), ".git-claw.toml should be created");

    // 4. Verify generated config can be parsed and has correct values
    let config = Config::load_or_default(&config_path, "test-repo").expect("parse config");

    assert_eq!(config.main_branch(), "main");
    assert_eq!(config.ports.get("APP_PORT"), Some(&8000));
    assert_eq!(config.ports.get("DEBUG_PORT"), Some(&9000));
    assert!(config.files.copy.contains(&".env".to_string()));
    assert_eq!(
        config.docker.compose_file.as_deref(),
        Some("docker-compose.yml")
    );
    assert_eq!(config.docker.network.as_deref(), Some("tune_network"));
    assert!(config
        .docker
        .shared_services
        .contains(&"postgres".to_string()));
}

#[test]
fn test_init_empty_repo() {
    let repo = setup_repo();
    let repo_path = repo.path();

    let result = run_init_workflow(InitOptions {
        yes: true,
        repo_root: Some(repo_path),
    });
    assert!(result.is_ok());

    let config_path = repo_path.join(".git-claw.toml");
    assert!(config_path.exists());

    let config = Config::load_or_default(&config_path, "empty-repo").expect("parse config");
    assert_eq!(config.main_branch(), "main");
    assert!(config.ports.is_empty());
    assert!(config.files.copy.is_empty());
    assert!(!config.docker.is_enabled());
}

#[test]
fn test_init_cli_command_yes() {
    let repo = setup_repo();
    let repo_path = repo.path();

    fs::write(repo_path.join(".env"), "PORT=5000\n").expect("write .env");

    let mut cmd = Command::cargo_bin("git-claw").expect("binary exists");
    cmd.args(["init", "--yes"])
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicates::str::contains("Initialized .git-claw.toml"));

    let config_path = repo_path.join(".git-claw.toml");
    assert!(config_path.exists());

    let config = Config::load_or_default(&config_path, "cli-repo").expect("parse config");
    assert_eq!(config.ports.get("PORT"), Some(&5000));
    assert!(config.files.copy.contains(&".env".to_string()));
}
