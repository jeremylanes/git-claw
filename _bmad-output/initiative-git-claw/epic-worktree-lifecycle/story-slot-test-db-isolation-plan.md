# Story Plan: Multi-Worktree Test Database Isolation per Slot

## Ticket 19
- **Epic**: `epic-worktree-lifecycle`
- **Component**: Environment variable injection & `.env` merging (`src/infra/env_file.rs`, `src/workflow/start.rs`)

## Roles
- **John (Product Manager)**: Requirements specification & Story plan.
- **Amelia (Senior Systems Developer)**: Core implementation in `src/infra/env_file.rs` and `src/workflow/start.rs`, unit tests, and integration.
- **Murat (QA & Test Architect)**: Independent verification, regression testing, multi-slot concurrency simulation, and QA Sign-Off.

## Implementation Details
1. **`src/infra/env_file.rs`**:
   - In `write_env_worktree`:
     - Add `TEST_DATABASE_NAME=test_{db_name}_{slot_id}` to `.env.worktree`.
     - Add `CLAW_TEST_DB=test_{db_name}_{slot_id}`.
   - In `update_env_content_with_ports`:
     - Detect database name from `DATABASE_NAME`, `POSTGRES_DB`, or fallback to repo name.
     - Replace existing `TEST_DATABASE_NAME` or append `TEST_DATABASE_NAME=test_{db_name}_{slot_id}`.
2. **`src/tune/settings/databases.py`**:
   - Activate `'TEST': {'NAME': env('TEST_DATABASE_NAME', default=None)}` in project `tune`.
3. **Tests (`tests/test_slot_db_isolation.rs`)**:
   - Test Slot 1 and Slot 2 generating `test_tune_1` and `test_tune_2`.
   - Test `.env` file updating in-place.
   - Verify full test suite passes with zero regressions (`cargo test`).
