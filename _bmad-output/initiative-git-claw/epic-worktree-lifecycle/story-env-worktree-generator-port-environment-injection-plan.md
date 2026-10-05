---
title: '.env.worktree generator & port environment injection'
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

**Problem:** Worktrees need local environment configuration for Docker Compose, services, and port offsets to run side-by-side without network port or container name collisions.

**Approach:** Implement `infra::env_file` to generate `.env.worktree` inside the worktree directory containing `CLAW_SLOT_ID`, `CLAW_WORKTREE_NAME`, `CLAW_WORKTREE_PATH`, sanitized `COMPOSE_PROJECT_NAME` (`<repo>_<name>_<slot_id>`), and all computed `PORT_<KEY>` variables.

## Boundaries & Constraints

**Always:**
- Format `.env.worktree` with standard `KEY=VALUE` shell/dotenv syntax.
- Sanitize `COMPOSE_PROJECT_NAME` to lowercase alphanumerics and underscores.
- Include all calculated `PORT_<KEY>` variables from `core::port::calculate_effective_ports`.

**Never:**
- Do not write unescaped newlines or syntax errors into `.env.worktree`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Standard env generation | repo `my-repo`, name `auth`, slot `2`, path `/path/to/wt`, ports `web: 3000` | Writes `.env.worktree` with `CLAW_SLOT_ID=2`, `COMPOSE_PROJECT_NAME=my_repo_auth_2`, `PORT_WEB=3002` | Ok(()) |
| Special characters in repo/name | repo `My-Repo.V1`, name `user-feat` | Sanitizes `COMPOSE_PROJECT_NAME=my_repo_v1_user_feat_1` | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/infra/env_file.rs` -- `.env.worktree` generation and `COMPOSE_PROJECT_NAME` sanitizer.
- `src/infra/mod.rs` -- Re-export `write_env_worktree` and `sanitize_compose_project_name`.
- `tests/test_env_file.rs` -- Integration tests for `.env.worktree` file writing and variable formatting.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/env_file.rs` -- Implement env file writer and compose project name sanitizer -- Generates `.env.worktree`.
- [x] `src/infra/mod.rs` -- Re-export env file generator -- Exposes `write_env_worktree`.
- [x] `tests/test_env_file.rs` -- Integration test for .env.worktree generator -- Verifies content and compose project name sanitization.

**Acceptance Criteria:**
- Given worktree parameters and ports `{"web": 3000}`, when `write_env_worktree` is called, `.env.worktree` is written with all expected keys and values.
- Given repository `Repo-App` and worktree `my-feat`, `COMPOSE_PROJECT_NAME` is formatted as `repo_app_my_feat_<slot_id>`.

## Implementation Notes
- Implemented `sanitize_compose_project_name` and `write_env_worktree` in `src/infra/env_file.rs`.
- Re-exported functions in `src/infra/mod.rs`.
- Added unit tests in `src/infra/env_file.rs` and integration tests in `tests/test_env_file.rs`.

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
Implemented `.env.worktree` generator and Docker Compose project name sanitizer injecting slot ID, worktree name, path, compose project name, and effective port offsets.

### Files changed
- `src/infra/env_file.rs` - `.env.worktree` formatter and sanitizer.
- `src/infra/mod.rs` - Re-exports for env file writer.
- `tests/test_env_file.rs` - Integration test suite verifying file writing and variables.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 33 passed, 0 failed, 0 warnings.
- `cargo clippy`: 0 warnings.
- `cargo fmt -- --check`: Clean formatting.

### Residual risks
- None. Formatting matches dotenv standards and compose project naming rules.
