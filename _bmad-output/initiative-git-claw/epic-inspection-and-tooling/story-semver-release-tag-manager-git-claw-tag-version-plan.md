---
title: 'SemVer release tag manager (git claw tag <version>)'
type: 'feature'
ticket: 4
created: '2026-10-06'
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
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Releasing software from trunk requires strict SemVer tagging discipline. Developers and automation need `git claw tag <version>` to validate the tag format, ensure the main repository is clean, and create an annotated Git tag directly on the main trunk.

**Approach:** Implement `validate_semver` in `src/core/semver.rs` and `workflow::tag` in `src/workflow/tag.rs`. The command verifies that `version` conforms to SemVer (supporting standard versions and optional `v` prefix), ensures the primary working tree is clean via `git_is_clean`, and calls `git tag -a <version> <main_branch> -m "Release <version>"`.

## Boundaries & Constraints

**Always:**
- Strictly validate SemVer format (e.g., `v1.2.0`, `1.2.0`, `v0.1.0-rc.1`).
- Verify the working tree is clean before creating a tag.
- Create an annotated Git tag on the configured `main_branch`.

**Never:**
- Never create a tag if the working tree has uncommitted modifications.
- Never create a tag if the version string is malformed or non-SemVer.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Valid SemVer tag | Clean trunk, `git claw tag v1.0.0` | Creates annotated tag `v1.0.0` on main | Ok(()) |
| Dirty trunk | Uncommitted changes in main repo | Aborts without tagging | Err(WorkflowError::DirtyWorkingTree) |
| Invalid SemVer | Malformed string `v1.0` or `latest` | Rejects tag creation with descriptive error | Err(WorkflowError::InvalidSemVer) |

</frozen-after-approval>

## Code Map

- `src/core/semver.rs` -- Pure SemVer validation function.
- `src/infra/git.rs` -- Helper `git_create_annotated_tag`.
- `src/workflow/tag.rs` -- Implementation of `tag_release`.
- `src/workflow/mod.rs` -- Re-export `tag_release` and `TagOptions`.
- `src/main.rs` -- Wire `Commands::Tag { version }`.
- `tests/test_tag_workflow.rs` -- Integration tests for SemVer validation, clean check, and annotated tag creation.

## Tasks & Acceptance

**Execution:**
- [x] `src/core/semver.rs` -- Implement `validate_semver` function with unit tests.
- [x] `src/infra/git.rs` -- Implement `git_create_annotated_tag`.
- [x] `src/workflow/tag.rs` -- Implement `tag_release` workflow.
- [x] `src/workflow/mod.rs` -- Expose `tag_release`.
- [x] `src/main.rs` -- Wire `Commands::Tag`.
- [x] `tests/test_tag_workflow.rs` -- Integration tests for release tag workflow.

**Acceptance Criteria:**
- Given clean trunk and valid SemVer string `v1.0.0`, `git claw tag v1.0.0` creates an annotated tag on main.
- Given dirty trunk, `git claw tag` aborts with `DirtyWorkingTree`.
- Given invalid version string, `git claw tag` aborts with `InvalidSemVer`.

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
- `cargo test`: PASS (59 passed: 13 unit tests, 46 integration tests across 17 suites)
