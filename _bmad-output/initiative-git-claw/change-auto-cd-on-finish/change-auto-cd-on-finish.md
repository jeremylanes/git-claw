# Sprint Change Proposal: Automatic Navigation to Primary Repository on Finish (`auto-cd`)

**Date:** 2026-10-18  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request: When finishing a worktree from inside its directory (`git claw finish`), the worktree directory is deleted from disk, leaving the shell prompt trapped in a deleted folder. The shell should automatically navigate (`cd`) back to the primary repository root (`main` or `master`).

---

## 1. Issue Summary

When a developer finishes a feature worktree (`git claw finish`), the worktree folder is removed from the filesystem. If the developer ran the command from within that worktree, their shell process's current working directory remains pointing to the dead directory inode.

In UNIX, a child executable cannot directly alter its parent shell's CWD. To solve this, `git-claw` provides a shell integration hook (`git claw shell-hook`). Currently:
1. `git claw cd` cannot resolve `"main"`, `"master"`, or `"root"` to the primary repository.
2. `git claw shell-hook` only supported bash/zsh and only intercepted `cd` and `start`. It did not intercept `finish` or `spike drop`.
3. Fish shell (`/bin/fish`), which is actively used by the user, was not supported in `git claw shell-hook`.

---

## 2. Requirement Specification

1. **Primary Root Navigation via `git claw cd`:**
   - In `src/workflow/nav.rs`, update target path resolution:
     - If `name` is `"main"`, `"master"`, `"root"`, or matches `config.main_branch()`, return `primary_repo_root`.
     - If `name` is omitted and the shell is not inside a worktree slot, return `primary_repo_root`.
     - Output the canonical primary repository path.

2. **Shell Hook Enhancements (Bash, Zsh, and Fish):**
   - In `src/workflow/shell.rs`, expand `generate_shell_hook`:
     - **Fish shell support** (`git claw shell-hook fish`): Define native fish functions for `claw` and `git claw`.
     - **Bash/Zsh support** (`git claw shell-hook bash|zsh`): Enhance wrapper function.
     - **Lifecycle interceptors**:
       - `finish` (e.g. `git claw finish [name]`, `git claw feature finish`, etc.): Record the primary repository root before deletion. Upon successful completion, if the working directory no longer exists on disk, automatically execute `cd "$primary_root"`.
       - `spike drop`: Apply identical auto-cd behavior.
       - `start`: Auto-cd to the newly created worktree.
       - `cd`: Change directory to any worktree, or to `main`/`master`/`root`.

3. **Persistent Shell Autoloading for Fish:**
   - Place `~/.config/fish/conf.d/git_claw.fish` calling `git claw shell-hook fish | source` so all interactive fish terminals inherit auto-navigation without manual setup.

---

## 3. Impact Analysis

- **Architecture Impact:**
  - `src/workflow/nav.rs`: Add primary repo resolution to `resolve_target_worktree_path`.
  - `src/workflow/shell.rs`: Multi-shell generation (`fish`, `bash`, `zsh`) with `finish`/`drop` hooks.
  - `tests/test_nav_workflow.rs`: Unit & integration tests for `cd main` and `cd master`.
  - `tests/test_shell_hook.rs`: Tests for Fish and Bash hook generation.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Clean implementation, full test suite passing, Sphinx documentation with examples, and updated binary installed.
