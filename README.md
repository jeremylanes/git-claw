# git-claw 🦀

> **Generic Git Worktrees Orchestrator for concurrent human & AI development.**

[![Vibe-Coded with Antigravity](https://img.shields.io/badge/Vibe--Coded%20in-Antigravity%20(AGY)-blueviolet?style=for-the-badge)](https://github.com/google/antigravity)
[![Powered by Gemini](https://img.shields.io/badge/Model-Google%20Gemini-4285F4?style=for-the-badge&logo=google)](https://deepmind.google/technologies/gemini/)
[![Methodology BMad](https://img.shields.io/badge/Methodology-BMad%20Method-orange?style=for-the-badge)](https://github.com/bmad-code-org/BMAD-METHOD)
[![License: GPL-2.0](https://img.shields.io/badge/License-GPL--2.0-blue?style=for-the-badge)](LICENSE)

---

## 💡 About git-claw

`git-claw` is an autonomous, lightweight CLI tool written in Rust that eliminates workspace friction during concurrent development. When developers or autonomous AI agents work across multiple Git branches simultaneously, they routinely suffer from local port collisions, conflicting container runtime names, and redundant dependency downloads.

`git-claw` solves this by:
- Allocating unique, deterministic **Slot IDs** (1, 2, 3...) per active worktree.
- Computing collision-free **Port Offsets** (`Effective Port = Base Port + Slot ID`).
- Generating an isolated `.env.worktree` inside each worktree.
- Managing shared build caches (`node_modules`, `target`, `.venv`) via transparent symlinks.
- Providing self-healing garbage collection of orphaned slots.
- Staying completely technology-agnostic: zero mandatory directory conventions or scripts required.

---

## 🤖 Built with AGY, Gemini & BMad

This project is **100% vibe-coded** inside **Google Antigravity (AGY)** using **Gemini**, meticulously structured and governed through the **BMad Method**:

- **Product Brief**: Validated product scope and modular philosophy.
- **Product Requirements Document (PRD)**: Comprehensive requirements, testable functional criteria (FR-1 through FR-9), and user journeys ([PRD document](_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md)).
- **Architecture Spine**: Layered command-driven design, boundary invariants, and architectural decisions AD-1 through AD-10 ([Architecture document](_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md)).
- **Ticket Slicing & Epics**: 3 sequential epics broken down into 18 test-driven stories managed by `bmad-ticket`.

---

## ⚡ CLI Usage Overview

```bash
# === 1. Create an isolated worktree ===
git claw feature start <name> [--isolated]   # Allocates slot, creates worktree, writes .env.worktree, runs post_start
git claw bugfix start <name>
git claw hotfix start <name>
git claw spike start <name>                 # Disposable experimental worktree

# === 2. Inspect active workspaces ===
git claw list                               # Dashboard: active worktrees, assigned slots, ports & Git status

# === 3. Execute & Navigate ===
git claw run [name] <command...>            # Run commands inside worktree context with .env.worktree injected
git claw open [name]                        # Open worktree folder in configured editor ($EDITOR / code / cursor)
git claw cd [name]                          # Print worktree path for shell cd navigation

# === 4. Validate & Finish ===
git claw finish [name] [--force]            # Runs pre_finish (tests), merges to main, cleans worktree, frees slot
git claw spike drop <name>                  # Discards experimental spike without merging

# === 5. Production Release ===
git claw tag <semver-version>               # Verifies clean main branch and cuts an annotated SemVer tag
```

---

## ⚙️ Configuration (`.git-claw.toml`)

Zero configuration is required by default. To customize port offsets, hooks, or cache sharing, drop a `.git-claw.toml` in your repository root:

```toml
[project]
name = "my-app"
main_branch = "main"
worktree_root = "../my-app-worktrees"

[ports]
web = 8000
database = 5432
redis = 6379

[hooks]
post_start = "bin/up"        # or "docker compose up -d" or omitted
pre_finish = "bin/test"      # or "cargo test" or "pytest" or omitted
post_finish = "bin/down"     # or "docker compose down" or omitted

[cache]
strategy = "shared"          # "shared" (symlinks) or "isolated"
shared_paths = [".venv", "node_modules", "target"]
```

---

## 📄 License
 
GPL-2.0-only (GNU General Public License v2.0) — exactly the same license as the upstream [Git](https://github.com/git/git/blob/master/COPYING) project. See the full text in [LICENSE](LICENSE).
