---
title: 'Herdr multiplexer auto-activation (non-blocking worktree open)'
type: 'feature'
ticket: 11
created: '2026-10-17'
status: 'built'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-herdr-integration/change-herdr-integration.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Users working inside Herdr must manually run `herdr worktree open --branch <branch>` after `git claw feature start <name>` to see their new worktree appear in Herdr's sidebar.

**Approach:** Implement `infra/herdr.rs` to detect Herdr (`HERDR_ENV=1`) and automatically trigger `herdr worktree open --cwd <repo_root> --branch <branch>`. Implement strict test isolation so that automated tests and temporary directories in `/tmp` NEVER trigger Herdr workspaces. Ensure 100% safety and no-op behavior for users without Herdr.

## Boundaries & Constraints

**Always:**
- Check `HERDR_ENV=1` before attempting any Herdr operations.
- Guard against test pollution: immediately abort Herdr invocation if repo path is in `/tmp` or if test environment variables are set.
- Treat Herdr command invocation as strictly non-fatal: if `herdr` command fails or is missing, warn or ignore gracefully; NEVER abort worktree creation.
- Document every function/struct in Sphinx/Rustdoc format with at least one usage example (Rule 22).
- Write minimal, clean, robust code (Rules 20, 30, 31).

**Never:**
- Do not run git commands directly in shell (Rule 26).
- Do not disrupt or slow down users running outside Herdr.
- Do not trigger Herdr workspaces during test execution (`cargo test`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Outside Herdr | `HERDR_ENV` unset | Normal worktree start, skips Herdr completely | Ok(()) |
| Temp repo / test | Repo in `/tmp` | Normal worktree start, skips Herdr completely | Ok(()) |
| Inside Herdr (real repo) | `HERDR_ENV=1`, `herdr` in PATH | Worktree created, calls `herdr worktree open`, prints confirmation | Ok(()) |
| Herdr error / crash | `HERDR_ENV=1`, `herdr` fails | Worktree created, warns gracefully, does not fail | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/infra/herdr.rs`: Functions `is_herdr_active() -> bool` and `open_worktree_in_herdr(repo_root: &Path, branch: &str) -> bool`.
- `src/infra/mod.rs`: Export `herdr` module and functions.
- `src/workflow/start.rs`: Call `open_worktree_in_herdr` post-creation.
- `tests/test_herdr_integration.rs`: Safe unit tests with mocked/isolated environment.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/herdr.rs` -- Implement `is_herdr_active` and `open_worktree_in_herdr` with strict temp dir and test guards.
- [x] `src/infra/mod.rs` -- Export `herdr` module.
- [x] `src/workflow/start.rs` -- Wire Herdr activation after `post_start` hook.
- [x] `tests/test_herdr_integration.rs` -- Add tests verifying inactive fallback, temp dir safety, and active trigger.
- [x] Run `cargo check` and `cargo test`.
- [x] Run `cargo install --path .`.

**Acceptance Criteria:**
- If `HERDR_ENV` is unset or repo is in `/tmp`, `git claw start` works identically to before with 0 calls to Herdr.
- If `HERDR_ENV=1` and repo is a real workspace, `git claw start` automatically opens the worktree in Herdr.
- All tests pass with `cargo test` without spawning any workspaces in the user's Herdr window.
- Binary installed in `~/.cargo/bin/git-claw`.
