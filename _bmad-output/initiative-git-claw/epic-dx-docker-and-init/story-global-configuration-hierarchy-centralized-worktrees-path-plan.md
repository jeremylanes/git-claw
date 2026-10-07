---
title: 'Global configuration hierarchy & centralized worktrees path'
type: 'feature'
ticket: 1
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

**Problem:** Worktrees previously defaulted to sibling directories (`../<repo>-worktrees/<name>`), cluttering project parent folders. Furthermore, users could not define user-level default preferences across multiple repositories without duplicating `.git-claw.toml`.

**Approach:** Implement a two-tier configuration hierarchy:
1. User-level global config at `~/.git-claw/config.toml` (with testable override via `GIT_CLAW_GLOBAL_CONFIG`).
2. Centralized default worktree root at `~/.git-claw/worktrees/<repo-name>/<worktree-name>`.
3. Project-level `.git-claw.toml` overrides global settings and defaults.
4. Support tilde expansion (`~/...`) in configured `worktree_root`.

## Boundaries & Constraints

**Always:**
- Default worktree root resolves to `~/.git-claw/worktrees/<repo-name>` (expanded with `$HOME`).
- Local `.git-claw.toml` takes precedence over global `~/.git-claw/config.toml`.
- Global `~/.git-claw/config.toml` takes precedence over hardcoded defaults.
- Missing global config is silently ignored without errors.
- Tilde prefix `~/` in `worktree_root` is automatically expanded to the user's home directory.

**Never:**
- Never fail if `~/.git-claw/config.toml` does not exist.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| No local or global config | Missing both files | Default `worktree_root` is `$HOME/.git-claw/worktrees/<repo>` | Ok(Config) |
| Global config only | `~/.git-claw/config.toml` sets `main_branch = "trunk"` | `main_branch` is "trunk", default worktrees under `~/.git-claw/worktrees/<repo>` | Ok(Config) |
| Local overrides global | Global sets `main_branch = "trunk"`, local sets `main_branch = "dev"` | `main_branch` is "dev" | Ok(Config) |
| Custom worktree_root with tilde | Local sets `worktree_root = "~/custom/<repo>"` | `worktree_root` expanded to `$HOME/custom/<repo>` | Ok(Config) |

</frozen-after-approval>

## Code Map

- `src/core/config.rs` -- Two-tier config loader, tilde expansion, centralized default worktree root, and merging logic.
- `tests/test_config.rs` -- Updated unit tests for centralized path, global config merging, and tilde expansion.

## Tasks & Acceptance

**Execution:**
- [x] `src/core/config.rs` -- Implement `default_worktree_root`, `expand_home`, `load_global_or_empty`, and merge logic.
- [x] `tests/test_config.rs` -- Update and add tests verifying centralized worktree path and two-tier config merging.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- Given no config, default worktree location is `$HOME/.git-claw/worktrees/<repo>/<name>`.
- Given global config and local config, local values override global values.

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
