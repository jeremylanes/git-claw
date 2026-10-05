---
tracker_id: ""
type: epic
title: "Project Scaffold, Configuration & Core Slot Allocation Engine"
parent: initiative-git-claw
covers: [FR-1, FR-2]
after: []
assignee: ""
risk: medium
---

# Project Scaffold, Configuration & Core Slot Allocation Engine

## Description

Initializes the Rust codebase (`git-claw`), establishing the layered architecture, Clap CLI presentation scaffold, zero-config configuration parsing, pure domain models for slot allocation and port calculation, atomic slot registry persistence in the shared Git directory with advisory file locking, base Git CLI runner, shared test fixtures, and GitHub Actions CI workflow.

## Outcome

The project builds and packages a functional, statically-linkable Rust CLI binary that can parse configuration files (or apply zero-config defaults) and reliably allocate, lock, and persist collision-free numeric Slot IDs in `.git/claw/slots.json` under concurrent executions.

## Requirements

The requirements are defined in `prd-git-claw.md`:
- FR-1: Configuration Parsing (`.git-claw.toml` and defaults)
- FR-2: Slot Registry Persistence & Self-Healing (`.git/claw/slots.json` with advisory file locking and auto-GC)

## Done when

1. `cargo build --release` compiles without warnings, produces the `git-claw` binary, and `--help`/`--version` execute cleanly.
2. Configuration engine successfully parses valid `.git-claw.toml` files and falls back to deterministic defaults when absent (AD-2).
3. Slot allocation engine allocates lowest available positive integers (1..N) and port arithmetic calculates `PORT_<KEY> = Base Port + Slot ID` (AD-5).
4. Advisory file locking (`slots.lock`) with retry polling and timeout protects all registry read-modify-write operations against race conditions, verified by multi-thread stress tests (`test_concurrency.rs`) (AD-3).
5. Registry paths resolve correctly via `git rev-parse --git-common-dir` and `git rev-parse --show-toplevel` (AD-9).
6. Multi-OS CI workflow (`.github/workflows/ci.yml`) runs tests, clippy, and rustfmt on Linux and macOS.

## Boundaries

Scaffold, domain layer (`core/`), advisory locking (`infra/lock.rs`), registry persistence (`infra/registry.rs`), baseline Git detection and pruning (`infra/git.rs`), and presentation skeleton (`cli/args.rs`, `main.rs`). Does not implement worktree lifecycle commands (`start`, `finish`, `spike`) or terminal formatting.

## References

- parent — _bmad-output/initiative-git-claw/initiative-git-claw.md
- prd — _bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md, sections 4.1 (FR-1, FR-2)
- architecture — _bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md, sections AD-1, AD-2, AD-3, AD-4, AD-9

## Notes

- Decision: Advisory file lock on `slots.lock` with non-blocking retry polling (50ms interval, 5s timeout) (2026-10-05).
- Decision: GitHub Actions multi-OS CI included in opening platform-baseline epic (2026-10-05).
