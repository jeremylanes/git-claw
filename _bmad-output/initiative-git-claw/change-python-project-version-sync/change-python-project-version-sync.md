# Sprint Change Proposal: Python / uv Project Version Synchronization on Release Tag

**Date:** 2026-10-17  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request: In Python/Django projects using `uv` (or standard PEP 621), `pyproject.toml` (or `project.toml`) should automatically have its version bumped by `git claw tag <version>`, synchronizing `uv.lock` if present, creating the release commit, and tagging.

---

## 1. Issue Summary

`git-claw` previously added automatic version synchronization for Rust projects (`Cargo.toml` and `Cargo.lock`). In Python and Django projects, particularly those using modern tooling like `uv` or Poetry, project metadata and versioning are specified in `pyproject.toml` (under `[project]` or `[tool.poetry]`), and optionally `project.toml`.

When running `git claw tag <version>` on a Python project, `git-claw` should automatically detect `pyproject.toml` or `project.toml`, update the SemVer version string, refresh `uv.lock` if present using `uv lock`, stage and commit the changes as `chore: release <version>`, and create the release tag.

---

## 2. Requirement Specification

1. **Manifest Detection & Version Update:**
   - Detect `pyproject.toml` and/or `project.toml` in the repository root.
   - Inspect table sections `[project]` (PEP 621 standard used by `uv`, `hatch`, `flit`, `setuptools`) and `[tool.poetry]`.
   - If `version = "..."` exists and differs from canonical SemVer (e.g. `v0.2.1` -> `0.2.1`):
     - Update the version string while strictly preserving all comments, indentation, and formatting.
2. **Lockfile Synchronization:**
   - If `uv.lock` exists in the repository root, invoke `uv lock` (non-blocking, ignore errors if `uv` is unavailable).
3. **Commit & Tag:**
   - Stage modified manifest files (`pyproject.toml`, `project.toml`, and `uv.lock` if present) along with any other synchronized files.
   - Commit changes with message `chore: release <version>`.
   - Output success notification: `  → Updated pyproject.toml to version '0.2.1'`.
   - Create annotated release tag on the resulting commit.
4. **Interoperability & Safety:**
   - Non-Python repositories or files without version fields are unaffected.
   - If version already matches, file writing and commit are skipped.
   - If repository has uncommitted changes, abort with `DirtyWorkingTree` before modifying files.

---

## 3. Impact Analysis

- **PRD Impact:** Amends FR-9 (SemVer release tagging).
- **Architecture Impact:**
  - `src/workflow/tag.rs`: Generalize TOML version updating (`update_toml_version`, `update_pyproject_toml_version`), detect `pyproject.toml` / `project.toml` and `uv.lock`.
  - `tests/test_tag_workflow.rs`: Integration tests verifying `pyproject.toml` and `project.toml` version updates, lockfile inclusion, and commit tagging.
  - `README.md`: Document automatic version synchronization support for both Rust (`Cargo.toml`) and Python (`pyproject.toml` / `project.toml`).

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Clean implementation, comprehensive integration tests, Sphinx documentation with examples, all tests passing, and updated binary installed.
