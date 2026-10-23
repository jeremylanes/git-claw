//! Test suite for multi-worktree test database isolation per slot.
//!
//! Validates deterministic test database naming (`test_{db}_slot_{slot_id}`), environment
//! injection in `.env.worktree`, and in-place rewriting of copied project `.env` files.
//!
//! # Examples
//!
//! ```
//! use git_claw::infra::env_file::extract_base_db;
//!
//! assert_eq!(extract_base_db("test_tune_slot_1", Some("tune")), "tune");
//! assert_eq!(extract_base_db("test_tune_1", Some("tune")), "tune");
//! ```

use git_claw::infra::env_file::{
    copy_and_merge_untracked_files, extract_base_db, update_env_content_with_ports,
    write_env_worktree,
};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

/// Verifies that `.env.worktree` generates isolated `TEST_DATABASE_NAME` and `CLAW_TEST_DB` per slot.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_write_env_worktree_slot_isolation() {
    let dir = tempdir().expect("tempdir");
    let wt1 = dir.path().join("slot-1");
    let wt2 = dir.path().join("slot-2");
    fs::create_dir_all(&wt1).expect("create wt1");
    fs::create_dir_all(&wt2).expect("create wt2");

    write_env_worktree(&wt1, "tune-app", "feature-a", 1, &BTreeMap::new(), None)
        .expect("write env slot 1");
    write_env_worktree(&wt2, "tune-app", "feature-b", 2, &BTreeMap::new(), None)
        .expect("write env slot 2");

    let content1 = fs::read_to_string(wt1.join(".env.worktree")).expect("read slot 1");
    let content2 = fs::read_to_string(wt2.join(".env.worktree")).expect("read slot 2");

    assert!(content1.contains("TEST_DATABASE_NAME=test_tune_app_slot_1"));
    assert!(content1.contains("CLAW_TEST_DB=test_tune_app_slot_1"));

    assert!(content2.contains("TEST_DATABASE_NAME=test_tune_app_slot_2"));
    assert!(content2.contains("CLAW_TEST_DB=test_tune_app_slot_2"));
}

/// Verifies concurrent slot isolation across Slot 1 and Slot 2 with both `.env.worktree`
/// and copied project `.env` files.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_concurrent_multi_slot_isolation() {
    let root_dir = tempdir().expect("root_dir");
    let wt_parent = tempdir().expect("wt_parent");
    let repo_root = root_dir.path();

    let primary_env = "DATABASE_NAME=production_tune\nAPI_PORT=8000\n";
    fs::write(repo_root.join("local.env"), primary_env).expect("write local.env");

    let wt1 = wt_parent.path().join("worktree-slot-1");
    let wt2 = wt_parent.path().join("worktree-slot-2");
    fs::create_dir_all(&wt1).expect("create wt1");
    fs::create_dir_all(&wt2).expect("create wt2");

    let mut ports = BTreeMap::new();
    ports.insert("API_PORT".to_string(), 8000);

    // Slot 1 setup
    write_env_worktree(&wt1, "tune", "feat-1", 1, &ports, None).expect("write env wt1");
    copy_and_merge_untracked_files(
        repo_root,
        &wt1,
        &["local.env".to_string()],
        &ports,
        1,
        Some("tune_feat_1_1"),
    )
    .expect("copy wt1");

    // Slot 2 setup
    write_env_worktree(&wt2, "tune", "feat-2", 2, &ports, None).expect("write env wt2");
    copy_and_merge_untracked_files(
        repo_root,
        &wt2,
        &["local.env".to_string()],
        &ports,
        2,
        Some("tune_feat_2_2"),
    )
    .expect("copy wt2");

    // Verify Slot 1
    let wt1_worktree_env =
        fs::read_to_string(wt1.join(".env.worktree")).expect("read wt1 worktree");
    let wt1_local_env = fs::read_to_string(wt1.join("local.env")).expect("read wt1 local");
    assert!(wt1_worktree_env.contains("TEST_DATABASE_NAME=test_tune_slot_1"));
    assert!(wt1_worktree_env.contains("CLAW_TEST_DB=test_tune_slot_1"));
    assert!(wt1_local_env.contains("TEST_DATABASE_NAME=test_production_tune_slot_1"));
    assert!(wt1_local_env.contains("CLAW_TEST_DB=test_production_tune_slot_1"));
    assert!(wt1_local_env.contains("API_PORT=8001"));

    // Verify Slot 2
    let wt2_worktree_env =
        fs::read_to_string(wt2.join(".env.worktree")).expect("read wt2 worktree");
    let wt2_local_env = fs::read_to_string(wt2.join("local.env")).expect("read wt2 local");
    assert!(wt2_worktree_env.contains("TEST_DATABASE_NAME=test_tune_slot_2"));
    assert!(wt2_worktree_env.contains("CLAW_TEST_DB=test_tune_slot_2"));
    assert!(wt2_local_env.contains("TEST_DATABASE_NAME=test_production_tune_slot_2"));
    assert!(wt2_local_env.contains("CLAW_TEST_DB=test_production_tune_slot_2"));
    assert!(wt2_local_env.contains("API_PORT=8002"));

    // Verify primary repo untouched
    let untouched = fs::read_to_string(repo_root.join("local.env")).expect("read original");
    assert_eq!(untouched, primary_env);
}

