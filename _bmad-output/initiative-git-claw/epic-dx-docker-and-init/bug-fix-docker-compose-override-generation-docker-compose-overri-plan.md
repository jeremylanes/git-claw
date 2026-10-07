---
title: 'Fix Docker Compose override generation: docker-compose.override.yml and !reset neutralization'
type: 'fix'
ticket: 7
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
context: ['_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md', '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md', '_bmad-output/initiative-git-claw/change-runtime-fixes-and-init/change-runtime-fixes-and-init.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:**
1. The Docker Compose override was generated as `docker-compose.claw.override.yml`, requiring explicit `-f` flags instead of Docker Compose's standard auto-loaded `docker-compose.override.yml`.
2. Setting `container_name: null` causes schema validation warnings or failures in strict compose validators, whereas Docker Compose v2 specifies `container_name: !reset` to unset inherited scalar values.
3. Using `profiles: ["claw-disabled"]` broke `depends_on` dependencies for services (like `web` depending on `postgres`).
4. `workflow/init.rs` did not scan network names under `networks:` in `docker-compose.yml`.

**Approach:**
1. In `src/infra/docker.rs`, change generated filename to `docker-compose.override.yml`.
2. In `generate_override_content`, replace `container_name: null` with `container_name: !reset`.
3. In `generate_override_content`, configure shared services with `entrypoint: ["true"]`, `restart: "no"`, and `scale: 0` without disabling profiles to prevent breaking `depends_on`.
4. In `src/workflow/init.rs`, scan `networks:` in Compose files to automatically populate `[docker] network`.

## Boundaries & Constraints

**Always:**
- Generate standard `docker-compose.override.yml`.
- Use `container_name: !reset` for container name neutralization.
- Ensure the generated override file validates against `docker compose config`.
- Connect worktree services to the external network.
- Scan network names in `git claw init`.

**Never:**
- Never break `depends_on` relationships between application services and shared services.

## I/O & Edge-Case Matrix

| Scenario | Input | Expected Output | Error Handling |
|----------|-------|-----------------|----------------|
| Base compose with `container_name` | `container_name: tune_web` | `container_name: !reset` in override | Ok |
| Base compose with `depends_on: [postgres]` | Shared service `postgres` | `postgres` neutralized without profiles, `depends_on` intact | Ok |
| Auto-loaded file name | Worktree start | `docker-compose.override.yml` created | Ok |
| `git claw init` network scan | `networks: { default: { name: tune_net } }` | `docker.network = Some("tune_net")` | Ok |

</frozen-after-approval>

## Code Map

- `src/infra/docker.rs` -- Update filename to `docker-compose.override.yml`, use `!reset`, remove profiles.
- `src/workflow/init.rs` -- Scan networks in `scan_compose_details`.
- `tests/test_docker_override.rs` -- Update test assertions and validate with `docker compose config`.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/docker.rs` -- Generate `docker-compose.override.yml` with `!reset` and without profiles.
- [x] `src/workflow/init.rs` -- Scan networks from compose files.
- [x] `tests/test_docker_override.rs` -- Update tests and add `docker compose config` validation test.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- `docker-compose.override.yml` is generated.
- `container_name: !reset` is used.
- Validates with `docker compose config`.

## Implementation Notes
- Changed override filename from `docker-compose.claw.override.yml` to standard auto-loaded `docker-compose.override.yml`.
- Replaced `container_name: null` with `container_name: !reset` for spec-compliant Docker Compose unsetting.
- Neutralized shared services with `entrypoint: ["true"]`, `restart: "no"`, and `scale: 0` without adding disabling profiles, preventing breakage of services depending on shared services.
- Added `scan_compose_network` in `src/workflow/init.rs` to automatically detect defined networks.
- Validated with `docker compose config` in integration tests.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test --test test_docker_override` -- expected: all tests pass

## Auto Run Result
- `cargo test --test test_docker_override`: 4/4 passed (including `docker compose config` validation)
- `cargo test --test test_init_workflow`: 3/3 passed
- `cargo clippy`: 0 errors, 0 warnings
- `cargo fmt -- --check`: clean

