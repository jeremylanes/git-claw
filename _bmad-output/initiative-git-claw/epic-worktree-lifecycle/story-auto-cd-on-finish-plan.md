---
title: 'Automatic Navigation to Primary Repository on Finish (auto-cd)'
type: 'feature'
ticket: 16
created: '2026-10-18'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-auto-cd-on-finish/change-auto-cd-on-finish.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** When `git claw finish` deletes a worktree directory while the user is inside it, the shell remains in the dead folder. The user wants to automatically be returned to the primary repository root (`main` or `master`).

**Approach:**
1. Update `git claw cd` so that `"main"`, `"master"`, `"root"`, or the configured trunk branch resolves to the primary repository root.
2. In `git claw shell-hook`, support `fish` in addition to `bash`/`zsh`, and intercept `finish` and `spike drop` commands.
3. If the working directory was deleted during `finish` or `spike drop`, the shell wrapper automatically executes `cd "$primary_root"`.

## Boundaries & Constraints

**Always:**
- Keep all existing commands backward compatible.
- Document functions in Sphinx/Rustdoc format with usage examples (Rule 22).
- Write minimal, clean, robust code (Rules 20, 30, 31).

**Never:**
- Do not run git commands directly in the shell terminal (Rule 26).
- Do not fail if `cd main` is called outside of a git repository (handle errors cleanly).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| `git claw cd main` | Repository with main branch | Prints path to primary repository root | Ok(PathBuf) |
| `git claw cd master` | Repository with master branch | Prints path to primary repository root | Ok(PathBuf) |
| Finish from worktree (with hook) | CWD inside worktree, run `git claw finish` | Worktree removed, shell automatically `cd`s to primary repo | Success |
| Start worktree (with hook) | In primary repo, run `git claw feature start auth` | Worktree created, shell automatically `cd`s to worktree | Success |

</frozen-after-approval>

## Code Map

- `src/workflow/nav.rs`: Update `resolve_target_worktree_path` to handle `"main"`, `"master"`, `"root"`, and configured trunk branch.
- `src/workflow/shell.rs`: Multi-shell generator (`fish`, `bash`, `zsh`) with `finish` and `spike drop` auto-cd.
- `tests/test_nav_workflow.rs`: Integration tests for `cd main` and `cd master`.
- `tests/test_shell_hook.rs`: Tests for Fish and Bash hook generation.

## Tasks & Acceptance

**Execution:**
- [ ] Update `src/workflow/nav.rs`.
- [ ] Update `src/workflow/shell.rs` with Fish and POSIX hooks.
- [ ] Add integration tests in `tests/test_nav_workflow.rs` and `tests/test_shell_hook.rs`.
- [ ] Run `cargo check`, `cargo fmt --check`, `cargo clippy`, and `cargo test`.
- [ ] Run `cargo install --path .`.
- [ ] Install fish conf.d file in `~/.config/fish/conf.d/git_claw.fish`.

**Acceptance Criteria:**
- `git claw cd main` outputs the primary repository root.
- `git claw shell-hook fish` emits a valid Fish script with auto-cd on `start`, `finish`, `drop`, and `cd`.
- All tests pass with `cargo test`.
- Binary installed in `~/.cargo/bin/git-claw`.
