use git_claw::core::branch::BranchType;
use git_claw::core::config::DockerConfig;
use git_claw::infra::docker::{
    generate_docker_compose_override, generate_override_content, parse_compose_services,
    ParsedService,
};
use git_claw::workflow::{start_worktree, StartOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_compose_services() {
    let yaml = r#"
version: '3.8'

services:
  web:
    build: .
    container_name: "tune_web"
    ports:
      - "80:80"
  worker:
    image: python:3.11
  postgres:
    image: postgres:15
    container_name: tune_postgres

networks:
  default:
    name: tune_network
"#;

    let services = parse_compose_services(yaml);
    assert_eq!(services.len(), 3);

    assert_eq!(services[0].name, "web");
    assert_eq!(services[0].container_name.as_deref(), Some("tune_web"));

    assert_eq!(services[1].name, "worker");
    assert_eq!(services[1].container_name, None);

    assert_eq!(services[2].name, "postgres");
    assert_eq!(services[2].container_name.as_deref(), Some("tune_postgres"));
}

#[test]
fn test_generate_override_content_syntax() {
    let config = DockerConfig {
        compose_file: None,
        network: Some("tune_network".to_string()),
        shared_services: vec!["postgres".to_string()],
    };

    let services = vec![
        ParsedService {
            name: "web".to_string(),
            container_name: Some("tune_web".to_string()),
        },
        ParsedService {
            name: "worker".to_string(),
            container_name: None,
        },
        ParsedService {
            name: "postgres".to_string(),
            container_name: Some("tune_postgres".to_string()),
        },
    ];

    let content = generate_override_content(&services, &config);

    // Verifies container_name neutralization with !reset
    assert!(content.contains("container_name: !reset"));

    // Verifies shared service disabled & scale 0 without breaking depends_on
    assert!(content.contains("postgres:"));
    assert!(!content.contains("claw-disabled"));
    assert!(content.contains("scale: 0"));
    assert!(content.contains("restart: \"no\""));
    assert!(content.contains("entrypoint: [\"true\"]"));

    // Verifies external network attachment
    assert!(content.contains("networks:\n  tune_network:\n    external: true"));
    assert!(content.contains("networks:\n      - tune_network"));
}

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

    fs::write(repo_path.join("README.md"), "# Test Docker Override").expect("write readme");

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
fn test_workflow_generates_docker_compose_override() {
    let (repo_dir, wt_parent) = setup_repo();
    let repo_path = repo_dir.path();

    // 1. Create docker-compose.yml in primary repository
    let compose_content = r#"
version: '3.8'
services:
  web:
    image: tune-web:latest
    container_name: tune_web
    depends_on:
      - postgres
    ports:
      - "8000:8000"
  postgres:
    image: postgres:15
    container_name: tune_postgres
networks:
  default:
    name: tune_shared_network
"#;
    fs::write(repo_path.join("docker-compose.yml"), compose_content).expect("write compose");
    let _ = std::process::Command::new("git")
        .args(["add", "docker-compose.yml"])
        .current_dir(repo_path)
        .output();
    let _ = std::process::Command::new("git")
        .args(["commit", "-m", "Add compose file"])
        .current_dir(repo_path)
        .output();

    // 2. Configure .git-claw.toml with [docker]
    let claw_toml = format!(
        r#"
[project]
main_branch = "main"
worktree_root = "{}"

[docker]
network = "tune_shared_network"
shared_services = ["postgres"]
"#,
        wt_parent.path().display()
    );
    fs::write(repo_path.join(".git-claw.toml"), claw_toml).expect("write config");

    // 3. Start a worktree
    let slot_id = start_worktree(StartOptions {
        branch_type: BranchType::Feature,
        name: "docker-interop",
        isolated: false,
        repo_root: Some(repo_path),
    })
    .expect("start worktree");
    assert_eq!(slot_id, 1);

    let wt_path = wt_parent.path().join("docker-interop");
    let override_path = wt_path.join("docker-compose.override.yml");
    assert!(override_path.exists(), "override file should exist");

    let override_content = fs::read_to_string(&override_path).expect("read override");

    // Check neutralization and service sharing
    assert!(override_content.contains("web:"));
    assert!(override_content.contains("container_name: !reset"));
    assert!(override_content.contains("postgres:"));
    assert!(!override_content.contains("claw-disabled"));
    assert!(override_content.contains("scale: 0"));

    // Check network attachment
    assert!(override_content.contains("tune_shared_network:"));
    assert!(override_content.contains("external: true"));

    // Base compose file remains unchanged
    let original = fs::read_to_string(repo_path.join("docker-compose.yml")).expect("read original");
    assert_eq!(original, compose_content);

    // Validate with docker compose config if docker command exists
    let compose_cmd = std::process::Command::new("docker")
        .args(["compose", "config"])
        .current_dir(&wt_path)
        .output();

    if let Ok(output) = compose_cmd {
        assert!(
            output.status.success(),
            "docker compose config failed: stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_docker_disabled_when_empty() {
    let dir = tempdir().expect("tempdir");
    let config = DockerConfig::default();
    let result =
        generate_docker_compose_override(dir.path(), dir.path(), &config).expect("generate");
    assert!(result.is_none());
    assert!(!dir.path().join("docker-compose.override.yml").exists());
}
