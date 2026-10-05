use git_claw::core::slot::allocate_lowest_slot;
use git_claw::infra::lock::SlotLock;
use git_claw::infra::registry::{SlotRecord, SlotRegistry};
use std::collections::HashSet;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::tempdir;

#[test]
fn test_concurrent_slot_allocation_stress() {
    let dir = tempdir().expect("tempdir");
    let lock_path = dir.path().join("claw/slots.lock");
    let registry_path = dir.path().join("claw/slots.json");

    let num_threads = 8;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for i in 0..num_threads {
        let b = Arc::clone(&barrier);
        let l_path = lock_path.clone();
        let r_path = registry_path.clone();

        let handle = thread::spawn(move || {
            b.wait(); // Synchronize all threads to compete at the exact same moment

            let _lock = SlotLock::acquire(&l_path).expect("acquires lock under contention");

            let mut registry = SlotRegistry::load_or_empty(&r_path).expect("loads registry");
            let next_slot = allocate_lowest_slot(registry.occupied_ids());

            registry.slots.push(SlotRecord {
                id: next_slot,
                name: format!("worker-{}", i),
                branch_type: "feature".to_string(),
                branch: format!("feature/worker-{}", i),
                path: format!("/path/worker-{}", i),
                allocated_at: "1000".to_string(),
            });

            registry
                .save_atomic(&r_path)
                .expect("saves registry atomically");
            // lock dropped here
            next_slot
        });
        handles.push(handle);
    }

    let mut allocated_slots = Vec::new();
    for handle in handles {
        let slot = handle.join().expect("thread completed successfully");
        allocated_slots.push(slot);
    }

    assert_eq!(allocated_slots.len(), num_threads);

    // Verify all allocated slots are strictly unique
    let unique_set: HashSet<u32> = allocated_slots.into_iter().collect();
    assert_eq!(unique_set.len(), num_threads);

    // Verify disk registry state
    let loaded = SlotRegistry::load_or_empty(&registry_path).expect("loads registry from disk");
    assert_eq!(loaded.slots.len(), num_threads);

    let disk_ids: HashSet<u32> = loaded.slots.into_iter().map(|s| s.id).collect();
    assert_eq!(disk_ids, unique_set);
    for expected_id in 1..=(num_threads as u32) {
        assert!(disk_ids.contains(&expected_id));
    }
}
