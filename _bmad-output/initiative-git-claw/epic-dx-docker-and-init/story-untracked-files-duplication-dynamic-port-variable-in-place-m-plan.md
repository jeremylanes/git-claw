---
title: 'Untracked files duplication & dynamic port variable in-place merging'
type: 'feature'
ticket: 2
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

**Problem:** Real-world containerized stacks (e.g. `tune`) rely on untracked configuration files (like `./src/tune/settings/.env`) that are gitignored. When creating a worktree with `git worktree add`, these files are missing, causing immediate service launch failures. Additionally, compose stacks look for specific variables (e.g. `APP_PORT=80`) rather than generic `PORT_WEB=80`.

**Approach:** Implement `[files] copy = [...]` in `src/infra/env_file.rs` and wire it into `workflow::start`.
When starting a worktree:
1. Copy declared untracked files from the primary repository to the worktree path.
2. In copied `.env` files, update matched port variables in-place with their effective calculated offsets (`Effective Port = Base Port + Slot ID`).
3. Append any declared port variables that were not already present in the file.

## Boundaries & Constraints

**Always:**
- Preserve non-port lines, formatting, and comments in copied files.
- Calculate effective port as `Base Port + Slot ID`.
- Create destination subdirectories if needed (`create_dir_all`).
- Non-existing declared files should not crash worktree creation (log/skip gracefully).

**Never:**
- Do not modify original files in the primary repository.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Copy `.env` with existing port | `APP_PORT=80`, Slot 1, `ports.APP_PORT = 80` | Copied file has `APP_PORT=81` in-place | Ok(()) |
| Copy `.env` missing port | Copied file has no `DEBUG_PORT`, `ports.DEBUG_PORT = 9000` | `DEBUG_PORT=9001` appended to copied file | Ok(()) |
| Non-env file copied | Declared `config.json` copied | Copied verbatim without alterations | Ok(()) |
| Source file absent | Declared `missing.env` does not exist | Skipped gracefully without aborting start | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/infra/env_file.rs` -- Implement `copy_and_merge_untracked_files` and in-place port updater.
- `src/workflow/start.rs` -- Invoke `copy_and_merge_untracked_files` during worktree creation.
- `tests/test_files_copy.rs` -- Integration tests for untracked file duplication and in-place variable merging.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/env_file.rs` -- Implement `copy_and_merge_untracked_files`.
- [x] `src/workflow/start.rs` -- Wire file copying in start workflow.
- [x] `tests/test_files_copy.rs` -- Integration test for `[files] copy` and dynamic port merging.
- [x] Verify test suite passes without regressions.

**Acceptance Criteria:**
- Given `[files] copy = [".env"]` and `[ports] APP_PORT = 80`, starting a worktree copies `.env` and updates `APP_PORT=81` in the worktree.
- Primary repository `.env` remains untouched.

## Implementation Notes
- Implemented `update_env_content_with_ports` and `copy_and_merge_untracked_files` in `src/infra/env_file.rs`.
- Preserves comments and non-port lines, calculates dynamic ports (`base + slot_id`), updates existing keys or appends missing keys.
- Wired into `src/workflow/start.rs` right after worktree creation.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test --test test_files_copy` -- expected: all unit and integration tests pass

## Auto Run Result
- `cargo test --test test_files_copy`: Passed (1/1 tests passing)
- `cargo check`: Passed with 0 errors, 0 warnings

