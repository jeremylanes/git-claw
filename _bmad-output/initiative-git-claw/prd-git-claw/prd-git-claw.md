---
title: git-claw
created: 2026-10-05
updated: 2026-10-05
status: draft
---

# PRD: git-claw (Generic Git Worktrees Orchestrator)

## 0. Document Purpose

This Product Requirements Document (PRD) defines the specification, functional requirements, and behavioral boundaries for `git-claw`, a standalone Rust CLI tool that orchestrates concurrent Git worktrees with deterministic slot and port allocation. It builds directly upon the validated [Product Brief & Functional Specification (v1.2.0)](file:///home/lane/.gemini/antigravity-cli/brain/516a3073-49be-458e-bbbf-fac5a2a02243/git-claw-product-brief.md) produced during Discovery. Downstream readers include the Software Architect (`bmad-architecture`) and Development Team (`bmad-build`).

---

## 1. Vision

Modern software engineering increasingly leverages concurrent workflows, whether driven by autonomous AI agents, parallel test runs, or developers juggling multiple simultaneous features. Standard Git worktrees solve workspace concurrency on the filesystem level, but quickly fail in practice due to collisions on local service ports, shared dependencies (`node_modules`, `target`, `.venv`), and container runtime environments (`docker-compose` naming collisions).

`git-claw` is an autonomous, lightweight, and completely tech-agnostic Git Worktree orchestrator written in Rust. It isolates concurrent workspaces by automatically provisioning numeric **Slot IDs**, deterministically calculating collision-free **Port Offsets**, injecting a localized `.env.worktree` configuration, managing build caches, and triggering optional project lifecycle hooks. It operates seamlessly across any stack—from a pure Rust CLI to a multi-container Django/PostgreSQL stack—without imposing any mandatory directory layout or scripting requirements.

---

## 2. Target User

### 2.1 Jobs To Be Done
- **Concurrent Feature Development**: Develop, run, and test multiple feature or bugfix branches simultaneously without shutting down local servers or encountering port clashes (e.g., `Address already in use: 8000`).
- **AI Agent Orchestration**: Enable autonomous AI coding agents to create isolated, disposable worktrees, execute builds and tests independently, and teardown workspaces without conflicting with human developers or concurrent agents.
- **Throwaway Spikes & POCs**: Spin up a sandbox worktree in seconds to test an experimental idea and discard it effortlessly without leaving orphaned branches or dirty state.
- **Standardized Clean Releases**: Enforce clean trunk-based merges and tagged SemVer releases directly against `main`.

### 2.2 Non-Users (v1)
- Teams requiring GUI-based Git management tools (v1 is purely a terminal CLI binary).
- Remote distributed multi-node runners (v1 coordinates workspaces on a single local machine/workstation).

### 2.3 Key User Journeys

- **UJ-1. Developer spins up a second feature alongside a running app.**
  - **Persona & Context**: Alex, full-stack engineer working on `feature/auth` on port 8000. An urgent bug report comes in.
  - **Entry State**: Authenticated terminal in the repository root. `feature/auth` is currently running on Slot 1 (`PORT_WEB=8001`).
  - **Path**: Alex runs `git claw bugfix start login-crash`. `git-claw` scans active slots, allocates Slot 2, creates worktree `bugfix/login-crash`, calculates `PORT_WEB=8002`, writes `.env.worktree`, creates symlinks for shared dependencies, and executes `post_start` if configured.
  - **Climax**: Alex runs `git claw open login-crash`, tests the fix on port 8002 without interrupting `feature/auth`.
  - **Resolution**: Alex runs `git claw finish login-crash`. Tests run via `pre_finish`; the branch is merged into `main`, Slot 2 is released, and the worktree is cleanly removed.
  - **Edge Case**: If tests fail during `pre_finish`, merge is rejected and the worktree remains open for inspection unless `--force` is passed.

- **UJ-2. AI coding agent conducts an isolated spike.**
  - **Persona & Context**: Claude/Antigravity coding agent assigned to research a database migration approach.
  - **Entry State**: Shell execution in repo root.
  - **Path**: Agent runs `git claw spike start pg-vector-eval`. `git-claw` allocates an available slot, creates an isolated worktree outside the standard release pipeline.
  - **Climax**: Agent runs experimental benchmarks inside the worktree using `git claw run pg-vector-eval cargo test`.
  - **Resolution**: Evaluation complete; agent runs `git claw spike drop pg-vector-eval`. The worktree and branch are purged with zero merge onto `main`.

- **UJ-3. Developer audits active slots and cleans orphaned state.**
  - **Persona & Context**: Dev manually deleted a worktree folder with `rm -rf` days ago and wants to inspect active resources.
  - **Entry State**: Terminal prompt in main repo.
  - **Path**: Dev runs `git claw list`.
  - **Climax**: `git-claw` detects the missing directory, performs self-healing garbage collection (releases the Slot ID and purges the stale entry in `.git/claw/slots.json`), and displays a neat dashboard of existing active worktrees and assigned ports.
  - **Resolution**: Slots state is synchronized, clear of dead allocations.

---

## 3. Glossary

- **Worktree**: An additional working directory linked to the primary Git repository created via `git worktree`.
- **Slot ID**: A positive integer (1, 2, 3...) uniquely assigned to an active worktree to prevent port and resource conflicts.
- **Base Port**: The reference port defined in `.git-claw.toml` (e.g., `web = 8000`).
- **Effective Port**: The calculated port for a worktree: `Effective Port = Base Port + Slot ID`.
- **Primary Repository**: The root Git repository holding the `.git/` folder and `.git-claw.toml`.
- **Hook**: An optional arbitrary command declared in `.git-claw.toml` executed at lifecycle milestones (`post_start`, `pre_finish`, `post_finish`).
- **Spike**: A disposable experimental worktree branch designed to be dropped rather than merged into `main`.
- **Slots Registry**: The JSON file stored at `.git/claw/slots.json` tracking active slots, branch names, worktree paths, and allocation timestamps.

---

## 4. Features

### 4.1 Configuration & Project Setup
**Description:** `git-claw` reads `.git-claw.toml` at the project root to discover project configuration, base port offsets, optional lifecycle hooks, and cache sharing rules. If `.git-claw.toml` is omitted, defaults are applied gracefully.

#### FR-1: Configuration Parsing
The system reads `.git-claw.toml` and extracts `[project]`, `[ports]`, `[hooks]`, and `[cache]` tables.
**Consequences (testable):**
- Missing `.git-claw.toml` uses default configuration (`main_branch = "main"`, default worktree directory `../<repo>-worktrees/`, empty ports, no hooks, `cache.strategy = "shared"`).
- Invalid TOML syntax triggers an explicit error message with line numbers and exits with code 1 without altering files.

#### FR-2: Slot Registry Persistence & Self-Healing
The system maintains the active slot allocations in `.git/claw/slots.json`.
**Consequences (testable):**
- Slot registry is created automatically if not present.
- When querying or allocating slots, if any registered worktree directory does not exist on disk, the system automatically frees the Slot ID and cleans the entry from `.git/claw/slots.json`.

---

### 4.2 Worktree Lifecycle Management
**Description:** Creates, isolates, lists, and cleans up worktrees for `feature`, `bugfix`, `hotfix`, and `spike` branches. Realizes UJ-1, UJ-2, UJ-3.

#### FR-3: Worktree Creation (`git claw <type> start <name>`)
Allows creating a new worktree for branch types `feature/`, `bugfix/`, `hotfix/`, and `spike/`.
**Consequences (testable):**
- Allocates the lowest available positive integer Slot ID (reuses freed slots).
- Invokes `git worktree add` targeting `{worktree_root}/{name}` with branch `{type}/{name}` branched off `main` (or latest commit).
- Writes `.env.worktree` inside the created worktree containing `CLAW_SLOT_ID`, `CLAW_WORKTREE_NAME`, `CLAW_WORKTREE_PATH`, `COMPOSE_PROJECT_NAME`, and all computed `PORT_<KEY>` variables.
- When `cache.strategy = "shared"`, creates symlinks in the worktree pointing to declared directories in the primary repository.
- If `--isolated` is passed, symlinking is skipped.
- Executes `hooks.post_start` if defined in `.git-claw.toml`. If the hook fails, the system outputs a non-fatal warning with exit status, retaining the created worktree.

#### FR-4: Worktree Completion & Merge (`git claw finish [name]`)
Completes a worktree, runs test hooks, merges changes to the main branch, and removes worktree assets. Realizes UJ-1.
**Consequences (testable):**
- Executes `hooks.pre_finish` if defined. If the command exits with non-zero status, merge and cleanup are aborted immediately, unless `--force` is supplied.
- Merges the branch into `main`.
- Executes `hooks.post_finish` if defined.
- Removes the worktree directory via `git worktree remove` and deletes the local branch.
- Releases the Slot ID in `.git/claw/slots.json`.

#### FR-5: Spike Abandonment (`git claw spike drop <name>`)
Removes a spike worktree without merging into `main`. Realizes UJ-2.
**Consequences (testable):**
- Removes the worktree directory and purges the `spike/<name>` branch.
- Releases the Slot ID in `.git/claw/slots.json`.
- Does not trigger `pre_finish` or `post_finish` hooks.

---

### 4.3 Dashboard & Utility Execution
**Description:** Provides visibility and operational ergonomics over running worktrees. Realizes UJ-3.

#### FR-6: Status Dashboard (`git claw list`)
Displays an ASCII/colorized table listing all active worktrees.
**Consequences (testable):**
- Columns: Slot ID, Branch Name, Worktree Directory, Assigned Ports, and Head Commit.
- Automatically purges orphaned entries before rendering.

#### FR-7: Contextual Execution (`git claw run [name] <command...>`)
Executes an arbitrary shell command within the working directory of the specified worktree with `.env.worktree` loaded into environment variables.
**Consequences (testable):**
- Command executes with exit code forwarded verbatim to caller.
- Omitting `[name]` infers current worktree if invoked from inside one.

#### FR-8: Editor & Shell Integration (`open` and `cd`)
- `git claw open [name]`: Launches the configured editor (`$EDITOR` or `code`/`cursor`) targeting the worktree directory.
- `git claw cd [name]`: Prints the worktree path (usable with shell aliases: `cd $(git claw cd <name>)`).

---

### 4.4 Release Tagging
**Description:** Manages SemVer releases on the main trunk.

#### FR-9: SemVer Tag Creation (`git claw tag <semver>`)
Validates SemVer format (e.g. `v1.2.0`), checks that `main` is clean, and creates an annotated tag.
**Consequences (testable):**
- Rejects non-SemVer formats with a descriptive error.
- Rejects execution if uncommitted changes exist on `main`.

---

## 5. Non-Goals (Explicit)

- **Not a Container Manager**: `git-claw` does not manage Docker lifecycles or daemon health; it simply calculates ports and invokes optional hooks configured by the user.
- **Not a Git Hosting or Forge Client**: `git-claw` does not interact directly with GitHub/GitLab APIs, create Pull Requests, or manage CI/CD runs.
- **Not a Background Daemon**: `git-claw` runs strictly as a short-lived synchronous CLI binary. It holds no long-running process, background thread, or socket listener.
- **No Mandatory Filesystem Opinion**: `git-claw` will never mandate files in `bin/`, `scripts/`, or standard template names. All integration is declarative via `.git-claw.toml`.

---

## 6. MVP Scope

### 6.1 In Scope (v0.1.0)
- Single compiled Rust binary `git-claw`.
- Configuration parsing (`.git-claw.toml`) and default fallback.
- Slot allocation engine (1..N) and collision-free port offset calculation.
- Generation of `.env.worktree`.
- Commands: `start` (`feature`, `bugfix`, `hotfix`, `spike`), `finish`, `spike drop`, `list`, `run`.
- Optional hooks: `post_start`, `pre_finish`, `post_finish`.
- Shared cache symlinks (`strategy = "shared"`).
- Auto-GC of orphaned slots during status inspection.

### 6.2 Out of Scope for MVP
- Shell completion scripts (`bash`, `zsh`, `fish`) — deferred to v0.2.0.
- Interactive TUI dashboard (`ratatui`) — deferred to v0.3.0.
- Automatic Git worktree repair across multiple Git worktree storage corruptions.

---

## 7. Success Metrics

### Primary
- **SM-1 (Allocation Latency)**: Creation of a new worktree (`git claw feature start`) completes in < 500ms (excluding user hook execution). Validates FR-1, FR-2, FR-3.
- **SM-2 (Zero Port Collision)**: 100% deterministic port separation across up to 50 concurrent worktrees. Validates FR-3, FR-6.

### Counter-Metrics (Do not optimize)
- **SM-C1 (Binary Portability over Feature Creep)**: Do not link dynamic C libraries (like heavy libgit2 bindings if `std::process::Command("git")` suffices) to keep the binary statically linkable and under 15MB.

---

## 8. Open Questions

1. *Shell function for `cd`*: Should `git claw` install a shell wrapper (`function claw() { ... }`) to allow native directory switching without subshell eval? *(Deferred to Developer Guide / Addendum)*.

---

## 9. Assumptions Index

- `[ASSUMPTION-1]`: The slot registry is persisted in `.git/claw/slots.json` to remain private to the local checkout and uncommitted to Git.
- `[ASSUMPTION-2]`: If a `post_start` hook fails, `git-claw` issues a warning and keeps the worktree intact for developer inspection.
- `[ASSUMPTION-3]`: If a `pre_finish` hook fails, merge is rejected unless `--force` is specified.
- `[ASSUMPTION-4]`: Cache sharing uses standard POSIX symlinks pointing from worktree paths to main repository paths.
- `[ASSUMPTION-5]`: Local Git CLI (`git`) is available on the system PATH and supports worktrees (Git >= 2.20).
