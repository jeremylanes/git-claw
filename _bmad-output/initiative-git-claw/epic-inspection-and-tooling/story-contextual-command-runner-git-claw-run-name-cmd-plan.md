---
title: 'Contextual command runner (git claw run [name] <cmd...>)'
type: 'feature'
ticket: 2
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

**Problem:** Developers and AI agents working on microservices across multiple worktrees need to run arbitrary commands (e.g. `npm test`, `cargo check`, `docker compose up`) inside a worktree with the corresponding `.env.worktree` (Slot ID and calculated ports) injected into the process environment, without having to manually CD or source files.

**Approach:** Implement `workflow::run_command` in `src/workflow/run.rs`. If `name` matches an active slot in `slots.json`, the command is executed in that worktree directory. If `name` does not match but the current directory is inside a worktree, `name` is prepended to `command` and executed in the current worktree. `.env.worktree` is parsed and exported to child process environment variables. The child process exit code is returned and forwarded to the caller.

## Boundaries & Constraints

**Always:**
- Load `.env.worktree` from the target worktree and inject all key-value pairs into the child process.
- Set child working directory to the target worktree path.
- Forward child exit code to the caller.
- Support both explicit worktree name and inferred worktree from current working directory.

**Never:**
- Do not modify or delete worktree state or registry records during execution.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Explicit worktree name | `git claw run my-feat env` | Runs `env` in `my-feat` worktree with `.env.worktree` loaded | Returns exit code 0 |
| Inferred worktree | Inside worktree, `git claw run env` | Detects worktree, prepends `env` to command, runs in cwd worktree | Returns exit code 0 |
| Non-zero exit code | Command exits 42 | Exit code 42 forwarded | Returns exit code 42 |
| Worktree not found | Unknown name and not inside worktree | Aborts with error | Err(WorkflowError::WorktreeNotFound) |
| Empty command | No command given | Aborts with error | Err(WorkflowError) |

</frozen-after-approval>

## Code Map

- `src/workflow/run.rs` -- Main run_command workflow implementation.
- `src/workflow/mod.rs` -- Re-export `run_command` and `RunOptions`.
- `src/main.rs` -- Wire `Commands::Run` to `workflow::run_command`.
- `tests/test_run_workflow.rs` -- Integration tests for contextual execution, env injection, and exit code forwarding.

## Tasks & Acceptance

**Execution:**
- [x] `src/workflow/run.rs` -- Implement `run_command` with .env parsing and child execution.
- [x] `src/workflow/mod.rs` -- Expose `run_command`.
- [x] `src/main.rs` -- Wire `Commands::Run`.
- [x] `tests/test_run_workflow.rs` -- Integration tests verifying env injection and exit code forwarding.

**Acceptance Criteria:**
- Given a worktree with `.env.worktree`, `git claw run <name> env` prints `CLAW_SLOT_ID` and port variables.
- Exit code of child command is preserved verbatim.

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
- `cargo test`: PASS (51 passed: 11 unit tests, 40 integration tests across 15 suites)
