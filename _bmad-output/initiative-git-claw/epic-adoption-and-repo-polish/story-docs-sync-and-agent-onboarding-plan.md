---
title: 'Documentation sync, emoji removal, and AI agent SKILL.md'
type: 'chore'
ticket: 4
created: '2026-11-13'
status: 'done'
route: 'full'
risk: 'low'
review: 'quick'
---

## Intent

**Problem:**
1. `README.md`, `CHEATSHEET.md`, and `docs/index.html` violate Rule 35 by containing crab emojis (U+1F980).
2. Documentation is out of sync with implemented functionality:
   - `README.md` shows non-existent `[project] name = "my-service"` and stale `worktree_root = "../my-service-worktrees"`, while missing `[files] copy`, `[docker]` (`compose_file`, `network`, `shared_services`, `!reset`, `TEST_DATABASE_NAME` per-slot test DB isolation), `[environment]` (`pyenv`/`uv`/`venv`/`poetry`/`nvmrc`), `git claw shell-hook` (bash/zsh/fish), `--cd` on `start`, `publish`, `completion`, `--json` agent output, Herdr multiplexer integration, and prebuilt binary installation instructions.
   - `CHEATSHEET.md` still shows a manual legacy `claw()` bash wrapper instead of `git-claw shell-hook` (bash/zsh/fish), misses `--cd`, `--json`, `[docker]`, test DB isolation, `pyenv`/`uv`, and Herdr.
   - `docs/index.html` lacks `--cd`, `--json`, `shell-hook`, `[docker]`, test DB isolation, `pyenv`/`uv`, `init --yes`, and Herdr.
   - No ready-to-use AI Agent integration snippet (`AGENTS.md` / `CLAUDE.md`) or `skills/git-claw/SKILL.md` exists in the repository.

**Approach:**
1. Audit every command/flag against `src/cli/args.rs` and every config field against `src/core/config.rs` (exact key names only, never inventing flags or TOML fields).
2. Remove ALL emojis across `README.md`, `CHEATSHEET.md`, `docs/index.html`, and any new doc files (grep verification afterwards).
3. Update `README.md`:
   - Clear value proposition (problem -> solution) focused on parallel human + AI agent worktrees with zero port/Docker/test-DB collisions.
   - Installation options: Prebuilt binaries (GitHub Releases + `cargo-binstall` / `cargo install git-claw` / source build).
   - Quick Start (1-minute flow: `git claw init`, `git claw shell-hook`, `git claw feature start`, `git claw list --json`, `git claw finish`).
   - Accurate `.git-claw.toml` reference matching `Config` (`[project]` with `main_branch`, `worktree_root`, `remote`; `[ports]`; `[files]`; `[docker]`; `[environment]`; `[cache]`; `[hooks]`).
   - Dedicated sections for Test DB Isolation (`TEST_DATABASE_NAME` / `CLAW_TEST_DB`), Environment Auto-Propagation (`pyenv`, `uv`, `venv`, `nvmrc`), Herdr integration, and AI Coding Agents (`--json` + `skills/git-claw/SKILL.md` + `AGENTS.md`/`CLAUDE.md` snippet).
   - Complete CLI Reference & License (`GPL-2.0-only`).
4. Update `CHEATSHEET.md` and `docs/index.html` to stay 100% consistent with `README.md` and current CLI.
5. Create `skills/git-claw/SKILL.md` (no emojis, valid YAML frontmatter `name: git-claw` + `description`, clear workflow rules for AI agents using `git claw ... --json`).

## Acceptance Criteria
1. Zero emojis present in `README.md`, `CHEATSHEET.md`, `docs/index.html`, `skills/git-claw/SKILL.md`, and `CHANGELOG.md`.
2. Every TOML section/field and CLI flag shown in docs exists verbatim in `src/core/config.rs` and `src/cli/args.rs`.
3. `skills/git-claw/SKILL.md` exists with valid frontmatter and clear agent instructions.
4. `docs/index.html` has valid HTML structure, balanced tags, and no broken internal links.
5. Doc-tests (`cargo test`) still pass completely.
