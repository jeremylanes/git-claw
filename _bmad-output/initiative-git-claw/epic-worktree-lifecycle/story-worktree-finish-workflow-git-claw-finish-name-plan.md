---
title: 'Worktree finish workflow (git claw finish [name])'
type: 'feature'
ticket: 6
created: '2026-10-06'
status: 'built'
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

**Problem:** Developers need a safe teardown command (`git claw finish [name] [--force]`) that validates `pre_finish` hooks, merges worktree changes into the primary repository branch (e.g. `main`), removes worktree files and Git metadata, deletes the merged branch, releases the Slot ID in `slots.json`, and triggers `post_finish` hooks.

**Approach:** Implement `workflow::finish` following the strict sequential teardown order defined in AD-8, with worktree inference when invoked inside a worktree directory, Git merge execution, and atomic registry slot release under `SlotLock`.

## Boundaries & Constraints

**Always:**
- Execute teardown in strict AD-8 sequence:
  1. `pre_finish` hook (abort unless `--force`).
  2. Verify primary working tree is clean.
  3. Merge branch into `main_branch`.
  4. Remove worktree via `git worktree remove --force`.
  5. Delete branch via `git branch -d`.
  6. Release Slot ID in `slots.json`.
  7. Run `post_finish` hook.
- Infer active worktree from CWD if `name` is omitted.
- Hold `SlotLock` while modifying `slots.json`.

**Never:**
- Do not delete unmerged branch without checking merge success (except spikes, which use spike drop).
- Do not release slot ID if merge failed.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Finish active worktree | Active worktree `auth` with commit | Merges into `main`, removes worktree, deletes branch, frees Slot ID | Ok(()) |
| Inferred worktree | CWD inside worktree, no `name` arg | Infers `name` from path, completes teardown | Ok(()) |
| Pre-finish hook failure without force | `pre_finish` exits 1, `force = false` | Aborts before merge or worktree removal | Err(WorkflowError) |
| Pre-finish hook failure with force | `pre_finish` exits 1, `force = true` | Warns, proceeds with merge and cleanup | Ok(()) |
| Unknown worktree | Name not found in registry | Exits with error "Worktree not found" | Err(WorkflowError) |

</frozen-after-approval>

## Code Map

- `src/workflow/finish.rs` -- Main finish workflow orchestration pipeline.
- `src/infra/git.rs` -- Add `git_merge` and `git_is_clean` helpers.
- `src/workflow/mod.rs` -- Re-export `finish_worktree`.
- `src/main.rs` -- Wire CLI `finish` subcommand to `workflow::finish_worktree`.
- `tests/test_finish_workflow.rs` -- Integration tests for worktree finishing, merging, and slot release.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/git.rs` -- Add git merge and status clean helpers -- Wraps `git merge` and `git status --porcelain`.
- [x] `src/workflow/finish.rs` -- Implement finish_worktree workflow -- Implements strict AD-8 teardown order.
- [x] `src/workflow/mod.rs` -- Re-export finish workflow -- Exposes `finish_worktree`.
- [x] `src/main.rs` -- Wire finish command -- Connects `Commands::Finish` to workflow handler.
- [x] `tests/test_finish_workflow.rs` -- Integration test for finish workflow -- Verifies merge, cleanup, and slot release.

**Acceptance Criteria:**
- Given an active worktree with a commit, when running `git claw finish`, it merges changes to `main`, removes the worktree, and frees the Slot ID.
- Given a failing `pre_finish` hook, `git claw finish` aborts without merging unless `--force` is passed.

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
- `cargo test`: PASS (41 passed: 11 unit tests, 30 integration tests across 12 suites)
