use git_claw::infra::registry::{SlotRecord, SlotRegistry};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_empty_registry_when_absent() {
    let dir = tempdir().expect("tempdir");
    let registry_path = dir.path().join("claw/slots.json");

    let registry = SlotRegistry::load_or_empty(&registry_path).expect("loads empty");
    assert_eq!(registry.version, 1);
    assert!(registry.slots.is_empty());
}

#[test]
fn test_save_atomic_and_reload() {
    let dir = tempdir().expect("tempdir");
    let registry_path = dir.path().join("claw/slots.json");

    let mut registry = SlotRegistry::empty();
    registry.slots.push(SlotRecord {
        id: 1,
        name: "feature-auth".to_string(),
        branch_type: "feature".to_string(),
        branch: "feature/feature-auth".to_string(),
        path: "/path/to/worktree".to_string(),
        allocated_at: "1000".to_string(),
    });

    registry
        .save_atomic(&registry_path)
        .expect("saves atomically");

    let loaded = SlotRegistry::load_or_empty(&registry_path).expect("loads saved");
    assert_eq!(loaded.slots.len(), 1);
    assert_eq!(loaded.slots[0].id, 1);
    assert_eq!(loaded.slots[0].name, "feature-auth");
}

#[test]
fn test_auto_gc_purges_deleted_worktrees() {
    let dir = tempdir().expect("tempdir");
    let registry_path = dir.path().join("claw/slots.json");

    let existing_worktree = dir.path().join("worktrees/active");
    fs::create_dir_all(&existing_worktree).expect("creates dir");

    let deleted_worktree = dir.path().join("worktrees/deleted");

    let mut registry = SlotRegistry::empty();
    registry.slots.push(SlotRecord {
        id: 1,
        name: "active".to_string(),
        branch_type: "feature".to_string(),
        branch: "feature/active".to_string(),
        path: existing_worktree.to_string_lossy().to_string(),
        allocated_at: "1000".to_string(),
    });
    registry.slots.push(SlotRecord {
        id: 2,
        name: "stale".to_string(),
        branch_type: "feature".to_string(),
        branch: "feature/stale".to_string(),
        path: deleted_worktree.to_string_lossy().to_string(),
        allocated_at: "1000".to_string(),
    });

    // Run auto-gc without grace period
    let purged = registry
        .auto_gc(&registry_path, None, false)
        .expect("runs auto-gc");

    assert_eq!(purged, 1);
    assert_eq!(registry.slots.len(), 1);
    assert_eq!(registry.slots[0].name, "active");

    // Verify written to disk
    let loaded = SlotRegistry::load_or_empty(&registry_path).expect("loads registry");
    assert_eq!(loaded.slots.len(), 1);
    assert_eq!(loaded.slots[0].name, "active");
}
