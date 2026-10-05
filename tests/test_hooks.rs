use git_claw::infra::error::HookError;
use git_claw::infra::hook::{
    execute_hook_command, run_post_finish_hook, run_post_start_hook, run_pre_finish_hook,
};
use std::collections::BTreeMap;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_hook_environment_injection() {
    let dir = tempdir().expect("tempdir");
    let mut env = BTreeMap::new();
    env.insert("CLAW_TEST_VAR".to_string(), "claw_works".to_string());

    let cmd = "echo $CLAW_TEST_VAR > result.txt";
    let success = execute_hook_command(cmd, dir.path(), &env).expect("executes hook");
    assert!(success);

    let output = fs::read_to_string(dir.path().join("result.txt")).expect("reads result");
    assert_eq!(output.trim(), "claw_works");
}

#[test]
fn test_post_start_failure_retains_worktree() {
    let dir = tempdir().expect("tempdir");
    let env = BTreeMap::new();

    // Command that fails (exit 1)
    let res = run_post_start_hook(Some("exit 1"), dir.path(), &env);
    assert!(!res.unwrap());
}

#[test]
fn test_pre_finish_failure_aborts_without_force() {
    let dir = tempdir().expect("tempdir");
    let env = BTreeMap::new();

    let res = run_pre_finish_hook(Some("exit 1"), dir.path(), &env, false);
    assert!(matches!(res, Err(HookError::PreFinishFailed { .. })));
}

#[test]
fn test_pre_finish_failure_proceeds_with_force() {
    let dir = tempdir().expect("tempdir");
    let env = BTreeMap::new();

    let res = run_pre_finish_hook(Some("exit 1"), dir.path(), &env, true);
    assert!(!res.unwrap());
}

#[test]
fn test_post_finish_failure_is_non_fatal() {
    let dir = tempdir().expect("tempdir");
    let env = BTreeMap::new();

    let res = run_post_finish_hook(Some("exit 1"), dir.path(), &env);
    assert!(!res.unwrap());
}