/// Verifies database name inference and in-place rewrite across multiple transitions.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_update_env_content_database_inference_and_rewrite() {
    let raw_django_env = r#"
DEBUG=True
DATABASE_NAME=tune
DATABASE_USER=postgres
"#;

    // Slot 1: Appends TEST_DATABASE_NAME and CLAW_TEST_DB based on DATABASE_NAME
    let slot1_env = update_env_content_with_ports(raw_django_env, &BTreeMap::new(), 1, None);
    assert!(slot1_env.contains("TEST_DATABASE_NAME=test_tune_slot_1"));
    assert!(slot1_env.contains("CLAW_TEST_DB=test_tune_slot_1"));

    // Slot 2: Rewrites existing TEST_DATABASE_NAME and CLAW_TEST_DB in-place
    let slot2_env = update_env_content_with_ports(&slot1_env, &BTreeMap::new(), 2, None);
    assert!(slot2_env.contains("TEST_DATABASE_NAME=test_tune_slot_2"));
    assert!(slot2_env.contains("CLAW_TEST_DB=test_tune_slot_2"));
    assert!(!slot2_env.contains("test_tune_slot_1"));
}

/// Verifies Postgres and MySQL environment variable detection with quotes.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_update_env_content_postgres_and_mysql_variants() {
    let postgres_env = "POSTGRES_DB=\"crm_db\"\nPOSTGRES_PORT=5432\n";
    let updated_pg = update_env_content_with_ports(postgres_env, &BTreeMap::new(), 3, None);
    assert!(updated_pg.contains("TEST_DATABASE_NAME=test_crm_db_slot_3"));
    assert!(updated_pg.contains("CLAW_TEST_DB=test_crm_db_slot_3"));

    let mysql_env = "MYSQL_DATABASE='analytics'\n";
    let updated_my = update_env_content_with_ports(mysql_env, &BTreeMap::new(), 4, None);
    assert!(updated_my.contains("TEST_DATABASE_NAME=test_analytics_slot_4"));
    assert!(updated_my.contains("CLAW_TEST_DB=test_analytics_slot_4"));
}

/// Verifies edge cases including commented lines, existing previous slot suffixes, and quoted values.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_update_env_content_edge_cases_and_comments() {
    let env_with_comments = r#"
