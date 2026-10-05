---
type: initiative
title: "git-claw — Generic Git Worktrees Orchestrator"
parent: none
covers: [FR-1, FR-2, FR-3, FR-4, FR-5, FR-6, FR-7, FR-8, FR-9]
after: []
assignee: ""
risk: medium
---

# git-claw — Generic Git Worktrees Orchestrator

## Description

`git-claw` is an autonomous, lightweight, and tech-agnostic Rust CLI binary that orchestrates concurrent Git worktrees with deterministic numeric Slot IDs, collision-free port offsets, automated `.env.worktree` configuration injection, POSIX symlink cache sharing, and lifecycle hooks. The specification and architecture spine are defined in the PRD and Architecture documents; this initiative delivers the complete v0.1.0 MVP release.

## Outcome

Developers and autonomous AI coding agents can concurrently develop, run, and test multiple features, bugfixes, and disposable spikes on a single workstation with zero port collisions and zero manual setup overhead, achieving sub-500ms workspace creation latency and 100% deterministic resource isolation.

## Requirements

The requirements are defined in the referenced PRD (`prd-git-claw.md`):
- FR-1: Configuration Parsing (`.git-claw.toml` and defaults)
- FR-2: Slot Registry Persistence & Self-Healing (`.git/claw/slots.json` with advisory file locking and auto-GC)
- FR-3: Worktree Creation (`git claw <type> start <name>` with slot allocation, `.env.worktree`, symlinks, `post_start`)
- FR-4: Worktree Completion & Merge (`git claw finish [name]` with `pre_finish`, merge to `main`, cleanup, and slot release)
- FR-5: Spike Abandonment (`git claw spike drop <name>` with branch purge, cleanup, and slot release)
- FR-6: Status Dashboard (`git claw list` with orphaned slot auto-GC)
- FR-7: Contextual Execution (`git claw run [name] <command...>`)
- FR-8: Editor & Shell Integration (`git claw open [name]`, `git claw cd [name]`)
- FR-9: SemVer Tag Creation (`git claw tag <semver>`)

## Done when

1. `git-claw` (and `git claw`) compiles and executes as a single binary on Linux and macOS with zero external daemon dependencies.
2. Concurrent worktrees across `feature/`, `bugfix/`, `hotfix/`, and `spike/` branches can be created, isolated, inspected, and torn down with deterministic slot IDs and port assignments.
3. Advisory file locking prevents race conditions and corrupted registry state under concurrent multi-agent executions.
4. Worktree completion cleanly executes `pre_finish` gates, merges into `main`, and prunes worktree directory and local branch.
5. Orphaned worktrees created by manual deletion are automatically detected and cleaned up on inspection.
6. SemVer releases can be validated and tagged on a clean trunk.

## Boundaries

Standalone CLI tool operating within a single local Git repository and its sibling worktrees. Git >= 2.20 must be installed. Non-goals: Docker daemon management, Git forge/PR integration, remote multi-host execution, background daemons. Tracer path: spin up a feature worktree, verify calculated port and `.env.worktree`, execute a command via `run`, and cleanly finish by merging into `main`.

- Touch point: system Git CLI (`git`) — invoked via `std::process::Command` for core detection, `rev-parse`, and `prune`; owner: epic-scaffold-and-core-engine
- Touch point: external editor (`$EDITOR`, `code`, `cursor`) — launched as subprocess for worktree opening; owner: epic-inspection-and-tooling

## References

- prd — _bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md
- architecture — _bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md
- addendum — _bmad-output/initiative-git-claw/prd-git-claw/addendum.md

## Notes

- Parked: Shell completion scripts (`bash`, `zsh`, `fish`) deferred to v0.2.0.
- Parked: Interactive TUI dashboard (`ratatui`) deferred to v0.3.0.
- Parked: Shell wrapper auto-installer (`function claw()`) deferred to v0.2.0; manual usage documented in v0.1.0.
- Decision: System `git` CLI invocation via `std::process::Command` instead of `libgit2` bindings (2026-10-05).
- Decision: Advisory file lock on `slots.lock` for atomic registry mutations (2026-10-05).
- Decision: Cache sharing via relative POSIX symlinks (2026-10-05).
- Decision: Registry persistence in `<git-common-dir>/claw/slots.json` (2026-10-05).
- Decision: GitHub Actions multi-OS CI included in opening platform-baseline epic (2026-10-05).
- Decision: git claw cd prints absolute directory path to stdout (2026-10-05).
