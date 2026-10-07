---
tracker_id: ""
type: epic
title: "Developer Experience, Docker Interop & Project Initialization"
parent: initiative-git-claw
covers: [FR-10, FR-11, FR-12]
after: ["epic-inspection-and-tooling"]
assignee: ""
risk: medium
---

# Developer Experience, Docker Interop & Project Initialization

## Description

Extends `git-claw` with real-world developer experience enhancements: global configuration hierarchy (`~/.git-claw/config.toml`), centralized worktree placement (`~/.git-claw/worktrees/<repo>/<name>`), untracked configuration duplication with in-place port merging (`[files] copy`), non-intrusive Docker Compose override generation for shared services and network interoperability (`[docker]`), interactive stack scanning via `git claw init`, and terminal shell integration (`git claw shell-hook`).

## Outcome

Developers can run `git claw init` in any existing repository (such as `tune`) to automatically detect environment files and exposed ports. When starting a worktree, untracked `.env` files are duplicated and populated with collision-free ports, `docker-compose.claw.override.yml` is generated to prevent container/network name collisions and attach to shared infrastructure (like an existing PostgreSQL database), and users can automatically `cd` into the new worktree.

## Requirements

The requirements are defined in `prd-git-claw.md`:
- FR-1: Two-Tier Configuration & `~/.git-claw/worktrees/` defaults
- FR-8: Navigation & Shell Integration (`shell-hook`)
- FR-10: Project Initialization (`git claw init`)
- FR-11: Untracked Files Duplication & Dynamic Variable Merging (`[files] copy`)
- FR-12: Docker Compose Override & Service Sharing (`[docker]`)

## Done when

1. Default worktree path resolves to `~/.git-claw/worktrees/<repo-name>/<worktree-name>` unless overridden by `~/.git-claw/config.toml` or `.git-claw.toml`.
2. `[files] copy = ["path/to/.env"]` copies files into newly created worktrees and updates configured port variables in-place with effective values.
3. `[docker]` configuration generates `docker-compose.claw.override.yml`, setting shared networks to `external: true` and eliminating duplicate container names.
4. `git claw init` (and `git claw init --yes`) inspects the current repository, detects `.env` and compose files, and generates a valid `.git-claw.toml`.
5. `git claw shell-hook` outputs shell wrapper script for bash/zsh enabling automatic `cd` into the created worktree on `git claw feature start <name>`.
6. Full test suite passes without regressions.

## References

- parent — _bmad-output/initiative-git-claw/initiative-git-claw.md
- change — _bmad-output/initiative-git-claw/change-runtime-fixes-and-init/change-runtime-fixes-and-init.md
- prd — _bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md
