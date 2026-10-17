# Sprint Change Proposal: Automated Conventional Commit Tagging and Changelog Generation (Commitizen Mode)

**Date:** 2026-10-17  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request: `git claw tag` without arguments should automatically infer the next SemVer version from Conventional Commits (like Commitizen), update `CHANGELOG.md`, synchronize manifest versions (`Cargo.toml`, `pyproject.toml`, `project.toml`) and lockfiles (`Cargo.lock`, `uv.lock`), commit, and create the release tag.

---

## 1. Issue Summary

Currently, `git claw tag <version>` requires the user to specify an explicit SemVer version string (e.g., `v0.2.1`). In many modern workflows (notably Python/Django and Rust), developers prefer automated release tooling like Commitizen (`cz bump`) or Standard Version, where:
1. Commit messages follow the Conventional Commits specification (`feat:`, `fix:`, `feat!:`, `BREAKING CHANGE:`, etc.).
2. The release tool inspects commits since the latest Git tag.
3. The bump level (Major, Minor, Patch) is automatically calculated.
4. `CHANGELOG.md` is automatically updated or created.
5. All manifest files and lockfiles are updated and committed atomically.
6. The annotated Git tag is created.

Allowing `git claw tag` to be called with an optional version argument (`Option<String>`) provides the best of both worlds: zero-configuration automatic releases when omitted, and explicit version override when specified.

---

## 2. Requirement Specification

1. **CLI Interface:**
   - In `src/cli/args.rs`, modify `Commands::Tag`:
     `version: Option<String>` (optional).
   - If `version` is provided (e.g. `git claw tag v0.3.0`), retain current explicit tagging behavior.
   - If `version` is omitted (i.e. `git claw tag`), trigger Commitizen-style automated release.

2. **Conventional Commit Analysis & Bump Calculation:**
   - Retrieve the latest release tag reachable from HEAD via `git describe --tags --abbrev=0`.
   - If a previous tag exists, collect all commits from that tag to HEAD (`git log <tag>..HEAD`).
   - If no previous tag exists, inspect manifest files (`Cargo.toml`, `pyproject.toml`, `project.toml`) for a base version, or fallback to `v0.1.0`. Collect all commits up to HEAD.
   - Parse commits according to Conventional Commits:
     - `feat!:` or `BREAKING CHANGE:` / `BREAKING-CHANGE:` -> **MAJOR** bump.
     - `feat:` / `feat(...):` -> **MINOR** bump.
     - `fix:`, `perf:`, `refactor:`, `chore:`, etc. -> **PATCH** bump.
     - If no conventional commits or only chores/fixes are present, default to **PATCH**.
   - Bump the current version accordingly, maintaining the `v` prefix convention of the repository.

3. **CHANGELOG.md Generation / Update:**
   - Group commits by categories:
     - `### Breaking Changes`
     - `### Features`
     - `### Bug Fixes`
     - `### Performance Improvements`
   - Prepend the release notes under `## [<version>] - YYYY-MM-DD` in `CHANGELOG.md` (creating it if it does not exist, preserving existing header/entries).

4. **Manifest Synchronization & Atomic Commit:**
   - Update `Cargo.toml`, `pyproject.toml`, `project.toml` with the new version.
   - Synchronize lockfiles (`Cargo.lock` via `cargo check`, `uv.lock` via `uv lock` if present).
   - Stage modified files (`CHANGELOG.md`, manifests, lockfiles) and commit as `chore: release <version>`.
   - Create annotated release tag.
   - Output clear feedback describing inferred bump type and updated artifacts.

---

## 3. Impact Analysis

- **PRD Impact:** Amends FR-9 (SemVer release tagging) to support automated Conventional Commit bumping and changelog generation.
- **Architecture Impact:**
  - `src/cli/args.rs`: `Commands::Tag { version: Option<String> }`.
  - `src/core/semver.rs`: Add SemVer parsing, bumping (`BumpType`), and string formatting.
  - `src/core/conventional.rs`: Conventional commit parser and bump inference.
  - `src/infra/git.rs`: Add `git_get_latest_tag` and `git_get_commits_since`.
  - `src/workflow/changelog.rs`: Changelog generation and prepending.
  - `src/workflow/tag.rs`: Integrate automated bump workflow when version is None.
  - `tests/test_tag_workflow.rs`: Integration tests for automatic bump and changelog creation.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Clean implementation, comprehensive integration tests, Sphinx documentation with examples, all tests passing, and updated binary installed.
