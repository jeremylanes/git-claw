---
title: 'Worktree list dashboard with terminal table formatting (git claw list)'
type: 'feature'
ticket: 1
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

**Problem:** Developers and AI agents need immediate visibility into all running worktrees, their allocated slots, associated ports, filesystem locations, and HEAD commits to prevent port conflicts and monitor active workspaces.

**Approach:** Implement `workflow::list` in `src/workflow/list.rs` and table formatting in `src/cli/output.rs`. `list` automatically triggers pre-flight auto-GC to prune stale or manually deleted worktrees from `slots.json` and git metadata, calculates effective ports from `.git-claw.toml`, queries abbreviated HEAD commits via `git rev-parse --short HEAD`, and renders an aligned, colorized ASCII table to stdout.

## Boundaries & Constraints

**Always:**
- Acquire `SlotLock` before performing auto-GC and reading `slots.json`.
- Execute auto-GC to self-heal and purge orphaned worktrees prior to rendering.
- Columns: Slot ID, Branch Name, Worktree Directory, Assigned Ports, Head Commit.
- Format empty state gracefully (e.g. "No active worktrees found").

**Never:**
- Never crash if a worktree HEAD cannot be read; display "-" gracefully.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Active worktrees present | 2 active slots | Formatted ASCII table with headers and row values | Ok(()) |
| No active worktrees | Empty registry | "No active worktrees found." | Ok(()) |
| Orphaned worktree on disk | Slot in registry but dir deleted | Auto-GC cleans slot and prunes metadata before listing | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/infra/git.rs` -- Helper `git_head_commit(worktree_path)`.
- `src/cli/output.rs` -- Formatter for active worktree table rows.
- `src/workflow/list.rs` -- Implementation of `list_worktrees`.
- `src/workflow/mod.rs` -- Re-export `list_worktrees`.
- `src/main.rs` -- Wire `Commands::List` to `list_worktrees`.
- `tests/test_list_workflow.rs` -- Integration tests for `git claw list` with active worktrees, ports, commits, and auto-GC.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/git.rs` -- Add `git_head_commit` function.
- [x] `src/cli/output.rs` -- Implement table rendering for worktree list.
- [x] `src/workflow/list.rs` -- Implement `list_worktrees` with auto-GC and port calculations.
- [x] `src/workflow/mod.rs` -- Expose `list_worktrees`.
- [x] `src/main.rs` -- Wire `Commands::List`.
- [x] `tests/test_list_workflow.rs` -- Integration tests for list command and table output.

**Acceptance Criteria:**
- Running `git claw list` displays an aligned table with columns: Slot ID, Branch Name, Directory, Assigned Ports, Head Commit.
- Orphaned worktree slots are purged before listing.

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
- `cargo test`: PASS (47 passed: 11 unit tests, 36 integration tests across 14 suites)
