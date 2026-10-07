# Sprint Change Proposal: Runtime Real-World DX & Docker Interoperability

**Date:** 2026-10-07  
**Status:** Approved  
**Author:** John (Product Manager)  
**Trigger:** Real-world testing against existing containerized stack (`tune`) revealed critical gaps in worktree location, untracked `.env` provisioning, port variable naming, Docker name/network collisions, and shell navigation.

---

## 1. Issue Summary

During field testing on a real-world Django/Docker stack (`tune`), `git-claw` encountered multiple friction points:
1. **Worktree Directory Default:** Worktrees defaulted to sibling directory `../<repo>-worktrees/<name>`, whereas user expects centralized storage in `~/.git-claw/worktrees/<repo>/<name>`.
2. **Untracked Environment Files:** Critical configuration files (`./src/tune/settings/.env`) are gitignored and not carried over by `git worktree add`, causing startup scripts (`bin/up`) to fail immediately.
3. **Port Variable Nomenclature Mismatch:** Base ports in `.git-claw.toml` were mapped to generic keys (e.g. `PORT_WEB`), but container compose files read specific variables directly (e.g. `${APP_PORT:-80}`, `${FLOWER_PORT:-5555}`).
4. **Docker Compose Name & Network Conflicts:** Hardcoded `container_name: tune_postgres` and named volumes/networks clashed with running containers. Specifically, worktrees intended to reuse the running primary database tried to spawn duplicate containers.
5. **Shell Navigation:** CLI binaries cannot alter parent shell working directories directly; `git claw open` triggered terminal editor fallback (`nano`) while users required seamless directory navigation (`cd`) into the worktree upon creation.
6. **Project Initialization DX:** Lack of a guided `git claw init` (analogous to `git flow init`) forced manual configuration authoring.

---

## 2. Impact Analysis

- **PRD Impact:**
  - Amends FR-1 (default worktree path hierarchy & global `~/.git-claw/config.toml`).
  - Adds FR-10: `git claw init` command for automated and interactive configuration generation.
  - Adds FR-11: Untracked files copying and in-place variable merging (`[files] copy = [...]`).
  - Adds FR-12: Docker Compose override generation and shared service/network interop (`[docker]`).
  - Amends FR-8: Replaces raw editor launch with shell integration (`git claw shell-hook` / auto-cd function).
- **Architecture Impact:**
  - `core/config.rs`: Two-tier configuration loading (`~/.git-claw/config.toml` global fallback merged with local `.git-claw.toml`).
  - `infra/env_file.rs`: Enhanced to copy declared files and inject computed ports in-place.
  - `infra/docker.rs`: New module generating `docker-compose.claw.override.yml` for container name neutralization and external network attachment.
  - `cli/args.rs`: New `init` and `shell-hook` subcommands.
- **Backlog Impact:**
  - Addition of **Epic 4: Developer Experience, Docker Interop & Project Initialization** with 5 prioritized stories.

---

## 3. Recommended Approach

- **Direct Adjustment:** Implement Epic 4 without rolling back Epics 1–3. Existing primitives (slot allocation, advisory locks, git worktree engine) remain solid and reusable.
- **Configuration Hierarchy:** Local `.git-claw.toml` overrides user global `~/.git-claw/config.toml`, which overrides hardcoded defaults.
- **Docker Compose Override Strategy:**
  - When `[docker] shared_services = ["postgres"]` is specified, generate an override file setting `external: true` for the shared network and neutralizing duplicate services.

---

## 4. Implementation Handoff

- **Scope:** Moderate (Architectural additions & new feature stories).
- **Recipient:** Developer Agent (Amelia / `bmad-build-auto`).
- **Target Deliverable:** Epic 4 implementation validated against unit, integration, and E2E tests, followed by verification on the `tune` repository.
