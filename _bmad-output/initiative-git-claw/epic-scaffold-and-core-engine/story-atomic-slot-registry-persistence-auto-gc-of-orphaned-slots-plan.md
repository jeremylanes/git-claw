---
title: 'Atomic slot registry persistence & auto-GC of orphaned slots'
type: 'feature'
ticket: 5
created: '2026-10-06'
status: 'built'
followup_review_recommended: false
baseline_revision: '68eb24820f3950ffc28102d78be771c7168787e9'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `git-claw` needs to persist slot metadata atomically to `<git-common-dir>/claw/slots.json` without risk of partial writes or corruption, and self-heal by garbage-collecting orphaned slots whose worktree directories were manually removed.

**Approach:** Implement `SlotRegistry` and `SlotRecord` in `infra::registry` with atomic write via tempfile rename, JSON serialization matching schema version 1, and self-healing auto-GC that validates path existence, purges orphaned records, and invokes `git worktree prune`.

## Boundaries & Constraints

**Always:**
- Use atomic rename (`slots.json.tmp` -> `slots.json` in the same directory) for all mutations.
- Registry schema must match version 1: `version: 1`, `slots: Vec<SlotRecord>`.
- Auto-GC must prune slots whose worktree paths no longer exist on disk (subject to grace period).
- Return clean `RegistryError` on JSON or I/O failures.

**Never:**
- Never perform in-place non-atomic writes to `slots.json`.
- Never fail silently if directory creation or file permissions fail.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Missing slots.json | File does not exist | Returns empty `SlotRegistry` with `version: 1` | Ok(SlotRegistry::empty()) |
| Atomic save | Modify slots and save | Writes tempfile, flushes, renames to `slots.json` | Ok(()) |
| Auto-GC orphaned worktree | Worktree directory removed from disk | Purges stale slot entry, saves registry, invokes `git worktree prune` | Purged count returned |
| Active worktree retained | Worktree directory exists on disk | Retains slot in registry | Slot unchanged |

</frozen-after-approval>

## Code Map

- `src/infra/registry.rs` -- `SlotRegistry`, `SlotRecord`, atomic file I/O, and `auto_gc_orphaned_slots`.
- `src/infra/git.rs` -- Add `git_worktree_prune` wrapper.
- `src/infra/error.rs` -- Add `RegistryError`.
- `src/infra/mod.rs` -- Re-export `SlotRegistry`, `SlotRecord`.
- `tests/test_registry.rs` -- Integration tests for atomic persistence, reloading, and auto-GC.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/error.rs` -- Add RegistryError -- Defines errors for JSON serialization and I/O.
- [x] `src/infra/git.rs` -- Add git worktree prune wrapper -- Executes `git worktree prune`.
- [x] `src/infra/registry.rs` -- Implement SlotRegistry with atomic persistence and auto-GC -- Handles JSON IO and self-healing.
- [x] `src/infra/mod.rs` -- Re-export registry types -- Exposes `SlotRegistry` and `SlotRecord`.
- [x] `tests/test_registry.rs` -- Implement registry persistence and GC tests -- Verifies atomic persistence and orphaned slot cleanup.

**Acceptance Criteria:**
- Given a path to a non-existent `slots.json`, when loading, an empty `SlotRegistry` (version 1) is returned.
- Given a `SlotRegistry`, when saving, it is written atomically and can be read back with identical records.
- Given an active slot whose directory was deleted, when running auto-GC, the slot is purged from the registry and `git worktree prune` is executed.

## Implementation Notes
- Implemented `SlotRecord` and `SlotRegistry` in `src/infra/registry.rs` matching JSON schema version 1.
- Implemented atomic persistence with `File::create` on temporary file, `sync_all`, and `fs::rename`.
- Implemented `auto_gc` checking disk existence of recorded worktrees and invoking `git_worktree_prune` upon purging.
- Added `RegistryError` to `src/infra/error.rs` and `git_worktree_prune` to `src/infra/git.rs`.
- Re-exported registry types in `src/infra/mod.rs`.
- Added integration tests in `tests/test_registry.rs`.

## Plan Change Log

## Review Triage Log

### 2026-10-06 — Review pass
- verdicts: 0 findings — high 0, medium 0, low 0, false 0, maybe-false 0
- findings: []

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test` -- expected: all unit and integration tests pass

## Auto Run Result

### Summary of implemented change
Implemented atomic persistence for `slots.json` using temporary file rename and self-healing auto-GC to detect deleted worktree directories and clean Git worktree metadata via `git worktree prune`.

### Files changed
- `src/infra/error.rs` - Added `RegistryError`.
- `src/infra/git.rs` - Added `git_worktree_prune`.
- `src/infra/registry.rs` - `SlotRecord`, `SlotRegistry`, atomic persistence, search helpers, and auto-GC.
- `src/infra/mod.rs` - Re-exports for registry types.
- `tests/test_registry.rs` - Integration test suite verifying atomic save, load, and auto-GC purging.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 29 passed, 0 failed, 0 warnings.
- `cargo check`: Passed with 0 errors and 0 warnings.

### Residual risks
- None. Atomic save protects against corruption and self-healing auto-GC verified.
