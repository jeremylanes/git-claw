use git_claw::core::config::{CacheStrategy, Config};
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_default_config_for_absent_file() {
    let non_existent = Path::new("/path/that/does/not/exist/.git-claw.toml");
    let config = Config::load_or_default(non_existent, "git-claw").expect("loads default config");

    assert_eq!(config.main_branch(), "main");
    assert_eq!(config.worktree_root("git-claw"), "../git-claw-worktrees");
    assert!(config.ports.is_empty());
    assert_eq!(config.hooks.post_start, None);
    assert_eq!(config.cache.strategy, CacheStrategy::Shared);
    assert!(config.cache.directories.is_empty());
}

#[test]
fn test_parse_full_valid_toml() {
    let toml_content = r#"
[project]
main_branch = "develop"
worktree_root = "/custom/worktrees"

[ports]
web = 3000
api = 8000

[hooks]
post_start = "make setup"
pre_finish = "cargo test"
post_finish = "echo done"

[cache]
strategy = "isolated"
directories = ["node_modules", "target"]
"#;

    let config = Config::parse_toml_str(toml_content, "git-claw").expect("parses valid toml");

    assert_eq!(config.main_branch(), "develop");
    assert_eq!(config.worktree_root("git-claw"), "/custom/worktrees");
    assert_eq!(config.ports.get("web"), Some(&3000));
    assert_eq!(config.ports.get("api"), Some(&8000));
    assert_eq!(config.hooks.post_start.as_deref(), Some("make setup"));
    assert_eq!(config.hooks.pre_finish.as_deref(), Some("cargo test"));
    assert_eq!(config.hooks.post_finish.as_deref(), Some("echo done"));
    assert_eq!(config.cache.strategy, CacheStrategy::Isolated);
    assert_eq!(config.cache.directories, vec!["node_modules", "target"]);
}

#[test]
fn test_parse_partial_toml() {
    let toml_content = r#"
[ports]
web = 5000
"#;

    let config = Config::parse_toml_str(toml_content, "my-repo").expect("parses partial toml");

    assert_eq!(config.main_branch(), "main");
    assert_eq!(config.worktree_root("my-repo"), "../my-repo-worktrees");
    assert_eq!(config.ports.get("web"), Some(&5000));
    assert_eq!(config.ports.len(), 1);
    assert_eq!(config.cache.strategy, CacheStrategy::Shared);
}

#[test]
fn test_parse_malformed_toml() {
    let invalid_toml = r#"
[project
main_branch = "broken
"#;

    let result = Config::parse_toml_str(invalid_toml, "git-claw");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Failed to parse .git-claw.toml"));
}

#[test]
fn test_load_from_existing_file() {
    let temp_file = NamedTempFile::new().expect("creates temp file");
    let content = r#"
[project]
main_branch = "trunk"
"#;
    std::fs::write(temp_file.path(), content).expect("writes content");

    let config = Config::load_or_default(temp_file.path(), "test-repo").expect("loads file");
    assert_eq!(config.main_branch(), "trunk");
    assert_eq!(config.worktree_root("test-repo"), "../test-repo-worktrees");
}
