# Proposal: Multi-Worktree Test Database Isolation per Slot

## Problem Statement
When running multiple worktrees concurrently against a shared database service (such as PostgreSQL in `shared_services = ["redis", "postgres"]`), testing frameworks (like Django's test runner, pytest-django, etc.) default to a single fixed test database name (e.g. `test_tune`).
When two worktrees execute tests simultaneously, both attempts to create or drop `test_tune` on the same database server collide, causing:
```text
Got an error creating the test database: database "test_tune" already exists
Destroying old test database for alias 'default' ('test_tune')...
psycopg.OperationalError: connection to server failed: FATAL: database "test_tune" does not exist
```

## Proposed Solution (Option 1: Slot-Scoped Test Database)
1. **Deterministic Slot Database Naming**:
   - For any worktree with Slot ID `N`, the test database name is deterministically set to `test_{base_db}_{slot_id}` (e.g. `test_tune_1`, `test_tune_2`).
2. **Environment File Injection**:
   - In `.env.worktree`, inject `TEST_DATABASE_NAME=test_{base_db}_{slot_id}` and `CLAW_TEST_DB=test_{base_db}_{slot_id}`.
   - In `copy_and_merge_untracked_files` (which copies project `.env` files like `src/tune/settings/.env`):
     - Detect `DATABASE_NAME`, `POSTGRES_DB`, or `MYSQL_DATABASE` to infer the project database name.
     - If `TEST_DATABASE_NAME` is already declared in `.env`, update it in-place to `test_{base_db}_{slot_id}`.
     - If `TEST_DATABASE_NAME` is not declared, automatically append `TEST_DATABASE_NAME=test_{base_db}_{slot_id}`.
3. **Application Integration (e.g. Django `databases.py`)**:
   - In Django settings, configure:
     ```python
     'TEST': {
         'NAME': env('TEST_DATABASE_NAME', default=None),
     }
     ```
   - When running in a worktree, Django isolates each slot into its own test database. Outside a worktree, it falls back cleanly to default.
4. **Zero Regressions & Shared Efficiency**:
   - Preserves single PostgreSQL container sharing (minimal memory footprint).
   - Allows unlimited concurrent test runs across multiple worktrees.
