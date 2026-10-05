---
title: 'Git directory resolution & advisory file locking (slots.lock)'
type: 'feature'
ticket: 4
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

**Problem:** Concurrent invocations of `git-claw` can race on shared state (`slots.json`), and worktrees need to locate the canonical shared `.git` directory across linked worktrees where `.git` is a file.

**Approach:** Implement `infra::git` to resolve the common Git directory via `git rev-parse --git-common-dir` and toplevel via `git rev-parse --show-toplevel`, and implement `infra::lock` with advisory file locking on `<common-dir>/claw/slots.lock` using `fd-lock` with non-blocking retry polling (50ms) up to a 5-second timeout.

## Boundaries & Constraints

**Always:**
- Resolve common Git directory and toplevel using system Git (`std::process::Command::new("git")`).
- Lock file must reside at `<git-common-dir>/claw/slots.lock`.
- Lock acquisition must retry every 50ms and timeout strictly after 5 seconds if not acquired.
- Releasing lock happens automatically on drop of `SlotLock`.

**Never:**
- Do not use blocking kernel locks that freeze indefinitely without timeout.
- Do not use dynamic C git bindings (`libgit2`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Common dir resolution | Normal git repo or worktree | Returns canonical PathBuf to common Git dir | Ok(PathBuf) |
| Non-git directory | Directory not inside git repo | Returns `GitError::NotAGitRepository` | Err(GitError) |
| Acquire uncontended lock | No other process holding lock | Lock acquired immediately | Ok(SlotLock) |
| Contended lock timeout | Lock held for > 5s | Times out after 5s polling, returns `LockError::Timeout` | Err(LockError::Timeout) |
| Lock drop releases | Lock acquired and dropped | Subsequent acquire succeeds immediately | Ok(SlotLock) |

</frozen-after-approval>

## Code Map

- `src/infra/git.rs` -- Git command wrappers for common directory and toplevel path resolution.
- `src/infra/lock.rs` -- `SlotLock` struct with 50ms polling loop, 5s timeout, and RAII unlock on drop.
- `src/infra/error.rs` -- `GitError` and `LockError` types.
- `src/infra/mod.rs` -- Exports `git`, `lock`, and `error` modules.
- `tests/test_lock.rs` -- Integration tests verifying lock acquisition, contention timeout, and release on drop.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/error.rs` -- Define infrastructure errors -- Implements `GitError` and `LockError`.
- [x] `src/infra/git.rs` -- Implement Git directory resolution -- Wraps `git rev-parse` commands.
- [x] `src/infra/lock.rs` -- Implement advisory file locking with timeout -- Polls `fd_lock::RwLock::try_write` with 50ms sleep and 5s timeout.
- [x] `src/infra/mod.rs` -- Re-export infra components -- Exposes Git and locking helpers.
- [x] `tests/test_lock.rs` -- Test advisory file locking and Git resolution -- Verifies lock contention, timeout, and release.

**Acceptance Criteria:**
- Given a Git repository, when resolving the common Git directory, then the absolute canonical `.git` path is returned.
- Given a lock path, when acquiring `SlotLock`, then lock is granted immediately.
- Given an active `SlotLock`, when a second process/thread attempts to acquire with a short timeout, then it fails with `LockTimeout`.
- Given an active `SlotLock`, when dropped, then a subsequent acquisition succeeds immediately.

## Implementation Notes
- Implemented `resolve_git_common_dir`, `resolve_toplevel`, and `resolve_repo_name` in `src/infra/git.rs` using `std::process::Command` to invoke Git with path canonicalization.
- Implemented `SlotLock` in `src/infra/lock.rs` using `fd-lock` non-blocking `try_write` with 50ms polling loop, configurable timeout, and RAII auto-unlock on drop.
- Implemented `GitError` and `LockError` in `src/infra/error.rs`.
- Re-exported infra utilities in `src/infra/mod.rs`.
- Added integration tests in `tests/test_lock.rs` verifying uncontended acquisition, contention timeout, drop release, and Git path resolution.

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
Implemented Git common directory and top-level repository resolution, and advisory file locking on `slots.lock` with retry polling and a 5-second timeout protecting against race conditions.

### Files changed
- `src/infra/error.rs` - Added `GitError` and `LockError`.
- `src/infra/git.rs` - Git discovery wrappers (`rev-parse --git-common-dir`, `--show-toplevel`).
- `src/infra/lock.rs` - `SlotLock` implementation with polling timeout.
- `src/infra/mod.rs` - Re-exports for infra layer.
- `tests/test_lock.rs` - Integration test suite verifying locking semantics and git discovery.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 26 passed, 0 failed, 0 warnings.
- `cargo check`: Passed with 0 errors and 0 warnings.

### Residual risks
- None. Advisory file locking matches OS semantics and Git path resolution verified.
