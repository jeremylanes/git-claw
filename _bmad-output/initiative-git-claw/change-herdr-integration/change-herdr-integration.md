# Sprint Change Proposal: Herdr Multiplexer Auto-Activation

**Date:** 2026-10-17  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request: when running `git claw feature start <name>` (or bugfix/hotfix/spike) from within Herdr, the worktree must automatically open in Herdr without requiring manual `herdr worktree open` commands.  
**Critical Constraints:**
1. Must be 100% non-breaking, zero-overhead, and transparent for users who do not use Herdr.
2. Must STRICTLY prevent test execution from ever invoking Herdr (repositories inside `/tmp` or test harnesses must never trigger Herdr workspaces).
3. Do only what the user requested and nothing else.

---

## 1. Issue Summary

Herdr organizes terminals into workspaces, tabs, and panes. When a Git worktree is created externally via `git claw [type] start <name>`, Herdr does not automatically attach a workspace to the new worktree until `herdr worktree open` is invoked.

Requiring users to manually run `herdr worktree open --branch <branch>` after `git claw feature start` disrupts developer flow.

---

## 2. Requirement Specification

1. **Automatic Detection & Strict Safety Guards:**
   - Detect if the current process is running inside Herdr by checking `HERDR_ENV=1`.
   - If `HERDR_ENV` is unset or not `"1"`, the integration is a complete no-op (zero overhead, zero subprocess execution).
   - If `CLAW_DISABLE_HERDR` is set, or if running in a test runner (`CARGO_MANIFEST_DIR` or `RUST_TEST_THREADS`), immediately return `false`.
   - If `repo_root` is inside `/tmp` or `std::env::temp_dir()`, immediately return `false` to guarantee test suites never pollute the user's running Herdr server.

2. **Non-Blocking Worktree Activation:**
   - In `workflow::start`, after worktree creation, `.env.worktree` generation, file copying, and `post_start` hook execution:
     - If Herdr is active and repo is not a temp/test repo:
       ```bash
       herdr worktree open --cwd <repo_root> --branch <branch>
       ```
     - Non-fatal error policy: If `herdr` is not in PATH or returns an error, output a non-blocking warning without failing worktree creation.
     - On success, emit a subtle confirmation (`  → Opened worktree workspace in Herdr`).

3. **Universal Compatibility:**
   - Zero impact and zero errors for users without Herdr.
   - Zero pollution of Herdr during `cargo test`.

---

## 3. Impact Analysis

- **PRD Impact:** Amends FR-3 & adds FR-15 (Terminal multiplexer interop).
- **Architecture Impact:**
  - `src/infra/herdr.rs`: Implement `is_herdr_active()`, `open_worktree_in_herdr(repo_root: &Path, branch: &str)`.
  - `src/infra/mod.rs`: Export `herdr` module.
  - `src/workflow/start.rs`: Call `open_worktree_in_herdr` post-creation.
  - `tests/test_herdr_integration.rs`: Unit tests verifying inactive fallback, temp dir guards, and active invocation.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Clean implementation, safe tests, passing `cargo test`, and installed binary.
