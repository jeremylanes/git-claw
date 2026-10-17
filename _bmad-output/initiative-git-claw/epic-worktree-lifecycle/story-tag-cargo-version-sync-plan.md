---
title: 'Cargo.toml version synchronization on release tag (git claw tag <version>)'
type: 'feature'
ticket: 12
created: '2026-10-17'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-tag-version-sync/change-tag-version-sync.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Developers must manually update `Cargo.toml` prior to running `git claw tag <version>`. Omitting this leads to tags pointing to stale versions and causes crates.io automated publishing to fail.

**Approach:** In `workflow::tag`, detect if `Cargo.toml` is present in the repository root. If present and its `[package]` version differs from the tag version, update `Cargo.toml`, synchronize `Cargo.lock` if present, commit the version bump, and then create the annotated tag on `main`.

## Boundaries & Constraints

**Always:**
- Preserve all comments, formatting, and other sections in `Cargo.toml`.
- If `Cargo.toml` already matches the target version, skip the file edit and commit step.
- If the repository has uncommitted changes prior to running `git claw tag`, abort with `DirtyWorkingTree` before touching `Cargo.toml`.
- Document every function/struct in Sphinx/Rustdoc format with at least one usage example (Rule 22).
- Write minimal, clean, robust code (Rules 20, 30, 31).

**Never:**
- Do not run git commands directly in shell (Rule 26).
- Do not fail if `Cargo.toml` does not exist (non-Rust projects).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cargo.toml exists, version differs | `Cargo.toml` v0.2.0, run `git claw tag v0.2.1` | Updates `Cargo.toml` to 0.2.1, commits, creates tag v0.2.1 | Ok(()) |
| Cargo.toml exists, version matches | `Cargo.toml` v0.2.1, run `git claw tag v0.2.1` | Skips file edit, creates tag v0.2.1 directly | Ok(()) |
| Non-Rust repo (no Cargo.toml) | No `Cargo.toml` present | Creates tag v0.2.1 directly | Ok(()) |
| Dirty working copy | Uncommitted edits present | Aborts with `DirtyWorkingTree` | Err(DirtyWorkingTree) |

</frozen-after-approval>

## Code Map

- `src/infra/git.rs`: Function `git_commit_files(repo_root: &Path, files: &[&str], message: &str) -> Result<(), GitError>`.
- `src/workflow/tag.rs`: Function `update_cargo_toml_version(content: &str, new_version: &str) -> Option<String>`, integration into `tag_release`.
- `tests/test_tag_workflow.rs`: Integration tests for automatic `Cargo.toml` version update and commit.

## Tasks & Acceptance

**Execution:**
- [x] `src/infra/git.rs` -- Implement `git_commit_files`.
- [x] `src/workflow/tag.rs` -- Implement `update_cargo_toml_version` and wire into `tag_release`.
- [x] `tests/test_tag_workflow.rs` -- Add tests verifying version update and commit.
- [x] Run `cargo check` and `cargo test`.
- [x] Run `cargo install --path .`.

**Acceptance Criteria:**
- Running `git claw tag v0.3.0` in a Rust repo updates `Cargo.toml` to 0.3.0, commits it, and tags the commit.
- All tests pass with `cargo test`.
- Binary installed in `~/.cargo/bin/git-claw`.
