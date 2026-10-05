---
title: 'Navigation and editor integration (git claw open and git claw cd)'
type: 'feature'
ticket: 3
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

**Problem:** Navigating between parallel worktrees and opening them in editors requires knowing and typing long paths. Developers need `git claw cd [name]` to print the worktree path for shell aliases (e.g. `cd $(git claw cd <name>)`) and `git claw open [name]` to launch their preferred editor directly targeting the worktree directory.

**Approach:** Implement `workflow::nav` (or `workflow::open` and `workflow::cd`) in `src/workflow/nav.rs`.
- `cd_worktree`: Resolves the target worktree path (explicit name or inferred from CWD) and prints the raw path to stdout.
- `open_worktree`: Resolves the target worktree path, identifies the editor from `$EDITOR`, `$VISUAL`, or fallback (`code`), and spawns the editor targeting the directory.

## Boundaries & Constraints

**Always:**
- `cd` must print ONLY the raw path to stdout without ANSI styling or prefixes, ensuring clean shell evaluation `cd $(git claw cd <name>)`.
- Support explicit worktree name or inferred worktree when inside a worktree directory.
- Fail explicitly with `WorktreeNotFound` if target slot cannot be located.

**Never:**
- Do not output ANSI color codes on `git claw cd`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Query worktree path | `git claw cd my-feat` | Prints `/path/to/my-feat` on stdout | Ok(()) |
| Query path inferred | Inside worktree, `git claw cd` | Prints current worktree path on stdout | Ok(()) |
| Open editor | `git claw open my-feat` | Spawns configured editor on worktree path | Ok(()) |
| Unknown worktree | `git claw cd unknown` | Prints error to stderr, exits 1 | Err(WorkflowError::WorktreeNotFound) |

</frozen-after-approval>

## Code Map

- `src/workflow/nav.rs` -- Main navigation and editor launcher implementations (`cd_worktree`, `open_worktree`).
- `src/workflow/mod.rs` -- Re-export `cd_worktree`, `open_worktree`, `CdOptions`, `OpenOptions`.
- `src/main.rs` -- Wire `Commands::Cd` and `Commands::Open`.
- `tests/test_nav_workflow.rs` -- Integration tests for `git claw cd` and `git claw open`.

## Tasks & Acceptance

**Execution:**
- [x] `src/workflow/nav.rs` -- Implement `cd_worktree` and `open_worktree`.
- [x] `src/workflow/mod.rs` -- Expose navigation workflow functions.
- [x] `src/main.rs` -- Wire `Commands::Cd` and `Commands::Open`.
- [x] `tests/test_nav_workflow.rs` -- Integration tests for path printing and editor invocation.

**Acceptance Criteria:**
- `git claw cd <name>` prints the worktree path to stdout with exit code 0.
- `git claw open <name>` invokes the configured editor on the worktree path.

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
- `cargo test`: PASS (55 passed: 11 unit tests, 44 integration tests across 16 suites)
