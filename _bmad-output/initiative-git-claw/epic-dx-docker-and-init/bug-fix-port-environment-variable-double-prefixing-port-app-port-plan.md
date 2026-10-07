---
title: 'Fix port environment variable double prefixing (PORT_APP_PORT -> APP_PORT)'
type: 'fix'
ticket: 6
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
context: ['_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md', '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md', '_bmad-output/initiative-git-claw/change-runtime-fixes-and-init/change-runtime-fixes-and-init.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** In `core/port.rs` and `infra/env_file.rs`, `format_port_env_key` unconditionally prepends `PORT_` to any key. If a port key is already named `APP_PORT` or `PORT_HTTP`, it becomes double-prefixed as `PORT_APP_PORT` or `PORT_PORT_HTTP` in `.env.worktree` and appended environment variables.

**Approach:**
1. Update `format_port_env_key` in `src/core/port.rs`:
   If the uppercase sanitized key starts with `PORT_`, ends with `_PORT`, or equals `PORT`, preserve it without adding another `PORT_` prefix.
   Otherwise (e.g. `web`, `api`), prepend `PORT_`.
2. In `src/infra/env_file.rs`, ensure both formatted and raw keys are indexed, and append unmatched ports using `format_port_env_key`.

## Boundaries & Constraints

**Always:**
- `format_port_env_key("APP_PORT")` returns `"APP_PORT"`.
- `format_port_env_key("web")` returns `"PORT_WEB"`.
- `format_port_env_key("PORT")` returns `"PORT"`.
- `format_port_env_key("PORT_HTTP")` returns `"PORT_HTTP"`.
- Handle case-insensitivity (e.g. `app_port` -> `APP_PORT`).

**Never:**
- Never regress existing port offset calculation or overflow checks.

## I/O & Edge-Case Matrix

| Scenario | Input | Expected Output | Error Handling |
|----------|-------|-----------------|----------------|
| Already ends with `_PORT` | `"APP_PORT"` | `"APP_PORT"` | Ok |
| Lowercase ending with `_port` | `"db_port"` | `"DB_PORT"` | Ok |
| Already starts with `PORT_` | `"PORT_GATEWAY"` | `"PORT_GATEWAY"` | Ok |
| Equals `PORT` | `"PORT"` | `"PORT"` | Ok |
| Generic service name | `"web"` | `"PORT_WEB"` | Ok |
| Hyphenated name | `"my-service"` | `"PORT_MY_SERVICE"` | Ok |

</frozen-after-approval>

## Code Map

- `src/core/port.rs` -- Update `format_port_env_key` and unit tests.
- `src/infra/env_file.rs` -- Ensure `update_env_content_with_ports` uses `format_port_env_key`.
- `tests/test_slot_port.rs` -- Add regression tests for `APP_PORT` and `PORT_` keys.

## Tasks & Acceptance

**Execution:**
- [x] `src/core/port.rs` -- Update `format_port_env_key` to avoid double-prefixing.
- [x] `src/infra/env_file.rs` -- Integrate with `update_env_content_with_ports`.
- [x] `tests/test_slot_port.rs` -- Verify `format_port_env_key("APP_PORT") == "APP_PORT"`.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- `format_port_env_key("APP_PORT")` returns `"APP_PORT"`.
- `format_port_env_key("web")` returns `"PORT_WEB"`.

## Implementation Notes
- Updated `format_port_env_key` in `src/core/port.rs` to detect if the sanitized key is `PORT`, starts with `PORT_`, or ends with `_PORT`, preserving it without double-prefixing.
- Integrated `format_port_env_key` in `src/infra/env_file.rs` for both matching and appending port keys.
- Regression tests added in `src/core/port.rs` and `tests/test_slot_port.rs`.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test --test test_slot_port` -- expected: all tests pass

## Auto Run Result
- `cargo test --test test_slot_port`: 5/5 passed
- `cargo test --test test_files_copy`: 1/1 passed
- `cargo clippy`: 0 errors, 0 warnings
- `cargo fmt -- --check`: clean

