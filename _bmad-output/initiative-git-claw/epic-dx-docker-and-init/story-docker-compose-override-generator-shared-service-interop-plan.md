---
title: 'Docker Compose override generator & shared service interop'
type: 'feature'
ticket: 3
created: '2026-10-07'
status: built
baseline_revision: '68eb24820f3950ffc28102d78be771c7168787e9'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-runtime-fixes-and-init/change-runtime-fixes-and-init.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Real-world containerized stacks (e.g. `tune`) encounter container name collisions and duplicate service launches when starting multiple worktrees. Specifically, hardcoded `container_name` entries clash, and auxiliary services (like PostgreSQL) are duplicated rather than attaching to the shared database running on the host/main network.

**Approach:** Implement `[docker]` configuration table support in `src/infra/docker.rs` and wire into `src/workflow/start.rs`.
When starting a worktree:
1. Detect base compose file (`config.docker.compose_file` or fallback to `docker-compose.yml`, `docker-compose.yaml`, `compose.yml`, `compose.yaml`).
2. Generate `docker-compose.claw.override.yml` in the worktree.
3. For services declared in `shared_services`: neutralize `container_name: null`, set `profiles: ["claw-disabled"]`, `restart: "no"`, `scale: 0`, and `entrypoint: ["true"]` so duplicate containers are not spawned.
4. For other services with hardcoded `container_name`: neutralize `container_name: null` and attach to `config.docker.network` if specified.
5. In the `networks` block, set the configured shared network to `external: true`.

## Boundaries & Constraints

**Always:**
- Generate valid YAML syntax.
- Neutralize container names (`container_name: null`) to allow Docker Compose project name prefixing.
- Connect services to external network when `config.docker.network` is configured.
- Disable shared services so worktrees reuse existing shared infrastructure.
- Handle missing compose files gracefully without crashing worktree creation.

**Never:**
- Never modify the base compose file in-place.
- Never execute Docker CLI commands directly (git-claw is not a container runner).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Base compose with hardcoded container names | `web` has `container_name: tune_web` | `web.container_name: null` in override | Ok(()) |
| Declared shared service | `shared_services = ["postgres"]` | `postgres` neutralized and scaled to 0 | Ok(()) |
| Declared network | `network = "tune_network"` | `networks.tune_network.external: true` | Ok(()) |
| Docker config empty | Default `DockerConfig` | No override file generated | Ok(None) |
| Missing base compose file | Compose file does not exist | Generates override with shared services & network | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/infra/docker.rs` -- Module for parsing Compose services and generating `docker-compose.claw.override.yml`.
- `src/infra/mod.rs` -- Export docker module functions.
- `src/workflow/start.rs` -- Invoke `generate_docker_compose_override` during worktree creation.
- `tests/test_docker_override.rs` -- Integration tests verifying override generation, neutralization, network attachment, and service syntax.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/docker.rs` -- Implement `generate_docker_compose_override` and compose parser.
- [x] `src/infra/mod.rs` -- Register `pub mod docker`.
- [x] `src/workflow/start.rs` -- Call generator in start workflow.
- [x] `tests/test_docker_override.rs` -- Test compose override generator.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- Given `[docker]` in config, `docker-compose.claw.override.yml` is generated in the worktree.
- Container names are neutralized to `null`.
- Shared services have disable profiles and zero scale.
- Configured network is marked `external: true`.

## Implementation Notes
- Implemented `parse_compose_services`, `generate_override_content`, and `generate_docker_compose_override` in `src/infra/docker.rs`.
- Neutralizes `container_name: null`, attaches services to configured network, configures shared services with `profiles: [claw-disabled]` and `scale: 0`, and declares external network.
- Wired into `src/workflow/start.rs`.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test --test test_docker_override` -- expected: all tests pass

## Auto Run Result
- `cargo test --test test_docker_override`: 4/4 passed
- `cargo clippy`: 0 errors, 0 warnings
- `cargo fmt -- --check`: clean

