# Sprint Change Proposal: Remote Collaboration & Worktree Branch Publishing

**Date:** 2026-10-10  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User requested Git Flow-style remote publishing (`git claw feature publish [name]`) to push branches to remote Git repositories with upstream tracking.

---

## 1. Issue Summary

`git-claw` provides complete local worktree orchestration (allocation, creation, in-place environment provisioning, Docker Compose isolation, and cleanup). However, developers working in collaborative or CI/CD environments need a direct, native way to publish their active feature, bugfix, or hotfix branches to remote Git remotes (analogous to `git flow feature publish <name>`), setting up upstream tracking in a single friction-free command.

---

## 2. Requirement Specification

1. **CLI Commands:**
   - `git claw feature publish [name]`
   - `git claw bugfix publish [name]`
   - `git claw hotfix publish [name]`
   - Top-level shorthand: `git claw publish [name]`
2. **Contextual Inference:**
   - If `name` is omitted, `publish` must detect the current worktree from the working directory (CWD) and resolve its associated branch from `slots.json` (or the current branch).
   - If `name` is provided, `publish` resolves the target branch for that branch type (`<type>/<name>`).
3. **Execution Semantics:**
   - Verifies the working tree and branch exist.
   - Pushes the branch to the remote repository (`origin` by default, or configured remote in `.git-claw.toml`).
   - Sets upstream tracking (`git push -u <remote> <branch>`).
   - Emits a clean, colored success confirmation with the remote name and published branch name.
4. **Error Handling:**
   - If no remote is configured in the Git repository, returns a clear error (`WorkflowError::NoRemoteConfigured`).
   - If the push is rejected (e.g. non-fast-forward), forwards Git's error cleanly.
   - If `name` is not found, returns `WorkflowError::WorktreeNotFound`.

---

## 3. Impact Analysis

- **PRD Impact:** Adds FR-13 (Remote branch publishing with upstream tracking).
- **Architecture Impact:**
  - `src/cli/args.rs`: Updates `WorktreeAction` to include `Publish { name: Option<String>, remote: Option<String> }` and adds top-level `Commands::Publish`.
  - `src/infra/git.rs`: Adds `git_push_upstream(repo_root: &Path, remote: &str, branch: &str) -> Result<(), GitError>`.
  - `src/workflow/publish.rs`: New workflow module `publish_worktree(options: PublishOptions) -> Result<(), WorkflowError>`.
  - `src/workflow/mod.rs`: Re-exports `publish_worktree`.
  - `src/main.rs`: Wires CLI arguments to `publish_worktree`.
  - `tests/test_publish_workflow.rs`: Integration tests for publishing from inside worktree and by explicit name.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Developer Subagent 💻).
- **Deliverables:** Implementation in `src/`, integration tests in `tests/test_publish_workflow.rs`, verification with `cargo check` and `cargo test`, and binary installation via `cargo install --path .`.
