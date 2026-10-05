---
title: 'End-to-end integration test suite & binary packaging'
type: 'feature'
ticket: 5
created: '2026-10-06'
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
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** We need a unified end-to-end test suite (`tests/cli_e2e_test.rs`) that validates the complete developer journey: zero-config start, port assignment, running commands inside the worktree, status dashboard listing, worktree finishing and merging back to trunk, release tagging, and spike experimentation/abandonment. Furthermore, release binary configuration must be configured in `Cargo.toml` so the final binary is lean (< 15MB) and self-contained.

**Approach:**
1. Configure `Cargo.toml` with release profile optimizations (`opt-level = "z"`, `lto = true`, `codegen-units = 1`, `strip = true`).
2. Implement `tests/cli_e2e_test.rs` executing the entire workflow end-to-end via CLI binary invocations.
3. Validate binary build size.

## Boundaries & Constraints

**Always:**
- Black-box CLI testing via `assert_cmd`.
- Cover user journeys UJ-1 (feature lifecycle), UJ-2 (spike abandonment), and UJ-3 (dashboard & tooling).
- Binary size strictly under 15MB.

**Never:**
- Do not bypass Git CLI in e2e tests; use real temporary Git repositories.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Full Developer Lifecycle | Init repo -> start feature -> run test -> list -> cd -> finish -> tag | All commands succeed, changes merged, tag created | Ok(()) |
| Spike Experiment Lifecycle | Start spike -> commit -> list -> drop | Spike cleaned, branch deleted, trunk unchanged | Ok(()) |
| Release build | `cargo build --release` | Produces binary <= 15MB | Ok(()) |

</frozen-after-approval>

## Code Map

- `Cargo.toml` -- Release profile optimizations for lean binary packaging.
- `tests/cli_e2e_test.rs` -- Complete end-to-end integration test suite.

## Tasks & Acceptance

**Execution:**
- [x] `Cargo.toml` -- Configure release profile for compact binary size (< 15MB).
- [x] `tests/cli_e2e_test.rs` -- Write comprehensive end-to-end lifecycle integration test.
- [x] Verify release build and test suite.

**Acceptance Criteria:**
- `cargo test --test cli_e2e_test` executes and passes all integration flows.
- `cargo build --release` produces binary well under 15MB (871KB).

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test` -- expected: all unit and integration tests pass

### Auto Run Result
- `cargo fmt -- --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS (0 warnings)
- `cargo test`: PASS (61 passed: 13 unit tests, 48 integration tests across 18 suites)
- `cargo build --release`: PASS (release binary size: 871KB <= 15MB)
