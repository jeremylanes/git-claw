---
title: 'Shell integration (git claw shell-hook) for auto-cd navigation'
type: 'feature'
ticket: 5
created: '2026-10-07'
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
  - '_bmad-output/initiative-git-claw/change-runtime-fixes-and-init/change-runtime-fixes-and-init.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** CLI binaries cannot alter parent shell working directories directly upon creating a new worktree with `git claw feature start <name>`, requiring manual `cd` steps or clunky editor launches.

**Approach:** Implement `git claw shell-hook` emitting bash/zsh wrapper functions:
1. Emits a `claw()` function wrapper that forwards commands to `command git-claw`.
2. When starting a worktree (`feature start`, `bugfix start`, `hotfix start`, `spike start`), upon successful exit, automatically resolves the worktree path via `git-claw cd <name>` and executes `cd "$dest"` in the parent shell.
3. When running `claw cd <name>`, changes the current directory to the resolved worktree.
4. Also emits a transparent `git()` wrapper function intercepting `git claw ...` to provide identical auto-cd behavior.

## Boundaries & Constraints

**Always:**
- Output clean, valid bash/zsh shell code to stdout.
- Preserve exit status codes from `git-claw`.
- Do nothing if `git-claw` command fails (preserve cwd).
- Safe quoting around directory paths.

**Never:**
- Never execute external git shell commands directly (Rule 26).
- Never break standard `git` or other commands passed to the wrapper.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Emit shell hook | `git-claw shell-hook` | Shell code containing `claw()` and `git()` functions | Exit 0 |
| Explicit shell arg | `git-claw shell-hook zsh` | Shell code compatible with zsh/bash | Exit 0 |
| Execution test | Source script and run `claw cd` / `claw start` | Working directory changes to target path | Non-zero forwarded |

</frozen-after-approval>

## Code Map

- `src/cli/args.rs` -- Add `Commands::ShellHook { shell: String }`.
- `src/workflow/shell.rs` -- Implement `generate_shell_hook` logic.
- `src/workflow/mod.rs` -- Expose `generate_shell_hook` and `ShellHookOptions`.
- `src/main.rs` -- Wire `Commands::ShellHook`.
- `tests/test_shell_hook.rs` -- Integration tests validating shell hook script generation and bash execution.

## Tasks & Acceptance

**Execution:**
- [x] `src/cli/args.rs` -- Add `Commands::ShellHook`.
- [x] `src/workflow/shell.rs` -- Implement shell script template.
- [x] `src/workflow/mod.rs` & `src/main.rs` -- Wire `Commands::ShellHook`.
- [x] `tests/test_shell_hook.rs` -- Verify shell hook generation and execution.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- `git claw shell-hook` emits valid shell function.
- In a subshell sourcing the hook, starting a worktree changes working directory.

## Implementation Notes
- Implemented `generate_shell_hook` and `run_shell_hook` in `src/workflow/shell.rs`.
- Emits wrapper functions `claw()` and `git()` for bash and zsh to auto-cd on `start` and `cd`.
- Added `Commands::ShellHook { shell: String }` in `src/cli/args.rs` and dispatched in `src/main.rs`.
- Tested in `tests/test_shell_hook.rs` with real bash subshell execution.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test --test test_shell_hook` -- expected: all tests pass

## Auto Run Result
- `cargo test --test test_shell_hook`: 3/3 passed
- `cargo clippy`: 0 errors, 0 warnings
- `cargo fmt -- --check`: clean

