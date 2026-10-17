# git-claw Cheatsheet 🦀

> An interactive, practical guide to orchestrating Git Worktrees with `git-claw`, inspired by the Git Flow cheatsheet.

---

## Index

- [About & Core Concepts](#about--core-concepts)
- [Setup & Installation](#setup--installation)
- [Getting Started](#getting-started)
- [Features](#features)
- [Bugfixes & Hotfixes](#bugfixes--hotfixes)
- [Spikes (Disposable POCs)](#spikes-disposable-pocs)
- [Workspace & Inspection](#workspace--inspection)
- [Releases (SemVer & Changelog)](#releases-semver--changelog)
- [Commands Quick Reference](#commands-quick-reference)

---

## About & Core Concepts

Unlike traditional Git workflows that swap branches in-place inside a single directory (`git checkout`), `git-claw` allocates an **isolated physical workspace (Worktree)** for every branch.

```text
[Main Repository] (main)
   ├── ../my-app-worktrees/auth-jwt     (feature/auth-jwt)     → Slot 1 (Web: 8001, DB: 5433)
   ├── ../my-app-worktrees/fix-session  (bugfix/fix-session)   → Slot 2 (Web: 8002, DB: 5434)
   └── ../my-app-worktrees/poc-redis    (spike/poc-redis)      → Slot 3 (Web: 8003, DB: 5435)
```

- **Deterministic Slot IDs**: Every active worktree is assigned a sequential number (`1, 2, 3...`).
- **Collision-Free Port Offsets**: `Effective Port = Base Port + Slot ID`. No more `Address already in use`.
- **Environment Injection**: Every worktree receives a generated `.env.worktree` ready to use.
- **Shared Build Caches**: Dependencies (`node_modules`, `target`, `.venv`) are symlinked transparently.
- **Auto-Healing GC**: Manually deleted folders are automatically detected and their slots reclaimed.

---

## Setup & Installation

### Cargo Installation

| Platform | Command |
|---|---|
| **Crates.io** | `cargo install git-claw` |
| **From Source** | `cargo install --path .` |

### Shell Auto-Completion

| Shell | Setup Command |
|---|---|
| **Fish** | `mkdir -p ~/.config/fish/completions && git claw completion fish > ~/.config/fish/completions/git-claw.fish` |
| **Bash** | `mkdir -p ~/.local/share/bash-completion/completions && git claw completion bash > ~/.local/share/bash-completion/completions/git-claw` |
| **Zsh** | `mkdir -p ~/.zfunc && git claw completion zsh > ~/.zfunc/_git-claw` |

### Shell Navigation Helper
Add this helper to `~/.bashrc` or `~/.zshrc` to navigate into worktrees with `claw cd`:
```bash
claw() {
    if [ "$1" = "cd" ]; then
        local target
        target=$(git-claw cd "$2") && cd "$target"
    else
        git-claw "$@"
    fi
}
```

---

## Getting Started

### Initialize in an existing repository
`git-claw` requires **zero mandatory configuration**. You can start using it immediately:

```bash
git claw feature start my-first-feature
```

### Guided Configuration (`git claw init`)
To customize base ports, lifecycle hooks, and cache sharing:

| Action | Details | Command |
|---|---|---|
| **Interactive Init** | Analyzes project stack and prompts for ports, branch name, and hooks | `git claw init` |
| **Non-Interactive** | Accepts all inferred defaults automatically | `git claw init --yes` |

---

## Features

Feature branches are prefixed with `feature/` and branched off `main`.

```text
(main) ───► git claw feature start oauth ───► [Worktree feature/oauth]
                                                      │
                                            (Develop & Test in isolation)
                                                      │
(main) ◄─── git claw finish oauth ◄───────────────────┘
```

### Start a new feature
Development of new features branches off `main`.

| Action | Description | Command |
|---|---|---|
| **Start feature** | Creates worktree `../<repo>-worktrees/auth-flow`, branch `feature/auth-flow`, allocates Slot ID, creates `.env.worktree` | `git claw feature start auth-flow` |
| **Start isolated** | Creates worktree without sharing cache symlinks (`node_modules`, `.venv`, etc.) | `git claw feature start auth-flow --isolated` |

### Work inside a feature
Work seamlessly inside your isolated workspace.

| Action | Description | Command |
|---|---|---|
| **Open in editor** | Opens the worktree directory in `$EDITOR`, VS Code, or Cursor | `git claw open auth-flow` |
| **Run command** | Executes a command inside the worktree with `.env.worktree` loaded | `git claw run auth-flow npm run dev` |
| **Navigate directory** | Changes directory into the worktree path | `claw cd auth-flow` |

### Publish a feature
Push your branch to remote (`origin`) with upstream tracking.

| Action | Description | Command |
|---|---|---|
| **Publish by name** | Pushes `feature/auth-flow` to remote with upstream tracking | `git claw feature publish auth-flow` |
| **Auto-infer publish** | When inside the worktree folder, branch name is inferred automatically | `git claw feature publish` |
| **Custom remote** | Push to a specific remote repository | `git claw feature publish auth-flow --remote upstream` |

### Finish up a feature
Finish the development of your feature.

| Action | Description | Command |
|---|---|---|
| **Finish feature** | Runs `pre_finish` hooks (tests), merges into `main`, deletes local & remote branch, frees Slot | `git claw finish auth-flow` |
| **Keep remote branch** | Merges and cleans up locally, but preserves the branch on remote | `git claw finish auth-flow --keep-remote` |
| **Force finish** | Forces merge and cleanup even if `pre_finish` hook fails | `git claw finish auth-flow --force` |

---

## Bugfixes & Hotfixes

### Bugfixes
Standard fixes branched off `main` for regular bug resolutions.

| Action | Description | Command |
|---|---|---|
| **Start bugfix** | Creates worktree for `bugfix/login-error` | `git claw bugfix start login-error` |
| **Publish bugfix** | Pushes `bugfix/login-error` to remote | `git claw bugfix publish login-error` |
| **Finish bugfix** | Merges into `main` and cleans up workspace | `git claw finish login-error` |

### Hotfixes
Urgent fixes for critical production issues.

| Action | Description | Command |
|---|---|---|
| **Start hotfix** | Creates worktree for `hotfix/security-patch` | `git claw hotfix start security-patch` |
| **Publish hotfix** | Pushes `hotfix/security-patch` to remote | `git claw hotfix publish security-patch` |
| **Finish hotfix** | Merges into `main`, deletes branch, cleans up | `git claw finish security-patch` |

---

## Spikes (Disposable POCs)

Spikes are disposable experiments and proof-of-concepts intended to be **discarded without merging into `main`**.

| Action | Description | Command |
|---|---|---|
| **Start a spike** | Creates worktree for `spike/vector-db` on a disposable branch | `git claw spike start vector-db` |
| **Drop a spike** | Safely deletes the worktree folder, branch, and releases the Slot ID | `git claw spike drop vector-db` |

---

## Workspace & Inspection

### List active workspaces
List all running worktrees, assigned slots, computed ports, and HEAD commit:

```bash
git claw list
```

```text
+------+-----------------------------+-----------------------------------+--------------------+---------+
| Slot | Branch                      | Path                              | Ports              | Commit  |
+------+-----------------------------+-----------------------------------+--------------------+---------+
| 1    | feature/auth-flow           | /home/dev/worktrees/auth-flow     | WEB:8001, DB:5433  | a1b2c3d |
| 2    | bugfix/memory-leak          | /home/dev/worktrees/memory-leak   | WEB:8002, DB:5434  | e5f6a7b |
+------+-----------------------------+-----------------------------------+--------------------+---------+
```

> **Auto-Healing Garbage Collection**: If you manually deleted a worktree folder (`rm -rf`), `git claw list` automatically detects it and releases its Slot ID.

---

## Releases (SemVer & Changelog)

`git-claw` includes an automated release engine inspired by Commitizen.

### Automatic Conventional Commit Mode
Run `git claw tag` on a clean `main` branch without arguments:

```bash
git claw tag
```

What happens automatically:
1. Analyzes commit history since the previous release tag.
2. Calculates the SemVer bump from Conventional Commits:
   - `feat!:` / `BREAKING CHANGE:` -> **MAJOR** (`0.3.0` -> `1.0.0`)
   - `feat:` -> **MINOR** (`0.3.0` -> `0.4.0`)
   - `fix:`, `perf:`, `chore:` -> **PATCH** (`0.3.0` -> `0.3.1`)
3. Updates `CHANGELOG.md` with categorized release notes.
4. Synchronizes project manifests:
   - **Rust**: `Cargo.toml` (and `Cargo.lock` via `cargo check`)
   - **Python**: `pyproject.toml` / `project.toml` (and `uv.lock` via `uv lock`)
5. Commits `chore: release <version>` and creates an annotated Git tag on `main`.

### Explicit Version Mode
To specify an exact version override:

```bash
git claw tag v1.0.0
```

---

## Commands Quick Reference

| Command | Action |
|---|---|
| `git claw init [-y]` | Initialize or update repository `.git-claw.toml` |
| `git claw list` | Display active worktrees, assigned slots, and ports |
| `git claw feature start <name>` | Create and checkout `feature/<name>` in an isolated worktree |
| `git claw bugfix start <name>` | Create and checkout `bugfix/<name>` in an isolated worktree |
| `git claw hotfix start <name>` | Create and checkout `hotfix/<name>` in an isolated worktree |
| `git claw spike start <name>` | Create a disposable experiment worktree |
| `git claw spike drop <name>` | Delete a spike worktree and branch without merging |
| `git claw publish [<name>]` | Push worktree branch to remote with upstream tracking |
| `git claw finish <name>` | Verify tests, merge into `main`, delete branch, and cleanup |
| `git claw open <name>` | Open worktree directory in `$EDITOR` or IDE |
| `git claw run <name> <cmd...>` | Execute a command inside worktree with `.env.worktree` |
| `git claw cd <name>` | Output physical directory path for shell navigation |
| `git claw tag` | Automatic SemVer bump, changelog update, and release tag |
| `git claw tag <version>` | Create an explicit release tag and synchronize manifests |
| `git claw completion <shell>` | Output shell completion script (`fish`, `bash`, `zsh`, etc.) |
