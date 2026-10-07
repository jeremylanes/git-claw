---
title: 'Project initialization command (git claw init)'
type: 'feature'
ticket: 4
created: '2026-10-07'
status: built
baseline_revision: '68eb24820f3950ffc28102d78be771c7168787e9'
route: 'full'
route_source: 'auto'
risk: 'low'
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

**Problem:** Setting up `.git-claw.toml` manually requires developers to inspect compose files, list gitignored `.env` paths, identify port variables, and author the TOML configuration by hand.

**Approach:** Implement `git claw init` (and non-interactive `git claw init --yes` / `-y`):
1. Inspect the repository for existing `.env` files (e.g. `.env`, `config/.env`, `settings/.env`) and add them to `[files] copy = [...]`.
2. Inspect compose files (`docker-compose.yml`, `compose.yaml`, etc.) to detect `compose_file`, shared services (e.g. `postgres`, `redis`, `db`), and network configurations.
3. Parse `.env` files and compose definitions for port variables (e.g. `APP_PORT=8000`, `PORT=3000`) and populate `[ports]`.
4. Construct and write a formatted `.git-claw.toml` to the repository root.

## Boundaries & Constraints

**Always:**
- Generate valid, idiomatic `.git-claw.toml`.
- Support `--yes` / `-y` flag to accept all inferred defaults without prompting.
- If stdin is not a tty, accept inferred defaults automatically.
- Detect existing files without altering repository source code.

**Never:**
- Never execute external git shell commands (Rule 26).
- Never fail silently when unable to write configuration.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Repo with `.env` and `docker-compose.yml` | Has `APP_PORT=8000`, `postgres` service | `.git-claw.toml` contains `[files]`, `[ports]`, `[docker]` | Ok(()) |
| Empty repository | No `.env` or compose files | Clean minimal `.git-claw.toml` generated | Ok(()) |
| Non-interactive invocation | `git claw init --yes` | Writes config immediately without user prompt | Ok(()) |
| Existing `.git-claw.toml` present | Already exists | Overwritten or updated when confirmed/`--yes` | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/cli/args.rs` -- Add `Commands::Init { yes: bool }`.
- `src/workflow/init.rs` -- Implement `run_init_workflow` repository inspection and TOML generator.
- `src/workflow/mod.rs` -- Expose init workflow.
- `src/main.rs` -- Wire `Commands::Init`.
- `tests/test_init_workflow.rs` -- Integration tests for `git claw init` and `--yes`.

## Tasks & Acceptance

**Execution:**
- [x] `src/cli/args.rs` -- Add `Commands::Init` subcommand.
- [x] `src/workflow/init.rs` -- Implement repository stack scanner and `.git-claw.toml` generator.
- [x] `src/workflow/mod.rs` & `src/main.rs` -- Wire init workflow.
- [x] `tests/test_init_workflow.rs` -- Integration tests for repository stack scanning.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- `git claw init --yes` in a repository with `.env` and `docker-compose.yml` generates `.git-claw.toml` with detected ports and files.
- Command exits with 0 on success.

## Implementation Notes
- Implemented `run_init_workflow`, `scan_env_files`, `scan_compose_file`, `extract_ports_from_env_files`, `scan_compose_details`, and `generate_init_toml` in `src/workflow/init.rs`.
- Added `Commands::Init { yes: bool }` to `src/cli/args.rs` and dispatched in `src/main.rs`.
- Tested in `tests/test_init_workflow.rs`.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test --test test_init_workflow` -- expected: all tests pass

## Auto Run Result
- `cargo test --test test_init_workflow`: 3/3 passed
- `cargo clippy`: 0 errors, 0 warnings
- `cargo fmt -- --check`: clean

