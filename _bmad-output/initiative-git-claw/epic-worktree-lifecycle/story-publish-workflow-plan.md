---
title: 'Remote worktree publish workflow (git claw [feature|bugfix|hotfix] publish [name])'
type: 'feature'
ticket: 24
created: '2026-10-10'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-feature-publish/change-feature-publish.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Developers need a seamless way to push their worktree branches to a remote Git repository (`origin` or custom remote) and configure upstream tracking (`git push -u <remote> <branch>`), matching the ergonomics of `git flow feature publish`.

**Approach:** Implement `workflow::publish::publish_worktree` that supports contextual inference when run inside a worktree, resolves branch names by type (`feature/<name>`, `bugfix/<name>`, `hotfix/<name>`), executes `git push -u <remote> <branch>`, and outputs human-readable confirmation.

## Boundaries & Constraints

**Always:**
- Push with `-u` / `--set-upstream` so that subsequent standard `git push` and `git pull` commands work out-of-the-box.
- Support both type-specific commands (`git claw feature publish [name]`, `git claw bugfix publish [name]`, `git claw hotfix publish [name]`) and shorthand (`git claw publish [name]`).
- Infer branch name and worktree from CWD when `name` is omitted.
- Default to remote `origin` unless `--remote <name>` is passed.

**Never:**
- Do not modify or force push (`--force` must not be used).
- Do not release slot ID or alter local registry during publish (publishing is non-destructive).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Publish active feature worktree | CWD in `core-review`, no name | Pushes `feature/core-review` to `origin` with upstream tracking | Ok(()) |
| Publish by explicit name | Outside worktree, `name = "login"` | Pushes `feature/login` to `origin` | Ok(()) |
| Explicit remote | `--remote upstream` | Pushes to specified remote `upstream` | Ok(()) |
| Unknown worktree | Name not in registry | Returns clear error | Err(WorkflowError::WorktreeNotFound) |
| Non-existing remote / network failure | Git push fails | Returns Git error with captured stderr | Err(GitError::CommandFailed) |

</frozen-after-approval>

## Code Map

- `src/cli/args.rs`: Add `Publish` variant to `WorktreeAction` and `Commands::Publish`.
- `src/infra/git.rs`: Add `git_push_upstream` helper.
- `src/workflow/publish.rs`: Main publish orchestration logic.
- `src/workflow/mod.rs`: Export `publish_worktree`.
- `src/main.rs`: Wire CLI handlers.
- `tests/test_publish_workflow.rs`: Integration tests with a local bare remote.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/git.rs` -- Implement `git_push_upstream(repo_root, remote, branch)`.
- [x] `src/cli/args.rs` -- Add `Publish` to `WorktreeAction` and `Commands::Publish`.
- [x] `src/workflow/publish.rs` -- Implement `publish_worktree(options: PublishOptions)`.
- [x] `src/workflow/mod.rs` & `src/main.rs` -- Wire and export `publish_worktree`.
- [x] `tests/test_publish_workflow.rs` -- Create integration test suite using a local bare repository as remote.

**Acceptance Criteria:**
- Given a feature worktree with commits and an upstream remote, running `git claw feature publish` pushes the branch and sets tracking.
- All unit, integration, and doc-tests pass (`cargo test`).
- Binary successfully compiles and installs to `~/.cargo/bin/git-claw`.
