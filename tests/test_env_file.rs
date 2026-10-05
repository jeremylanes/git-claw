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

    write_env_worktree(&worktree_path, "my-repo", "auth-v1", 2, &ports)
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
