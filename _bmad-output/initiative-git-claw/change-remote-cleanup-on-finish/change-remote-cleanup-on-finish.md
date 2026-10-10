# Sprint Change Proposal: Remote Branch Teardown on Finish

**Date:** 2026-10-10  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request to match Git Flow behavior: `git claw finish` should clean up published remote branches (`git push origin --delete <branch>`) when completing a feature, bugfix, or hotfix.

---

## 1. Issue Summary

Currently, `git claw finish` completes the local lifecycle:
1. Validates `pre_finish` hooks.
2. Merges the branch into `main`.
3. Removes the worktree directory on disk.
4. Deletes the local branch (`git branch -d`).
5. Releases the slot in `slots.json`.
6. Executes `post_finish` hooks.

However, if the worktree was published to a remote via `git claw [type] publish` (or manual `git push`), the remote branch remains on the remote Git server indefinitely. In standard Git Flow (`git flow feature finish`), published remote branches are automatically purged from the remote upon completion.

---

## 2. Requirement Specification

1. **Automatic Remote Teardown:**
   - In `workflow::finish`, after merging and deleting the local branch, check if the branch exists on the remote repository (`origin` by default or configured remote).
   - If present on remote, execute `git push <remote> --delete <branch>`.
   - Emit a confirmation message (`Remote branch '<branch>' deleted from '<remote>'`).
2. **Opt-Out Safety Flag:**
   - Add `--keep-remote` (short `-k`) to `git claw finish`:
     ```bash
     git claw finish [name] --keep-remote
     ```
   - When `--keep-remote` is specified, the remote branch is preserved.
3. **Resilience / Non-Fatal Remote Error:**
   - If the remote is unreachable (e.g. offline/network failure), emit a warning without failing the local merge and worktree teardown.

---

## 3. Impact Analysis

- **PRD Impact:** Amends FR-4 & FR-13 to include remote branch lifecycle cleanup.
- **Architecture Impact:**
  - `src/infra/git.rs`: Add `git_remote_branch_exists` and `git_delete_remote_branch`.
  - `src/cli/args.rs`: Add `keep_remote: bool` (`--keep-remote` / `-k`) to `Commands::Finish`.
  - `src/workflow/finish.rs`: Update `FinishOptions` and inject remote branch teardown step.
  - `src/main.rs`: Wire `--keep-remote` argument.
  - `tests/test_finish_workflow.rs`: Add integration tests covering remote branch deletion and `--keep-remote`.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Complete implementation, integration tests against local bare git remote, `cargo test` passing, and binary installation.
