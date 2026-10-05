# git-claw 🦀

> **Generic Git Worktrees Orchestrator for concurrent human & AI development.**

[![Vibe-Coded with Antigravity](https://img.shields.io/badge/Vibe--Coded%20in-Antigravity%20(AGY)-blueviolet?style=for-the-badge)](https://github.com/google/antigravity)
[![Powered by Gemini](https://img.shields.io/badge/Model-Google%20Gemini-4285F4?style=for-the-badge&logo=google)](https://deepmind.google/technologies/gemini/)
[![Methodology BMad](https://img.shields.io/badge/Methodology-BMad%20Method-orange?style=for-the-badge)](https://github.com/bmad-code-org/BMAD-METHOD)
[![License: GPL-2.0](https://img.shields.io/badge/License-GPL--2.0-blue?style=for-the-badge)](LICENSE)

---

## 💡 About git-claw

`git-claw` is an autonomous, lightweight CLI tool written in Rust that eliminates workspace collisions during concurrent development. When human developers or autonomous AI coding agents work across multiple Git branches in parallel, they routinely hit frustrating friction: local port conflicts (`Error: Address already in use: 8000`), conflicting container runtime names, and repeated dependency installations.

`git-claw` solves this with zero friction:
- **Deterministic Slot IDs**: Automatically allocates a unique numeric slot (`1, 2, 3...`) per active worktree.
- **Collision-Free Port Offsets**: Computes ports deterministically (`Effective Port = Base Port + Slot ID`).
- **Environment Injection**: Generates an isolated `.env.worktree` in every worktree containing slot and port definitions.
- **Shared Build Caches**: Reuses heavy directories (`node_modules`, `target`, `.venv`) via transparent symlinks.
- **Self-Healing Garbage Collection**: Detects manually deleted worktree directories and frees their slots automatically.
- **100% Technology Agnostic**: Zero mandatory directory layouts or template scripts imposed.

---

## 📦 Installation

### One-Line Install with Cargo ⚡
Install `git-claw` directly in a single command:

```bash
# Directly from the Git repository (one-liner):
cargo install --git https://github.com/jeremylanes/git-claw.git

# Or from crates.io (once published):
cargo install git-claw

# Or from the local source directory:
cargo install --path .
```

Verify the installation:
```bash
git-claw --version
```

### Native Git Subcommand Integration
Because the binary is named `git-claw`, Git automatically registers it as a first-class subcommand. As long as `~/.cargo/bin` is in your `$PATH`, both of these commands work identically:
```bash
git-claw --help
git claw --help
```

### Optional Shell Helper for Directory Navigation
To navigate directly into a worktree directory using `git claw cd`, add this function to your `~/.bashrc` or `~/.zshrc`:
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

## 🚀 Getting Started & Initialization

### Option A: In an Existing Git Repository (Zero-Config)
`git-claw` requires **zero configuration files** to work out of the box!  
Simply navigate to any existing Git repository and start a worktree right away:

```bash
cd ~/projects/my-existing-app

# Instantly create an isolated worktree for a new feature:
git claw feature start user-authentication
```

What happens automatically:
1. `git-claw` assigns **Slot ID 1**.
2. Creates the worktree at `../my-existing-app-worktrees/user-authentication`.
3. Creates and checks out branch `feature/user-authentication` based on `main`.
4. Writes `.env.worktree` with `CLAW_SLOT_ID=1`.

---

### Option B: In a Brand New Git Project
If starting a new project from scratch:

```bash
mkdir my-new-project && cd my-new-project
git init -b main
touch README.md
git add README.md
git commit -m "Initial commit"

# You are immediately ready to use git-claw!
git claw feature start initial-scaffold
```

---

### Option C: Customizing with `.git-claw.toml` (Recommended for Complex Stacks)
To define base ports to offset, lifecycle hooks, or shared caches, place a `.git-claw.toml` file at the root of your primary repository:

```toml
[project]
name = "my-app"
main_branch = "main"                     # Defaults to "main"
worktree_root = "../my-app-worktrees"    # Defaults to "../<repo>-worktrees"

[ports]
# Ports automatically shifted by Slot ID (e.g. Slot 1 -> 8001, Slot 2 -> 8002)
web = 8000
database = 5432
redis = 6379

[hooks]
# Optional shell commands executed at lifecycle events
post_start = "bin/up"        # Run right after worktree creation
pre_finish = "bin/test"      # Run before merging into main (aborts merge if tests fail)
post_finish = "bin/down"     # Run after merge and worktree cleanup

[cache]
strategy = "shared"          # "shared" (symlinks) or "isolated"
shared_paths = [".venv", "node_modules", "target"]
```

---

## 📖 Complete User Manual (Day-to-Day Workflow)

### 1. Starting a Worktree
`git-claw` supports 4 semantic branch prefixes: `feature`, `bugfix`, `hotfix`, and `spike`.

```bash
# Create a standard feature worktree
git claw feature start auth-flow

# Create a hotfix worktree
git claw hotfix start memory-leak

# Create an isolated worktree without sharing cache symlinks
git claw feature start complex-refactor --isolated

# Create a throwaway spike/POC
git claw spike start test-vector-db
```

Each worktree receives a dedicated `.env.worktree` inside its root:
```bash
cat ../my-app-worktrees/auth-flow/.env.worktree
# Outputs:
# CLAW_SLOT_ID=1
# CLAW_WORKTREE_NAME="auth-flow"
# CLAW_WORKTREE_PATH="/path/to/my-app-worktrees/auth-flow"
# COMPOSE_PROJECT_NAME="my_app_wt1"
# PORT_WEB=8001
# PORT_DATABASE=5433
# PORT_REDIS=6380
```

---

### 2. Inspecting Active Workspaces (`git claw list`)
Check all running worktrees, their assigned slots, calculated ports, and latest commit:

```bash
git claw list
```

Example Output:
```text
+------+-----------------------------+-----------------------------------+--------------------+---------+
| Slot | Branch                      | Path                              | Ports              | Commit  |
+------+-----------------------------+-----------------------------------+--------------------+---------+
| 1    | feature/auth-flow           | /home/user/worktrees/auth-flow    | WEB:8001, DB:5433  | a1b2c3d |
| 2    | bugfix/memory-leak          | /home/user/worktrees/memory-leak  | WEB:8002, DB:5434  | e5f6a7b |
+------+-----------------------------+-----------------------------------+--------------------+---------+
```

> **Auto-Healing GC**: If you ever manually deleted a worktree folder (`rm -rf`), `git claw list` detects the missing directory and automatically frees its Slot ID for future branches.

---

### 3. Working inside a Worktree

#### Open in your preferred editor:
```bash
# Opens the folder with $EDITOR, VS Code, Cursor, etc.
git claw open auth-flow
```

#### Run commands with worktree environment variables injected:
```bash
# Runs the command directly inside the worktree directory with .env.worktree loaded:
git claw run auth-flow npm run dev
git claw run auth-flow cargo test
git claw run auth-flow docker compose up -d
```

#### Navigate to the worktree folder:
```bash
cd $(git claw cd auth-flow)
# Or with the shell helper:
claw cd auth-flow
```

---

### 4. Completing and Merging a Worktree (`git claw finish`)
When your feature or bugfix is complete, finish it:

```bash
git claw finish auth-flow
```

What `git claw finish` does:
1. Runs the `hooks.pre_finish` command (e.g. `bin/test` or `cargo test`). If tests fail, **merge is blocked** to protect `main`!
2. Merges the branch into `main`.
3. Runs the `hooks.post_finish` command (e.g. `bin/down`).
4. Removes the worktree directory and deletes the branch.
5. Releases the Slot ID in `.git/claw/slots.json`.

> **Bypassing test failures**: If you need to force merge despite a failed hook, use:  
> `git claw finish auth-flow --force`

---

### 5. Discarding a Spike / POC (`git claw spike drop`)
Spikes are disposable experiments that should never be merged into `main`:

```bash
git claw spike drop test-vector-db
```
This safely deletes the worktree folder, purges the `spike/test-vector-db` branch, and releases the Slot ID immediately.

---

### 6. Production SemVer Release (`git claw tag`)
When you are on a clean `main` branch and ready to cut a production release:

```bash
git claw tag v1.2.0
```
`git-claw` validates the SemVer format, ensures your working copy is clean, and creates an annotated Git tag on `main`.

---

## 🛠️ Stack Configuration Recipes

### 🐍 Python / Django / FastAPI Stack
```toml
[ports]
web = 8000
database = 5432
redis = 6379

[hooks]
post_start = "docker compose up -d && python manage.py migrate"
pre_finish = "pytest"
post_finish = "docker compose down"

[cache]
strategy = "shared"
shared_paths = [".venv"]
```

### ☕ Node.js / TypeScript Stack
```toml
[ports]
web = 3000
api = 4000

[hooks]
pre_finish = "npm run test"

[cache]
strategy = "shared"
shared_paths = ["node_modules", ".next"]
```

### 🦀 Rust Stack
```toml
[hooks]
pre_finish = "cargo test"

[cache]
strategy = "shared"
shared_paths = ["target"]
```

---

## 🤖 Built with AGY, Gemini & BMad

This project was **100% vibe-coded** inside **Google Antigravity (AGY)** using **Gemini**, meticulously planned and governed through the **BMad Method**:

- **Product Brief**: Validated product scope and modular philosophy.
- **Product Requirements Document (PRD)**: Functional criteria (FR-1 to FR-9) and user journeys ([PRD document](_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md)).
- **Architecture Spine**: Layered command-driven design, boundary invariants, and architectural decisions AD-1 to AD-10 ([Architecture document](_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md)).
- **Ticket Slicing & Epics**: 3 sequential epics broken down into 18 test-driven stories managed by `bmad-ticket`.

---

## 📄 License
 
GPL-2.0-only (GNU General Public License v2.0) — exactly the same license as the upstream [Git](https://github.com/git/git/blob/master/COPYING) project. See the full text in [LICENSE](LICENSE).
