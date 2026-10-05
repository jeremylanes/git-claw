---
title: 'Spike abandonment workflow (git claw spike drop <name>)'
type: 'feature'
ticket: 7
created: '2026-10-06'
status: built
baseline_revision: '68eb24820f3950ffc28102d78be771c7168787e9'
route: 'full'
route_source: 'auto'
risk: 'low'
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

**Problem:** Developers and AI agents use spike worktrees for throwaway experiments. When the experiment is finished or rejected, they need `git claw spike drop <name>` to delete the worktree and branch completely without merging into `main`, without triggering `pre_finish` or `post_finish` hooks, and cleanly freeing the allocated Slot ID.

**Approach:** Implement `workflow::drop_spike` in `src/workflow/spike.rs` following FR-5. The handler acquires the advisory `SlotLock`, locates the slot by name, forces removal of the worktree directory via `git worktree remove --force`, forcefully deletes the branch via `git branch -D`, purges the slot record from `slots.json`, and outputs a success confirmation.

## Boundaries & Constraints

**Always:**
- Acquire `SlotLock` before modifying `slots.json` or worktree filesystem.
- Use `git worktree remove --force` to delete the worktree cleanly even with untracked/dirty experimental files.
- Force delete branch with `-D`.
- Purge slot entry from `slots.json` atomically.
- Bypass `pre_finish` and `post_finish` hooks completely.

**Never:**
- Never merge spike branch into `main`.
- Never fail silently if the worktree slot cannot be found.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Drop active spike | Existing spike `proto` in slot registry | Worktree removed, branch `spike/proto` deleted with `-D`, slot freed | Ok(()) |
| Worktree not found | Name not registered in `slots.json` | Aborts without modifying Git state | Err(WorkflowError::WorktreeNotFound) |
| Worktree already deleted on disk | Directory gone but slot in registry | Worktree pruned, branch deleted, slot purged | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/workflow/spike.rs` -- Main drop_spike workflow implementation.
- `src/workflow/mod.rs` -- Export `drop_spike` and `DropSpikeOptions`.
- `src/main.rs` -- Wire `Commands::Spike { action: SpikeAction::Drop { name } }` to `drop_spike`.
- `tests/test_spike_workflow.rs` -- Integration tests for spike start and drop workflow.

## Tasks & Acceptance

**Execution:**
- [x] `src/workflow/spike.rs` -- Implement `drop_spike` function.
- [x] `src/workflow/mod.rs` -- Expose `drop_spike`.
- [x] `src/main.rs` -- Wire CLI `spike drop` to `workflow::drop_spike`.
- [x] `tests/test_spike_workflow.rs` -- Integration tests for spike creation and abandonment.

**Acceptance Criteria:**
- Given an active spike worktree with unmerged commits, when running `git claw spike drop <name>`, it removes the worktree and purges the branch without merging to `main`, bypassing finish hooks, and frees the Slot ID.
- Given an unknown worktree name, it returns a descriptive `WorktreeNotFound` error.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test` -- expected: all unit and integration tests pass

### Auto Run Result
- `cargo fmt -- --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo test`: PASS (44 passed: 11 unit tests, 33 integration tests across 13 suites)
