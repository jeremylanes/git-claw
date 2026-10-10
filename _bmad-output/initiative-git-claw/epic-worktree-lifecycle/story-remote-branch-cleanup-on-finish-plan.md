---
title: 'Remote branch teardown on finish (git claw finish [--keep-remote])'
type: 'feature'
ticket: 9
created: '2026-10-10'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-remote-cleanup-on-finish/change-remote-cleanup-on-finish.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** When a worktree branch has been published to a remote (e.g. `origin`), completing it with `git claw finish` only deletes the local branch and worktree directory, leaving stale remote branches on the remote repository.

**Approach:** Extend `workflow::finish` to inspect remote branches on `origin` (or configured remote). If the branch exists on remote and `--keep-remote` is not passed, delete it via `git push <remote> --delete <branch>`, matching Git Flow ergonomics.

## Boundaries & Constraints

**Always:**
- Check if remote branch exists before attempting deletion.
- Honor `--keep-remote` / `-k` to allow users to preserve remote branches when desired.
- Keep local merge and worktree cleanup successful even if remote deletion fails due to network outage (warn gracefully).

**Never:**
- Do not delete remote `main` or base branch.
- Do not fail the local finish workflow if the remote repository is temporarily offline.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Finish published worktree | Published `feature/login` on `origin` | Merges locally, deletes worktree, deletes local branch, deletes `origin/feature/login` | Ok(()) |
| Finish with `--keep-remote` | `--keep-remote` passed | Merges locally, deletes local worktree & branch, keeps `origin/feature/login` | Ok(()) |
| Finish unpublished worktree | Branch only exists locally | Merges locally, deletes worktree & local branch, skips remote deletion cleanly | Ok(()) |
| Remote network unreachable | Remote exists but push fails | Warns about remote deletion failure, completes local cleanup | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/cli/args.rs`: Add `keep_remote: bool` to `Commands::Finish`.
- `src/infra/git.rs`: Add `git_remote_branch_exists` and `git_delete_remote_branch`.
- `src/workflow/finish.rs`: Update `FinishOptions` and add remote branch teardown step.
- `src/main.rs`: Wire `--keep-remote` flag.
- `tests/test_finish_workflow.rs`: Integration tests for remote branch deletion.

## Tasks & Acceptance

**Execution:**
- [ ] `src/infra/git.rs` -- Implement `git_remote_branch_exists` and `git_delete_remote_branch`.
- [ ] `src/cli/args.rs` -- Add `--keep-remote` (`-k`) flag to `Commands::Finish`.
- [ ] `src/workflow/finish.rs` -- Add remote branch teardown step with non-fatal warning on network error.
- [ ] `src/main.rs` -- Wire `keep_remote` argument in CLI execution.
- [ ] `tests/test_finish_workflow.rs` -- Add integration tests with bare remote verifying branch deletion and `--keep-remote`.

**Acceptance Criteria:**
- Given a feature worktree pushed to a remote, running `git claw finish` merges changes and deletes both local and remote branches.
- Passing `--keep-remote` retains the branch on the remote.
- All tests pass with `cargo test`.
