use git_claw::infra::error::LockError;
use git_claw::infra::git::{resolve_git_common_dir, resolve_repo_name, resolve_toplevel};
use git_claw::infra::lock::SlotLock;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_acquire_and_release_lock() {
    let dir = tempdir().expect("creates tempdir");
    let lock_path = dir.path().join("claw/slots.lock");

    {
        let lock = SlotLock::acquire(&lock_path).expect("acquires lock");
        assert_eq!(lock.path(), lock_path);
    } // lock dropped here

    // Re-acquire must succeed immediately
    let lock2 = SlotLock::acquire(&lock_path).expect("re-acquires lock after drop");
    assert_eq!(lock2.path(), lock_path);
}

#[test]
fn test_contended_lock_timeout() {
    let dir = tempdir().expect("creates tempdir");
    let lock_path = dir.path().join("claw/slots.lock");

    let _held_lock = SlotLock::acquire(&lock_path).expect("acquires initial lock");

    // Attempt second acquire in a thread with a short 100ms timeout
    let timeout = Duration::from_millis(100);
    let poll_interval = Duration::from_millis(20);
    let result = SlotLock::acquire_with_timeout(&lock_path, timeout, poll_interval);

    assert!(matches!(result, Err(LockError::Timeout { .. })));
}

#[test]
fn test_git_resolution_in_current_repo() {
    let common_dir = resolve_git_common_dir(None).expect("resolves git common dir");
    assert!(common_dir.exists());

    let toplevel = resolve_toplevel(None).expect("resolves toplevel");
    assert!(toplevel.exists());

    let repo_name = resolve_repo_name(&toplevel);
    assert_eq!(repo_name, "git-claw");
}
