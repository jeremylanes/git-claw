---
tracker_id: ""
type: epic
title: "Worktree Lifecycle Orchestration & Workspace Isolation"
parent: initiative-git-claw
covers: [FR-3, FR-4, FR-5]
after: []
assignee: ""
risk: high
---

# Worktree Lifecycle Orchestration & Workspace Isolation

## Description

Implements the core worktree workflows (`start`, `finish`, `spike drop`), managing Git worktree creation and removal, branch lifecycle (`feature/`, `bugfix/`, `hotfix/`, `spike/`), deterministic environment variable injection (`.env.worktree`), POSIX symlink cache sharing, and lifecycle hook execution (`post_start`, `pre_finish`, `post_finish`).

## Outcome

Developers and AI coding agents can spin up isolated worktrees in under 500ms with fully populated environment configuration and shared caches, run lifecycle hooks, cleanly merge completed work into the main trunk, or discard experimental spikes with zero residual branches or slot leaks.

## Requirements

The requirements are defined in `prd-git-claw.md`:
- FR-3: Worktree Creation (`git claw <type> start <name>` with slot allocation, `.env.worktree`, symlinks, `post_start`)
- FR-4: Worktree Completion & Merge (`git claw finish [name]` with `pre_finish`, merge to `main`, cleanup, and slot release)
- FR-5: Spike Abandonment (`git claw spike drop <name>` with branch purge, cleanup, and slot release)

## Done when

1. `git claw <type> start <name>` provisions a worktree with lowest available Slot ID, generates `.env.worktree` (with `CLAW_SLOT_ID`, `COMPOSE_PROJECT_NAME`, and effective ports), creates relative POSIX symlinks for declared cache directories, and triggers `post_start` (AD-5, AD-7, AD-8).
2. `git claw finish [name]` evaluates `pre_finish` (aborting on error unless `--force`), verifies clean trunk, merges branch into `main`, removes worktree directory via `git worktree remove --force`, deletes local branch, and releases Slot ID (AD-6, AD-8).
3. `git claw spike drop <name>` purges the worktree directory and branch via `git branch -D` without merging and releases Slot ID (AD-8).
4. Failure of `post_start` issues a warning without failing worktree creation (AD-6).
5. Integration test suites (`test_start_finish.rs`, `test_hooks.rs`) validate end-to-end lifecycle operations against realistic Git repositories.

## Boundaries

Worktree and branch lifecycle orchestration, hook execution, environment file writing, and symlink creation. Does not implement the status dashboard (`list`), contextual command execution (`run`), or release tagging (`tag`).

## References

- parent — _bmad-output/initiative-git-claw/initiative-git-claw.md
- prd — _bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md, sections 4.2 (FR-3, FR-4, FR-5)
- architecture — _bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md, sections AD-1, AD-3, AD-5, AD-6, AD-7, AD-8, AD-9

## Notes

- Waits on epic-scaffold-and-core-engine because: needs Config model, SlotRegistry, advisory file lock, port arithmetic, and base Git runner.
- Decision: Worktree finish sequence enforces pre_finish -> clean trunk check -> merge -> worktree remove -> branch delete -> slot release -> post_finish (2026-10-05).
- Decision: Hook failures in post_start and post_finish are non-fatal warnings (2026-10-05).
