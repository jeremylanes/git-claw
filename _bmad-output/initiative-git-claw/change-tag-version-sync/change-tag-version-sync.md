# Sprint Change Proposal: Cargo.toml Version Synchronization on Release Tag

**Date:** 2026-10-17  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request: `git claw tag <version>` should automatically update the package version in `Cargo.toml` so releases and CI automated crates.io publications are always in sync.

---

## 1. Issue Summary

Currently, `git claw tag <version>` validates that the trunk working copy is clean and creates an annotated Git tag on `main`. However, in Rust projects, developers must manually edit `Cargo.toml` to bump the version string prior to running `git claw tag`. If forgotten, the tag points to a commit containing an outdated `version` in `Cargo.toml`, causing automated crates.io CI publishing to fail with `crate version already exists`.

Automating the version synchronization inside `git claw tag` ensures that the release commit and tag always reflect the exact version specified.

---

## 2. Requirement Specification

1. **Version Extraction & Comparison:**
   - Extract the canonical SemVer string (e.g. `v0.2.1` -> `0.2.1`).
   - If `Cargo.toml` exists in the repository root:
     - Check if the `version` field under `[package]` already matches `0.2.1`.
     - If it differs, update `version = "0.2.1"` in `Cargo.toml` while preserving all comments, sections, and formatting.
2. **Lockfile & Git Commit:**
   - If `Cargo.lock` exists, execute a lightweight `cargo check` to synchronize the lockfile.
   - Stage and commit the updated `Cargo.toml` (and `Cargo.lock` if present) with commit message `chore: release <version>`.
   - Output confirmation: `  → Updated Cargo.toml to version '0.2.1'`.
3. **Tagging:**
   - Create the annotated tag on the resulting release commit.
4. **Non-Rust Repositories:**
   - If `Cargo.toml` does not exist, proceed with standard tag creation without error (100% backward compatible).

---

## 3. Impact Analysis

- **PRD Impact:** Amends FR-9 (SemVer release tagging).
- **Architecture Impact:**
  - `src/infra/git.rs`: Add `git_commit_files(repo_root: &Path, files: &[&str], message: &str) -> Result<(), GitError>`.
  - `src/workflow/tag.rs`: Implement `update_cargo_toml_version(content: &str, new_version: &str) -> Option<String>`, synchronize `Cargo.lock`, commit, and tag.
  - `tests/test_tag_workflow.rs`: Integration tests verifying automatic `Cargo.toml` update, commit, and tag creation.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Clean implementation, test suite passing, and installed binary.
