# Sprint Change Proposal: Primary Repository Root Resolution from Worktree Context

**Date:** 2026-10-18  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User bug report in project `tune`: Running `git claw finish <name>` from inside a worktree directory fails with `error: Git command failed (git checkout main): fatal: 'main' is already used by worktree at '/path/to/primary/repo'`.

---

## 1. Issue Summary

In Git Worktree architecture, two worktrees can never checkout the same branch concurrently. The primary trunk branch (`main`) is checked out in the primary repository root.

Currently, `git-claw` workflows invoke `resolve_toplevel`, which calls `git rev-parse --show-toplevel`. When executed inside a linked worktree directory, `git rev-parse --show-toplevel` returns the linked worktree directory itself rather than the primary repository. Consequently, `git claw finish` attempts to run `git checkout main` and `git merge` inside the linked worktree instead of the primary repository, causing Git to fail with:
`fatal: 'main' is already used by worktree at '<primary-repo>'`.

Additionally, other commands (`start`, `list`, `spike drop`, `tag`) run from within a linked worktree can misinterpret the active worktree as the repository root, leading to mislocated `.git-claw.toml` loading or inconsistent path resolutions.

---

## 2. Requirement Specification

1. **Primary Repository Root Resolution:**
   - In `src/infra/git.rs`, implement `resolve_primary_repo_root(repo_root: Option<&Path>) -> Result<PathBuf, GitError>`.
   - Query `git rev-parse --git-common-dir`.
   - In standard Git repositories and linked worktrees, `--git-common-dir` always resolves to `<primary-repo-root>/.git` (or the bare root).
   - If the canonicalized common directory's file name is `.git`, its parent directory is the canonical primary repository root.
   - If not, fall back cleanly to `resolve_toplevel`.

2. **Workflow Updates:**
   - In `src/workflow/finish.rs`: Resolve `primary_repo_root`. All merge operations (`git_is_clean`, `git_checkout main`, `git_merge`, `remove_worktree`, `delete_branch`) must execute in the primary repository context.
   - In `src/workflow/start.rs`, `src/workflow/spike.rs`, `src/workflow/list.rs`, `src/workflow/run.rs`, `src/workflow/nav.rs`, `src/workflow/tag.rs`, `src/workflow/publish.rs`: Use `resolve_primary_repo_root` to ensure consistent repository name and configuration loading regardless of current working directory.

3. **Verification & Testing:**
   - Add integration tests verifying that `finish_worktree` succeeds when executed with `repo_root` pointing to a linked worktree directory.
   - Ensure all existing tests pass and coding conventions are strictly preserved.

---

## 3. Impact Analysis

- **Architecture Impact:**
  - `src/infra/git.rs`: Add `resolve_primary_repo_root`.
  - `src/infra/mod.rs`: Export `resolve_primary_repo_root`.
  - `src/workflow/*`: Adopt `resolve_primary_repo_root`.
  - `tests/test_finish_workflow.rs`: Add worktree-context finish tests.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Clean implementation, full test suite passing, Sphinx documentation with examples, and updated binary installed.