# Database configuration section
# DATABASE_NAME=commented_out_db
DATABASE_NAME="order_service"
# TEST_DATABASE_NAME=test_old_99
TEST_DATABASE_NAME=test_order_service_slot_99
CLAW_TEST_DB=test_order_service_slot_99
"#;

    let updated = update_env_content_with_ports(env_with_comments, &BTreeMap::new(), 5, None);

    // Active declarations rewritten for slot 5
    assert!(updated.contains("TEST_DATABASE_NAME=test_order_service_slot_5"));
    assert!(updated.contains("CLAW_TEST_DB=test_order_service_slot_5"));
    assert!(!updated.contains("test_order_service_slot_99"));

    // Comments preserved untouched
    assert!(updated.contains("# DATABASE_NAME=commented_out_db"));
    assert!(updated.contains("# TEST_DATABASE_NAME=test_old_99"));
}

/// Verifies fallback behavior when no database configuration is declared in project `.env`.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_fallback_when_no_db_in_env() {
    let env_without_db = "CACHE_PORT=6379\nSECRET_KEY=supersecret\n";
    let updated = update_env_content_with_ports(env_without_db, &BTreeMap::new(), 1, None);

    // Project env without database names must not be contaminated with test db names
    assert!(!updated.contains("TEST_DATABASE_NAME"));
    assert!(!updated.contains("CLAW_TEST_DB"));

    // Worktree env still safely provides fallback test db names derived from repo name
    let dir = tempdir().expect("tempdir");
    write_env_worktree(
        dir.path(),
        "web-service",
        "fix-1",
        1,
        &BTreeMap::new(),
        None,
    )
    .expect("write env");
    let wt_env = fs::read_to_string(dir.path().join(".env.worktree")).expect("read env");
    assert!(wt_env.contains("TEST_DATABASE_NAME=test_web_service_slot_1"));
    assert!(wt_env.contains("CLAW_TEST_DB=test_web_service_slot_1"));

    // extract_base_db edge cases
    assert_eq!(extract_base_db("test_tune_slot_1", Some("tune")), "tune");
    assert_eq!(extract_base_db("test_tune_slot_1_2", Some("tune")), "tune");
    assert_eq!(extract_base_db("", Some("fallback_db")), "fallback_db");
    assert_eq!(extract_base_db("123", Some("fallback_db")), "fallback_db");
    assert_eq!(extract_base_db("test_", Some("fallback_db")), "fallback_db");
    assert_eq!(extract_base_db("", None), "db");
}

/// Verifies end-to-end untracked file copy and merge with database isolation.
///
/// # Examples
///
/// ```
/// // Verified via cargo test --test test_slot_db_isolation
/// ```
#[test]
fn test_copy_and_merge_untracked_files_with_db_isolation() {
    let repo_dir = tempdir().expect("repo_dir");
    let wt_dir = tempdir().expect("wt_dir");

    let repo_root = repo_dir.path();
    let wt_path = wt_dir.path().join("wt-slot-1");
    fs::create_dir_all(&wt_path).expect("create wt_path");

    let env_filename = "app.env";
    let primary_env = "DATABASE_NAME=tune\nAPP_PORT=8000\n";
    fs::write(repo_root.join(env_filename), primary_env).expect("write primary env");

    let mut ports = BTreeMap::new();
    ports.insert("APP_PORT".to_string(), 8000);

    copy_and_merge_untracked_files(
        repo_root,
        &wt_path,
        &[env_filename.to_string()],
        &ports,
        1,
        Some("tune_wt_1"),
    )
    .expect("copy and merge");

    let copied = fs::read_to_string(wt_path.join(env_filename)).expect("read copied env");
    assert!(copied.contains("APP_PORT=8001"));
    assert!(copied.contains("TEST_DATABASE_NAME=test_tune_slot_1"));
    assert!(copied.contains("CLAW_TEST_DB=test_tune_slot_1"));
    assert!(copied.contains("COMPOSE_PROJECT_NAME=tune_wt_1"));

    // Ensure primary repo file was not modified
    let primary_check = fs::read_to_string(repo_root.join(env_filename)).expect("read primary");
    assert_eq!(primary_check, primary_env);
}
