use git_claw::core::config::EnvironmentConfig;
use git_claw::infra::env_file::write_env_worktree;
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_write_env_worktree_all_variables() {
    let dir = tempdir().expect("tempdir");
    let worktree_path = dir.path().join("worktree-auth");
    fs::create_dir_all(&worktree_path).expect("creates dir");

    let mut ports = BTreeMap::new();
    ports.insert("PORT_WEB".to_string(), 3002);
    ports.insert("PORT_API".to_string(), 8002);

    write_env_worktree(&worktree_path, "my-repo", "auth-v1", 2, &ports, None)
        .expect("writes env worktree");

    let env_file = worktree_path.join(".env.worktree");
    assert!(env_file.exists());

    let content = fs::read_to_string(&env_file).expect("reads env file");
    assert!(content.contains("CLAW_SLOT_ID=2"));
    assert!(content.contains("CLAW_WORKTREE_NAME=auth-v1"));
    assert!(content.contains(&format!("CLAW_WORKTREE_PATH={}", worktree_path.display())));
    assert!(content.contains("COMPOSE_PROJECT_NAME=my_repo_auth_v1_2"));
    assert!(content.contains("PORT_WEB=3002"));
    assert!(content.contains("PORT_API=8002"));
}

#[test]
fn test_write_env_worktree_pyenv_version() {
    let dir = tempdir().expect("tempdir");
    let worktree_path = dir.path().join("worktree-pyenv");
    fs::create_dir_all(&worktree_path).expect("creates dir");

    let env_config = EnvironmentConfig {
        r#type: Some("pyenv".to_string()),
        name: Some("tune".to_string()),
        copy_files: vec![".python-version".to_string()],
    };

    write_env_worktree(
        &worktree_path,
        "my-repo",
        "pyenv-wt",
        1,
        &BTreeMap::new(),
        Some(&env_config),
    )
    .expect("writes env worktree");

    let env_file = worktree_path.join(".env.worktree");
    let content = fs::read_to_string(&env_file).expect("reads env file");
    assert!(content.contains("PYENV_VERSION=tune"));
    assert!(!content.contains("VIRTUAL_ENV="));
}

#[test]
fn test_write_env_worktree_venv_present() {
    let dir = tempdir().expect("tempdir");
    let worktree_path = dir.path().join("worktree-venv");
    fs::create_dir_all(worktree_path.join(".venv")).expect("creates venv dir");

    write_env_worktree(
        &worktree_path,
        "my-repo",
        "venv-wt",
        3,
        &BTreeMap::new(),
        None,
    )
    .expect("writes env worktree");

    let env_file = worktree_path.join(".env.worktree");
    let content = fs::read_to_string(&env_file).expect("reads env file");
    let venv_path = worktree_path.join(".venv");
    assert!(content.contains(&format!("VIRTUAL_ENV={}", venv_path.display())));
    assert!(content.contains(&format!("PATH={}/bin:$PATH", venv_path.display())));
}
