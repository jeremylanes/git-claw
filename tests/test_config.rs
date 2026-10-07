use git_claw::core::config::{default_worktree_root, expand_home, CacheStrategy, Config};
use std::path::Path;
use std::sync::Mutex;
use tempfile::NamedTempFile;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_default_config_for_absent_file() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("GIT_CLAW_GLOBAL_CONFIG");

    let non_existent = Path::new("/path/that/does/not/exist/.git-claw.toml");
    let config = Config::load_or_default(non_existent, "git-claw").expect("loads default config");

    assert_eq!(config.main_branch(), "main");
    assert_eq!(
        config.worktree_root("git-claw"),
        default_worktree_root("git-claw")
    );
    assert!(config
        .worktree_root("git-claw")
        .contains(".git-claw/worktrees/git-claw"));
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
    assert_eq!(
        config.worktree_root("my-repo"),
        default_worktree_root("my-repo")
    );
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
    assert_eq!(
        config.worktree_root("test-repo"),
        default_worktree_root("test-repo")
    );
}

#[test]
fn test_global_config_hierarchy_and_override() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let global_file = NamedTempFile::new().expect("creates global temp file");
    let global_content = r#"
[project]
main_branch = "main-global"
worktree_root = "~/global-worktrees/test-repo"

[ports]
web = 8080
db = 5432
"#;
    std::fs::write(global_file.path(), global_content).expect("writes global config");

    std::env::set_var("GIT_CLAW_GLOBAL_CONFIG", global_file.path());

    // 1. When local does not exist, global values are used
    let non_existent = Path::new("/path/that/does/not/exist/.git-claw.toml");
    let config1 = Config::load_or_default(non_existent, "test-repo").expect("loads merged config");
    assert_eq!(config1.main_branch(), "main-global");
    assert!(config1
        .worktree_root("test-repo")
        .contains("global-worktrees/test-repo"));
    assert_eq!(config1.ports.get("web"), Some(&8080));
    assert_eq!(config1.ports.get("db"), Some(&5432));

    // 2. When local exists, local overrides global
    let local_file = NamedTempFile::new().expect("creates local temp file");
    let local_content = r#"
[project]
main_branch = "local-branch"

[ports]
web = 9090
api = 3000
"#;
    std::fs::write(local_file.path(), local_content).expect("writes local config");

    let config2 =
        Config::load_or_default(local_file.path(), "test-repo").expect("loads merged config");
    assert_eq!(config2.main_branch(), "local-branch");
    assert_eq!(config2.ports.get("web"), Some(&9090)); // overridden
    assert_eq!(config2.ports.get("db"), Some(&5432)); // inherited from global
    assert_eq!(config2.ports.get("api"), Some(&3000)); // added from local

    std::env::remove_var("GIT_CLAW_GLOBAL_CONFIG");
}

#[test]
fn test_expand_home() {
    if let Ok(home) = std::env::var("HOME") {
        let expanded = expand_home("~/my/path");
        assert_eq!(expanded, format!("{}/my/path", home));
    }
    assert_eq!(expand_home("/absolute/path"), "/absolute/path");
}
