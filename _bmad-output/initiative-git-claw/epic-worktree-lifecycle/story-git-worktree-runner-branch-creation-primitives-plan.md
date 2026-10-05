---
title: 'Git worktree runner & branch creation primitives'
type: 'feature'
ticket: 1
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

**Problem:** `git-claw` needs low-level primitives to create Git worktrees targeting `{worktree_root}/{name}` with typed branches (`feature/<name>`, `bugfix/<name>`, `hotfix/<name>`, `spike/<name>`) based off a base branch (e.g. `main`), with atomic cleanup/rollback if creation fails.

**Approach:** Implement `infra::worktree` providing `create_worktree(repo_root, worktree_path, branch, base_branch)` and branch type parsing/validation, wrapping `git worktree add -b <branch> <path> <base_branch>` with clean error propagation and rollback.

## Boundaries & Constraints

**Always:**
- Execute system git via `std::process::Command` without dynamic C bindings.
- Allowed branch types: `feature`, `bugfix`, `hotfix`, `spike`.
- Clean up partially created worktree directories if `git worktree add` fails.

**Never:**
- Do not hardcode repository roots or branches.
- Do not silently ignore git execution failures.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Valid worktree creation | Valid repo, target path, branch `feature/test-1`, base `main` | Invokes `git worktree add -b feature/test-1 <path> main`, dir created | Ok(()) |
| Branch already exists | Branch `feature/test-1` already exists in git | Fails with explicit error, cleans up any created directory | Err(WorktreeError) |
| Invalid base branch | Base branch `non-existent` | Git fails, runner rolls back and returns descriptive error | Err(WorktreeError) |

</frozen-after-approval>

## Code Map

- `src/infra/worktree.rs` -- Git worktree add and removal primitives.
- `src/core/branch.rs` -- Branch type validation (`feature`, `bugfix`, `hotfix`, `spike`) and name formatting.
- `src/infra/error.rs` -- Add `WorktreeError`.
- `src/infra/mod.rs` -- Re-export worktree primitives.
- `tests/test_worktree.rs` -- Integration test creating worktree in temporary git repo.

## Tasks & Acceptance

**Execution:**
- [x] `src/core/branch.rs` -- Implement branch type model and validation -- Formats `<type>/<name>` and validates kebab-case names.
- [x] `src/infra/error.rs` -- Add WorktreeError -- Defines typed errors for worktree operations.
- [x] `src/infra/worktree.rs` -- Implement worktree creation primitive -- Executes `git worktree add -b` with rollback on failure.
- [x] `src/infra/mod.rs` -- Re-export worktree primitives -- Exposes worktree API.
- [x] `tests/test_worktree.rs` -- Test worktree runner in temporary Git repository -- Verifies worktree creation and failure cleanup.

**Acceptance Criteria:**
- Given a Git repo, when `create_worktree` is called with branch `feature/my-feat`, the worktree directory is created and attached to the branch.
- Given a branch that already exists, when `create_worktree` is called, it returns an error and leaves no residual empty directory.

## Implementation Notes
- Implemented `BranchType` enum (`Feature`, `Bugfix`, `Hotfix`, `Spike`) and `validate_leaf_name` kebab-case validator in `src/core/branch.rs`.
- Implemented `create_worktree`, `remove_worktree`, and `delete_branch` in `src/infra/worktree.rs` with automatic rollback on error.
- Added `BranchError` to `src/core/error.rs` and `WorktreeError` to `src/infra/error.rs`.
- Re-exported branch models in `src/core/mod.rs` and worktree functions in `src/infra/mod.rs`.
- Added unit tests in `src/core/branch.rs` and integration tests in `tests/test_worktree.rs`.

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
Implemented Git worktree creation, removal, and branch deletion primitives with branch type validation (`feature/`, `bugfix/`, `hotfix/`, `spike/`) and rollback on failure.

### Files changed
- `src/core/branch.rs` - Branch type enum and kebab-case validator.
- `src/core/error.rs` - Added `BranchError`.
- `src/core/mod.rs` - Re-exports for branch model.
- `src/infra/error.rs` - Added `WorktreeError`.
- `src/infra/worktree.rs` - Low-level worktree runner with failure rollback.
- `src/infra/mod.rs` - Re-exports for worktree functions.
- `tests/test_worktree.rs` - Integration test suite verifying worktree lifecycle and failure rollback.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 32 passed, 0 failed, 0 warnings.
- `cargo clippy`: 0 warnings.
- `cargo fmt -- --check`: Passed.

### Residual risks
- None. System git delegation and cleanup on failure verified with real temporary git repos.
