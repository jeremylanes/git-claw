---
title: 'Zero-config parser & .git-claw.toml loader'
type: 'feature'
ticket: 2
created: '2026-10-06'
status: 'built'
followup_review_recommended: false
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
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `git-claw` needs to load project configurations from `.git-claw.toml` or provide zero-config deterministic defaults if the file is absent or partially specified.

**Approach:** Implement `core::config::Config` (and nested structs `ProjectConfig`, `HooksConfig`, `CacheConfig`, `CacheStrategy`) in `src/core/config.rs` with `serde` deserialization, default values, error reporting for syntax errors with line numbers, and a loader function that accepts an optional repository root / repo name.

## Boundaries & Constraints

**Always:**
- Keep `core/config.rs` pure domain logic (no Git CLI execution or OS side-effects).
- Fallback to `main_branch = "main"`, `worktree_root = "../<repo_name>-worktrees"`, empty ports, empty hooks, and `cache.strategy = "shared"`.
- If invalid TOML syntax is passed, return a structured error containing syntax details and line information.

**Never:**
- Do not make configuration mutable during workflow execution.
- Do not import `infra/` or `workflow/` from `core/`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Missing file | Path does not exist | Returns default `Config` with `main_branch="main"`, `worktree_root="../<repo>-worktrees"` | Ok(Config::default_for_repo(...)) |
| Full valid TOML | Valid `.git-claw.toml` with project, ports, hooks, cache | Parsed `Config` matches all specified values | Ok(Config) |
| Partial TOML | Only `[project]` or `[ports]` defined | Unspecified sections populate with deterministic defaults | Ok(Config) |
| Malformed TOML | Invalid syntax (e.g. unclosed string) | Returns `ConfigError::ParseError` with line and error details | Err(ConfigError::ParseError) |

</frozen-after-approval>

## Code Map

- `src/core/config.rs` -- Config data structures (`Config`, `ProjectConfig`, `HooksConfig`, `CacheConfig`, `CacheStrategy`) and parser methods.
- `src/core/error.rs` -- Domain error types including `ConfigError`.
- `src/core/mod.rs` -- Exports `config` and `error` modules.
- `tests/test_config.rs` -- Unit and integration tests for zero-config defaults, partial parsing, and syntax errors.

## Tasks & Acceptance

**Execution:**
- [x] `src/core/error.rs` -- Create domain error definitions -- Defines `ConfigError` with thiserror.
- [x] `src/core/config.rs` -- Implement configuration models and parsing -- Supports full parsing, partial overrides, zero-config defaults.
- [x] `src/core/mod.rs` -- Expose config and error modules -- Re-exports public domain configuration API.
- [x] `tests/test_config.rs` -- Test configuration parsing and fallbacks -- Verifies absent file, valid TOML, partial TOML, and syntax errors.

**Acceptance Criteria:**
- Given no `.git-claw.toml` file, when loading config for repo `my-repo`, then default config is returned with `main_branch = "main"` and `worktree_root = "../my-repo-worktrees"`.
- Given a valid full `.git-claw.toml`, when parsing, then all custom values (ports, hooks, cache directories) are populated accurately.
- Given a partial `.git-claw.toml` containing only `[ports]`, when parsing, then unspecified sections fall back to defaults.
- Given a malformed TOML string, when parsing, then an explicit error is returned describing the syntax issue.

## Implementation Notes
- Implemented `Config`, `ProjectConfig`, `HooksConfig`, `CacheConfig`, and `CacheStrategy` in `src/core/config.rs`.
- Implemented `ConfigError` in `src/core/error.rs` for domain-level error handling.
- Exported crate as library (`src/lib.rs`) and binary (`src/main.rs`).
- Added 5 unit/integration tests in `tests/test_config.rs` covering absent files, full valid TOML, partial configs, and syntax error reporting.

## Plan Change Log

## Review Triage Log

### 2026-10-06 — Review pass
- verdicts: 0 findings — high 0, medium 0, low 0, false 0, maybe-false 0
- findings: []

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test` -- expected: all unit and integration tests pass

## Auto Run Result

### Summary of implemented change
Implemented zero-config parser and loader for `.git-claw.toml` with strongly typed domain models in `core/config.rs`, handling missing files with deterministic defaults and reporting malformed syntax.

### Files changed
- `src/lib.rs` - Library root exporting modules for binary and integration tests.
- `src/core/mod.rs` - Re-exports domain configuration and error types.
- `src/core/error.rs` - Domain error types (`ConfigError`).
- `src/core/config.rs` - Strongly-typed configuration structures and TOML parsing logic.
- `src/main.rs` - Updated entrypoint referencing `git_claw` modules.
- `tests/test_config.rs` - Test suite for configuration loading, partial fallback, and error handling.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 11 passed, 0 failed, 0 warnings.
- `cargo check`: Passed with 0 errors and 0 warnings.

### Residual risks
- None. All requirements and edge-cases in matrix verified.
