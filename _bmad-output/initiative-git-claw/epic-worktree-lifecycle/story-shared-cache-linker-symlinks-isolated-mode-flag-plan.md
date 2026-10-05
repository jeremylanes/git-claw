---
title: 'Shared cache linker (symlinks) & isolated mode flag'
type: 'feature'
ticket: 3
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

**Problem:** Worktrees need to share heavy dependencies/cache folders (such as `node_modules` or `target`) without duplicating disk space, while also allowing developers to opt-out via `--isolated` or `cache.strategy = "isolated"`.

**Approach:** Implement `infra::fs` (or `infra::cache`) to link directories declared in `cache.directories` using relative POSIX symlinks from the worktree to the primary repository root, skipping linking if `--isolated` is enabled or strategy is `isolated`.

## Boundaries & Constraints

**Always:**
- Use POSIX symlinks (`std::os::unix::fs::symlink`).
- Ensure target directories exist in the primary repo before symlinking (create them if missing).
- Skip symlink creation entirely if `isolated` flag is true or `strategy == CacheStrategy::Isolated`.
- If a file or symlink already exists at the destination in the worktree, handle or overwrite safely.

**Never:**
- Do not fail fatal if cache directories list is empty (default zero-config behavior).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Shared strategy | directories `["node_modules", "target"]`, `isolated = false` | Symlinks created in worktree pointing to primary repo dirs | Ok(2) |
| Isolated flag | `isolated = true` | Symlink creation skipped | Ok(0) |
| Non-existent primary dir | Target directory does not exist in primary repo | Creates primary directory and links to it | Ok(1) |

</frozen-after-approval>

## Code Map

- `src/infra/fs.rs` -- Symlink creation logic: `link_shared_cache_directories(repo_root, worktree_path, cache_config, isolated)`.
- `src/infra/mod.rs` -- Re-export `link_shared_cache_directories`.
- `tests/test_cache.rs` -- Integration tests for symlink creation and isolated flag.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/fs.rs` -- Implement cache symlinking logic -- Links cache directories using POSIX symlinks when not isolated.
- [x] `src/infra/mod.rs` -- Re-export fs/cache helpers -- Exposes cache linker to workflow.
- [x] `tests/test_cache.rs` -- Integration test for cache symlinking -- Tests shared cache linking and isolated bypass.

**Acceptance Criteria:**
- Given cache directory `["target"]` and `isolated = false`, when linking, `worktree/target` is a symlink pointing to `repo/target`.
- Given `isolated = true`, when linking, no symlinks are created in worktree.

## Implementation Notes
- Implemented `link_shared_cache_directories` in `src/infra/fs.rs` using POSIX symlinks.
- Verified `--isolated` flag and `CacheStrategy::Isolated` bypasses symlink creation.
- Re-exported cache linker in `src/infra/mod.rs`.
- Added integration tests in `tests/test_cache.rs`.

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
Implemented shared cache directory linker creating POSIX symlinks from worktree to primary repository directories, with support for the `--isolated` flag and `isolated` strategy.

### Files changed
- `src/infra/fs.rs` - Shared cache symlinker.
- `src/infra/mod.rs` - Re-exports for cache linker.
- `tests/test_cache.rs` - Integration test suite verifying symlinks and isolated bypass.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 36 passed, 0 failed, 0 warnings.
- `cargo clippy`: 0 warnings.
- `cargo fmt -- --check`: Clean formatting.

### Residual risks
- None. POSIX symlinking tested and verified.
