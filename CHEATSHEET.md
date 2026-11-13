# git-claw Cheatsheet

> Practical guide to orchestrating isolated Git Worktrees for concurrent human and AI agent workflows.

---

## Index

- [About & Core Concepts](#about--core-concepts)
- [Setup & Installation](#setup--installation)
- [Getting Started](#getting-started)
- [Features](#features)
- [Bugfixes & Hotfixes](#bugfixes--hotfixes)
- [Spikes (Disposable POCs)](#spikes-disposable-pocs)
- [Docker, Test DBs & Environments](#docker-test-dbs--environments)
- [Workspace, Inspection & AI Agents](#workspace-inspection--ai-agents)
- [Releases (SemVer & Changelog)](#releases-semver--changelog)
- [Commands Quick Reference](#commands-quick-reference)

---

## About & Core Concepts

Unlike traditional Git workflows that swap branches in-place inside one directory (`git checkout`), `git-claw` allocates an isolated physical workspace (worktree) per branch:

```text
[Main Repository] (main)
   |-- ~/.git-claw/worktrees/<repo>/auth-jwt     (feature/auth-jwt)     -> Slot 1 (PORT_WEB: 8001)
   |-- ~/.git-claw/worktrees/<repo>/fix-session  (bugfix/fix-session)   -> Slot 2 (PORT_WEB: 8002)
   +-- ~/.git-claw/worktrees/<repo>/poc-redis    (spike/poc-redis)      -> Slot 3 (PORT_WEB: 8003)
```

- **Deterministic Slot IDs**: Sequential numeric slots (`1, 2, 3...`) allocated under advisory locks.
- **Collision-Free Port Offsets**: `Effective Port = Base Port + Slot ID` (shared client service ports preserved).
- **Per-Slot Test DB Isolation**: `TEST_DATABASE_NAME` and `CLAW_TEST_DB` (`test_<db>_slot_<id>`) injected automatically.
- **Environment Injection**: Generates `.env.worktree` and propagates `pyenv`, `uv`/`.venv`, `.nvmrc`, `.node-version`, `.envrc`.
- **Docker Compose Interop**: Generates `docker-compose.override.yml` with `container_name: !reset` and `shared_services` scaling.
- **Shared Build Caches**: Symlinks `node_modules`, `target`, `.venv` (opt out with `--isolated`).
- **Auto-Healing GC**: Reclaims slots when worktree directories are deleted manually.

---

## Setup & Installation

### Prebuilt Binaries & Cargo

| Method | Command / Source |
|---|---|
| **GitHub Releases** | Download target archive + `SHA256SUMS.txt` from [Releases](https://github.com/jeremylanes/git-claw/releases) |
| **cargo-binstall** | `cargo binstall git-claw` |
| **Crates.io** | `cargo install git-claw` |
| **From Source** | `cargo install --path .` |

### Shell Integration (`shell-hook`)

Enable auto-cd navigation (`--cd` on `start`, automatic return to `main` on `finish` and `spike drop`, plus `claw cd`):

| Shell | Setup Command |
|---|---|
| **Bash** | `eval "$(git claw shell-hook bash)"` |
| **Zsh** | `eval "$(git claw shell-hook zsh)"` |
| **Fish** | `git claw shell-hook fish \| source` |

### Shell Auto-Completion

| Shell | Setup Command |
|---|---|
| **Fish** | `mkdir -p ~/.config/fish/completions && git claw completion fish > ~/.config/fish/completions/git-claw.fish` |
| **Bash** | `mkdir -p ~/.local/share/bash-completion/completions && git claw completion bash > ~/.local/share/bash-completion/completions/git-claw` |
| **Zsh** | `mkdir -p ~/.zfunc && git claw completion zsh > ~/.zfunc/_git-claw` |

---

## Getting Started

### Zero-Config Start

```bash
git claw feature start my-first-feature
```

### Guided Configuration (`git claw init`)

`git claw init` scans `.env` files, `docker-compose.yml`, `.python-version`, `uv.lock`, `.venv`, `.nvmrc`, and `.node-version`:

| Action | Details | Command |
|---|---|---|
| **Interactive Init** | Guards existing `.git-claw.toml`, prompts `main_branch`, confirms before writing | `git claw init` |
| **Non-Interactive** | Overwrites/writes `.git-claw.toml` with detected defaults without prompting | `git claw init --yes` |
| **Agent JSON Init** | Emits JSON config summary (requires `--yes`) | `git claw init --yes --json` |

---

## Features

```text
(main) ---> git claw feature start oauth ---> [Worktree feature/oauth]
                                                       |
                                             (Develop & Test in isolation)
                                                       |
(main) <--- git claw finish oauth <--------------------+
```

### Start a new feature

| Action | Description | Command |
|---|---|---|
| **Start feature** | Creates worktree in `worktree_root`, allocates Slot ID, writes `.env.worktree` | `git claw feature start auth-flow` |
| **Start & enter** | Creates worktree and navigates shell into it (`shell-hook` enabled) | `git claw feature start auth-flow --cd` |
| **Start isolated** | Skips shared cache directory symlinking | `git claw feature start auth-flow --isolated` |
| **Start (JSON)** | Returns `{ slot_id, name, branch, path, ports, test_database }` | `git claw feature start auth-flow --json` |

### Work inside a feature

| Action | Description | Command |
|---|---|---|
| **Open in editor** | Opens worktree in `$EDITOR`, `$VISUAL`, or `code` | `git claw open auth-flow` |
| **Run command** | Runs command inside worktree with `.env.worktree` injected | `git claw run auth-flow npm test` |
| **Navigate directory** | Changes shell directory into the worktree (or `main`) | `git claw cd auth-flow` |

### Publish a feature

| Action | Description | Command |
|---|---|---|
| **Publish by name** | Pushes `feature/auth-flow` with upstream tracking | `git claw feature publish auth-flow` |
| **Auto-infer publish** | Infers worktree when run inside the worktree directory | `git claw feature publish` |
| **Top-level shorthand** | Publishes any worktree type without repeating type | `git claw publish auth-flow` |
| **Custom remote / JSON** | Push to specific remote and emit `{ name, branch, remote }` | `git claw feature publish auth-flow --remote upstream --json` |

### Finish up a feature

| Action | Description | Command |
|---|---|---|
| **Finish feature** | Runs `pre_finish` hook, merges into `main_branch`, deletes local & remote branch, frees Slot ID | `git claw finish auth-flow` |
| **Keep remote branch** | Preserves remote branch while cleaning up locally | `git claw finish auth-flow --keep-remote` |
| **Force finish** | Continues merge even if `pre_finish` hook exits non-zero | `git claw finish auth-flow --force` |
| **Finish (JSON)** | Emits `{ name, branch, target_branch, slot_id, primary_root, remote_deleted }` | `git claw finish auth-flow --json` |

---

## Bugfixes & Hotfixes

| Action | Description | Command |
|---|---|---|
| **Start bugfix** | Creates worktree for `bugfix/login-error` | `git claw bugfix start login-error` |
| **Publish bugfix** | Pushes `bugfix/login-error` to remote | `git claw bugfix publish login-error` |
| **Start hotfix** | Creates worktree for `hotfix/security-patch` | `git claw hotfix start security-patch` |
| **Publish hotfix** | Pushes `hotfix/security-patch` to remote | `git claw hotfix publish security-patch` |
| **Finish fix** | Runs hooks, merges into `main_branch`, cleans up | `git claw finish security-patch` |

---

## Spikes (Disposable POCs)

Spikes are throwaway branches (`spike/<name>`) meant to be discarded without merging.

| Action | Description | Command |
|---|---|---|
| **Start a spike** | Creates isolated worktree for `spike/vector-db` | `git claw spike start vector-db` |
| **Drop a spike** | Force-removes worktree and branch, frees Slot ID, returns shell to `main` | `git claw spike drop vector-db` |
| **Drop spike (JSON)** | Emits `{ name, branch, slot_id, primary_root }` | `git claw spike drop vector-db --json` |

---

## Docker, Test DBs & Environments

- **Untracked Files (`[files].copy`)**: Copied into every worktree; `.env` files have ports, `COMPOSE_PROJECT_NAME`, `TEST_DATABASE_NAME`, and `CLAW_TEST_DB` updated in-place (`test_<db>_slot_<id>`).
- **Docker Compose (`[docker]`)**: Generates `docker-compose.override.yml` with `container_name: !reset`, shared services disabled (`scale: 0`), external network/volume wiring, and per-slot `TEST_DATABASE_NAME` / `CLAW_TEST_DB`.
- **Environments (`[environment]`)**: Propagates `.python-version` (`PYENV_VERSION`), symlinks `.venv` (`VIRTUAL_ENV` + `PATH` for `uv`/`venv`), and copies `.nvmrc`, `.node-version`, `.envrc`.
- **Herdr Integration**: Auto-opens new worktree branches in Herdr when `HERDR_ENV=1` (disable with `CLAW_DISABLE_HERDR=1`).

---

## Workspace, Inspection & AI Agents

### List active workspaces

```bash
git claw list
git claw list --json
```

`git claw list --json` outputs:

```json
{
  "worktrees": [
    {
      "slot_id": 1,
      "name": "auth-flow",
      "branch": "feature/auth-flow",
      "path": "/home/dev/.git-claw/worktrees/my-app/auth-flow",
      "ports": { "PORT_WEB": 8001 },
      "status": "active"
    }
  ]
}
```

> **Auto-Healing Garbage Collection**: Manually deleted folders are detected and reclaimed automatically on `list` and `start`.

---

## Releases (SemVer & Changelog)

```bash
# Automatic mode (Conventional Commits -> CHANGELOG.md -> manifest sync -> annotated tag):
git claw tag

# Explicit SemVer override:
git claw tag v1.2.0
```

Synchronizes `Cargo.toml` (`Cargo.lock` via `cargo check`) and `pyproject.toml` / `project.toml` (`uv.lock` via `uv lock`).

---

## Commands Quick Reference

| Command | Action |
|---|---|
| `git claw init [-y] [--json]` | Initialize or update `.git-claw.toml` |
| `git claw list [--json]` | Display active worktrees, slots, and ports |
| `git claw feature\|bugfix\|hotfix start <name> [--isolated] [--cd] [--json]` | Create typed isolated worktree |
| `git claw spike start <name> [--isolated] [--cd] [--json]` | Create disposable experiment worktree |
| `git claw spike drop <name> [--json]` | Discard spike worktree and branch without merging |
| `git claw publish [<name>] [--remote <r>] [--json]` | Push worktree branch to remote with upstream tracking |
| `git claw finish [<name>] [--force] [-k\|--keep-remote] [--json]` | Run hooks, merge into `main`, delete branch, and cleanup |
| `git claw open [<name>]` | Open worktree directory in configured editor |
| `git claw run [<name>] <cmd...>` | Execute command inside worktree with `.env.worktree` |
| `git claw cd [<name>]` | Output worktree path for shell navigation |
| `git claw tag [<version>]` | Commitizen auto bump or explicit release tag |
| `git claw shell-hook [bash\|zsh\|fish]` | Emit shell auto-cd integration script |
| `git claw completion <shell>` | Output shell completion script |
