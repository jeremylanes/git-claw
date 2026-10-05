---
title: 'Lifecycle hooks runner (post_start, pre_finish, post_finish)'
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

**Problem:** `git-claw` needs to run user-configured lifecycle hooks (`post_start`, `pre_finish`, `post_finish`) with `.env.worktree` environment variables injected, adhering to strict failure semantics (`post_start` non-fatal warning, `pre_finish` fatal abort unless `--force`, `post_finish` non-fatal warning).

**Approach:** Implement `infra::hook` to execute shell commands with environment inheritance and custom env variables in a specified working directory, and provide hook runner helpers enforcing the AD-6 lifecycle fault tolerance policies.

## Boundaries & Constraints

**Always:**
- Execute hooks via `sh -c "<hook_command>"` (or `cmd.exe /C` on Windows).
- Inject all `.env.worktree` environment variables into hook subprocess execution.
- `post_start` failure prints a warning to stderr and returns `Ok(false)` without aborting.
- `pre_finish` failure aborts with `HookError::PreFinishFailed` unless `force: true`.
- `post_finish` failure prints a warning to stderr and returns `Ok(false)`.

**Never:**
- Do not let a failing `post_start` hook delete or roll back the worktree.
- Do not bypass `pre_finish` failure unless `--force` is explicitly provided.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Successful hook | Valid command `echo hello` | Executes, outputs to terminal, returns success | Ok(true) |
| Failing post_start | Command exits 1 | Outputs warning to stderr, returns Ok(false) | Non-fatal warning |
| Failing pre_finish without force | Command exits 1, `force = false` | Returns `Err(HookError::PreFinishFailed)` | Err(HookError) |
| Failing pre_finish with force | Command exits 1, `force = true` | Outputs warning, returns Ok(false) | Continues |
| Environment injection | Command `echo $PORT_WEB` with env | Subprocess has `$PORT_WEB` in environment | Variable accessible |

</frozen-after-approval>

## Code Map

- `src/infra/hook.rs` -- Hook execution logic, shell command invocation, and AD-6 policy handlers.
- `src/infra/error.rs` -- Add `HookError`.
- `src/infra/mod.rs` -- Re-export hook functions.
- `tests/test_hooks.rs` -- Integration tests for hook execution, environment injection, and failure policies.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/error.rs` -- Add HookError -- Defines errors for hook execution failure.
- [x] `src/infra/hook.rs` -- Implement hook execution and policy enforcement -- Shell execution and AD-6 handling.
- [x] `src/infra/mod.rs` -- Re-export hook utilities -- Exposes hook runner to workflows.
- [x] `tests/test_hooks.rs` -- Integration tests for lifecycle hooks -- Tests env injection and fatal/non-fatal policies.

**Acceptance Criteria:**
- Given a command with environment variables, when executed via hook runner, the variables are readable by the command.
- Given a failing `pre_finish` command with `force = false`, hook runner returns an error.
- Given a failing `pre_finish` command with `force = true`, hook runner warns and allows completion.
- Given a failing `post_start` command, hook runner warns to stderr and returns Ok without aborting.

## Implementation Notes
- Implemented `execute_hook_command`, `run_post_start_hook`, `run_pre_finish_hook`, and `run_post_finish_hook` in `src/infra/hook.rs`.
- Added `HookError` to `src/infra/error.rs`.
- Re-exported hook functions in `src/infra/mod.rs`.
- Added integration tests in `tests/test_hooks.rs` verifying environment variable inheritance, `post_start` non-fatal warning, `pre_finish` fatal abort and `--force` override, and `post_finish` non-fatal warning.

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
Implemented lifecycle hook runner executing shell commands with worktree environment variables injected, supporting warning-and-retain on `post_start` failure and fatal abort on `pre_finish` failure unless `--force` is supplied.

### Files changed
- `src/infra/error.rs` - Added `HookError`.
- `src/infra/hook.rs` - Lifecycle hook runners and policy enforcement.
- `src/infra/mod.rs` - Re-exports for hook functions.
- `tests/test_hooks.rs` - Integration test suite verifying hook execution and failure semantics.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 41 passed, 0 failed, 0 warnings.
- `cargo clippy`: 0 warnings.
- `cargo fmt -- --check`: Clean formatting.

### Residual risks
- None. Hook execution and failure policies match AD-6 rules.
