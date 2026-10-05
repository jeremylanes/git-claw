---
title: 'Worktree start workflow (git claw <type> start <name>)'
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

**Problem:** The individual primitives (locking, slot allocation, worktree creation, `.env.worktree` writing, cache symlinking, hook execution) need to be composed into an application workflow orchestrated by `workflow::start::start_worktree` and wired into the CLI commands (`feature start`, `bugfix start`, `hotfix start`, `spike start`).

**Approach:** Implement `workflow::start` orchestrating the complete worktree start sequence under exclusive lock, persisting the allocated slot in `slots.json`, writing `.env.worktree`, linking cache directories, running `post_start` hooks, and wire CLI handlers in `src/main.rs`.

## Boundaries & Constraints

**Always:**
- Hold `SlotLock` across both slot allocation and `create_worktree` to prevent races.
- Run `registry.auto_gc` before allocating new slot.
- Ensure leaf names are unique across active slots.
- Support `feature`, `bugfix`, `hotfix`, and `spike` branch types.
- Print clear success output with assigned Slot ID, branch, path, and ports.

**Never:**
- Do not release the lock between slot allocation and worktree directory creation.
- Do not roll back worktree on non-fatal `post_start` hook failure.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Successful feature start | `git-claw feature start auth` | Assigns Slot 1, creates worktree & branch `feature/auth`, writes `.env.worktree` | Ok(slot_id) |
| Duplicate active slot name | Worktree `auth` already active in registry | Refuses creation with descriptive error | Err(WorkflowError) |
| Post-start hook failure | Hook exits 1 | Outputs warning to stderr, exits 0, retains worktree | Ok(slot_id) |

</frozen-after-approval>

## Code Map

- `src/workflow/start.rs` -- Main start workflow orchestration pipeline.
- `src/workflow/error.rs` -- Application workflow error types.
- `src/workflow/mod.rs` -- Re-export start workflow functions.
- `src/main.rs` -- Route CLI subcommands to `workflow::start::start_worktree`.
- `tests/test_start_workflow.rs` -- End-to-end integration tests for `start` workflow.

## Tasks & Acceptance

**Execution:**
- [x] `src/workflow/error.rs` -- Define workflow errors -- Handles domain, infra, and workflow errors.
- [x] `src/workflow/start.rs` -- Implement start workflow -- Orchestrates locking, slot allocation, git worktree add, env file, symlinks, and hooks.
- [x] `src/workflow/mod.rs` -- Re-export start workflow -- Exposes start workflow API.
- [x] `src/main.rs` -- Wire CLI commands to workflow::start -- Connects feature, bugfix, hotfix, spike start subcommands.
- [x] `tests/test_start_workflow.rs` -- Test end-to-end start workflow -- Verifies slot assignment, worktree creation, env file, and branch creation.

**Acceptance Criteria:**
- Given a git repo, when running `git-claw feature start my-feat`, it creates `feature/my-feat`, writes `.env.worktree`, assigns Slot 1, and records it in `slots.json`.
- Given an already active slot name, when attempting to start with the same name, it fails with an explicit error.

## Implementation Notes
- Implemented `start_worktree` in `src/workflow/start.rs` orchestrating locking, slot allocation, git worktree creation, `.env.worktree` generation, cache symlinking, and `post_start` hook execution.
- Added `WorkflowError` in `src/workflow/error.rs`.
- Re-exported start workflow in `src/workflow/mod.rs`.
- Wired CLI commands (`feature start`, `bugfix start`, `hotfix start`, `spike start`) in `src/main.rs`.
- Added end-to-end integration tests in `tests/test_start_workflow.rs`.

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
Implemented end-to-end worktree start workflow (`git claw <type> start <name>`) coordinating advisory locking, slot allocation, Git worktree and branch creation, `.env.worktree` writing, shared cache symlinking, and `post_start` hook execution.

### Files changed
- `src/workflow/error.rs` - Added `WorkflowError`.
- `src/workflow/start.rs` - `start_worktree` pipeline and `StartOptions`.
- `src/workflow/mod.rs` - Re-exports for start workflow.
- `src/main.rs` - Wired CLI subcommands to start workflow.
- `tests/test_start_workflow.rs` - End-to-end integration tests.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 38 passed, 0 failed, 0 warnings.
- `cargo clippy`: 0 warnings.
- `cargo fmt -- --check`: Clean formatting.

### Residual risks
- None. Complete end-to-end worktree start verified with real git repository scaffolding.
