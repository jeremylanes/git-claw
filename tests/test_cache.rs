use git_claw::core::config::{CacheConfig, CacheStrategy};
use git_claw::infra::fs::link_shared_cache_directories;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_link_shared_cache_directories() {
    let repo_dir = tempdir().expect("repo tempdir");
    let worktree_dir = tempdir().expect("worktree tempdir");

    let primary_node_modules = repo_dir.path().join("node_modules");
    fs::create_dir_all(&primary_node_modules).expect("creates primary dir");
    fs::write(primary_node_modules.join("package.json"), "{}").expect("writes file");

    let cache_config = CacheConfig {
        strategy: CacheStrategy::Shared,
        directories: vec!["node_modules".to_string(), "target".to_string()],
    };

    let linked =
        link_shared_cache_directories(repo_dir.path(), worktree_dir.path(), &cache_config, false)
            .expect("links cache");

    assert_eq!(linked, 2);

    let wt_node_modules = worktree_dir.path().join("node_modules");
    assert!(wt_node_modules.is_symlink());
    assert!(wt_node_modules.join("package.json").exists());

    let wt_target = worktree_dir.path().join("target");
    assert!(wt_target.is_symlink());
}

#[test]
fn test_skip_cache_when_isolated_flag() {
    let repo_dir = tempdir().expect("repo tempdir");
    let worktree_dir = tempdir().expect("worktree tempdir");

    let cache_config = CacheConfig {
        strategy: CacheStrategy::Shared,
        directories: vec!["node_modules".to_string()],
    };

    let linked =
        link_shared_cache_directories(repo_dir.path(), worktree_dir.path(), &cache_config, true)
            .expect("links cache");

    assert_eq!(linked, 0);
    assert!(!worktree_dir.path().join("node_modules").exists());
}

#[test]
fn test_skip_cache_when_strategy_isolated() {
    let repo_dir = tempdir().expect("repo tempdir");
    let worktree_dir = tempdir().expect("worktree tempdir");

    let cache_config = CacheConfig {
        strategy: CacheStrategy::Isolated,
        directories: vec!["node_modules".to_string()],
    };

    let linked =
        link_shared_cache_directories(repo_dir.path(), worktree_dir.path(), &cache_config, false)
            .expect("links cache");

    assert_eq!(linked, 0);
    assert!(!worktree_dir.path().join("node_modules").exists());
}
