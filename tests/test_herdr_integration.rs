use git_claw::infra::herdr::{is_herdr_active, open_worktree_in_herdr};
use std::path::Path;

#[test]
fn test_is_herdr_active_when_env_unset() {
    let orig = std::env::var("HERDR_ENV").ok();
    std::env::remove_var("HERDR_ENV");

    assert!(!is_herdr_active());

    if let Some(val) = orig {
        std::env::set_var("HERDR_ENV", val);
    }
}

#[test]
fn test_open_worktree_in_herdr_rejects_tmp_paths_even_if_active() {
    let orig = std::env::var("HERDR_ENV").ok();
    std::env::set_var("HERDR_ENV", "1");

    // Even if HERDR_ENV is 1, temporary directory paths must never trigger herdr
    assert!(!open_worktree_in_herdr(
        Path::new("/tmp/test_project"),
        "feature/abc"
    ));
    assert!(!open_worktree_in_herdr(
        &std::env::temp_dir().join("some_repo"),
        "feature/abc"
    ));

    match orig {
        Some(val) => std::env::set_var("HERDR_ENV", val),
        None => std::env::remove_var("HERDR_ENV"),
    }
}

#[test]
fn test_open_worktree_in_herdr_when_disabled_via_flag() {
    let orig_herdr = std::env::var("HERDR_ENV").ok();
    let orig_disable = std::env::var("CLAW_DISABLE_HERDR").ok();

    std::env::set_var("HERDR_ENV", "1");
    std::env::set_var("CLAW_DISABLE_HERDR", "1");

    assert!(!is_herdr_active());
    assert!(!open_worktree_in_herdr(
        Path::new("/home/user/project"),
        "feature/test"
    ));

    match orig_disable {
        Some(val) => std::env::set_var("CLAW_DISABLE_HERDR", val),
        None => std::env::remove_var("CLAW_DISABLE_HERDR"),
    }
    match orig_herdr {
        Some(val) => std::env::set_var("HERDR_ENV", val),
        None => std::env::remove_var("HERDR_ENV"),
    }
}

#[test]
fn test_herdr_inactive_during_cargo_test() {
    // Under cargo test, CARGO_MANIFEST_DIR is set, protecting test suites
    let orig_herdr = std::env::var("HERDR_ENV").ok();
    std::env::set_var("HERDR_ENV", "1");

    assert!(std::env::var("CARGO_MANIFEST_DIR").is_ok());
    assert!(!is_herdr_active());

    match orig_herdr {
        Some(val) => std::env::set_var("HERDR_ENV", val),
        None => std::env::remove_var("HERDR_ENV"),
    }
}
