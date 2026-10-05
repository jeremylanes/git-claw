---
tracker_id: ""
type: epic
title: "Dashboard, Contextual Execution & Release Management"
parent: initiative-git-claw
covers: [FR-6, FR-7, FR-8, FR-9]
after: []
assignee: ""
risk: medium
---

# Dashboard, Contextual Execution & Release Management

## Description

Implements operational inspection, execution helpers, and release workflows: the `git claw list` dashboard with automated self-healing of orphaned slots, contextual subprocess execution with `.env.worktree` environment loading (`git claw run`), editor opening (`open`), directory navigation (`cd`), and SemVer release validation and tagging (`tag`).

## Outcome

Users and agents benefit from complete visibility into active worktrees and ports, frictionless in-workspace command execution without manual environment sourcing, instant shell/editor navigation, and foolproof trunk-based SemVer release tagging.

## Requirements

The requirements are defined in `prd-git-claw.md`:
- FR-6: Status Dashboard (`git claw list` with orphaned slot auto-GC)
- FR-7: Contextual Execution (`git claw run [name] <command...>`)
- FR-8: Editor & Shell Integration (`git claw open [name]`, `git claw cd [name]`)
- FR-9: SemVer Tag Creation (`git claw tag <semver>`)

## Done when

1. `git claw list` displays an ANSI-formatted table (Slot ID, Branch, Path, Ports, Head Commit) and automatically detects and purges orphaned worktree directories from `slots.json` (AD-4).
2. `git claw run [name] <cmd...>` resolves target worktree, injects `.env.worktree` into the subprocess environment, and forwards the exit code verbatim.
3. `git claw open [name]` launches configured `$EDITOR` (or `code`/`cursor`) targeting the worktree directory.
4. `git claw cd [name]` outputs the absolute worktree path to stdout.
5. `git claw tag <semver>` validates SemVer syntax, verifies the `main` branch is clean, and creates an annotated Git tag.
6. Integration tests (`test_self_healing.rs`, CLI execution tests) pass cleanly.

## Boundaries

Inspection, contextual command execution, navigation helpers, and trunk tagging. Does not modify worktree creation or merge logic.

- Touch point: external editor (`$EDITOR`, `code`, `cursor`) — launched via subprocess; owner: epic-inspection-and-tooling

## References

- parent — _bmad-output/initiative-git-claw/initiative-git-claw.md
- prd — _bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md, sections 4.3 (FR-6, FR-7, FR-8), 4.4 (FR-9)
- architecture — _bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md, sections AD-1, AD-4, AD-5, AD-9

## Notes

- Waits on epic-worktree-lifecycle because: requires active worktrees, `.env.worktree` format, and worktree state resolution.
- Decision: git claw cd prints absolute directory path to stdout (2026-10-05).
- Parked: Interactive TUI dashboard (`ratatui`) deferred to v0.3.0.
- Parked: Shell wrapper auto-installer (`function claw()`) deferred to v0.2.0.
