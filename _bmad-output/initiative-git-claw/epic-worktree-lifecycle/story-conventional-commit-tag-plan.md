---
title: 'Automated Conventional Commit Tagging and Changelog Generation (git claw tag)'
type: 'feature'
ticket: 14
created: '2026-10-17'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-conventional-commit-tag/change-conventional-commit-tag.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Running `git claw tag` requires passing a manual version (e.g. `git claw tag v0.2.1`). Users working with Conventional Commits (common in Python, Django, and Rust projects using tools like Commitizen) want `git claw tag` to automatically infer the version bump (major, minor, patch), generate `CHANGELOG.md`, update manifests, commit, and tag in a single step.

**Approach:** Make the version argument optional in CLI (`git claw tag [version]`). When omitted, query the latest Git tag, parse commits since that tag using Conventional Commits rules, determine the bump type, update `CHANGELOG.md`, update project manifests (`Cargo.toml`, `pyproject.toml`, `project.toml`), update lockfiles (`Cargo.lock`, `uv.lock`), commit `chore: release <version>`, and create the annotated release tag.

## Boundaries & Constraints

**Always:**
- Keep full backward compatibility when an explicit version is supplied (`git claw tag v0.3.0`).
- Parse Conventional Commits:
  - Breaking changes (`!` or `BREAKING CHANGE:` / `BREAKING-CHANGE:`) -> `BumpType::Major`.
  - `feat` -> `BumpType::Minor`.
  - `fix`, `perf`, `refactor`, `chore`, `docs`, etc. -> `BumpType::Patch`.
  - Default to `BumpType::Patch` if no higher bump is triggered.
- If no previous tag exists, infer starting version from manifest files or default to `v0.1.0`.
- Maintain `v` prefix style if previous tag had it (e.g. `v0.2.3` -> `v0.2.4`, `0.2.3` -> `0.2.4`).
- Prepend new version notes to `CHANGELOG.md` under `## [<version>] - YYYY-MM-DD`.
- Document every function/struct in Sphinx/Rustdoc format with at least one usage example (Rule 22).
- Write minimal, clean, robust code (Rules 20, 30, 31).

**Never:**
- Do not run git commands directly in the shell terminal (Rule 26).
- Do not fail if `uv` is absent or `CHANGELOG.md` did not previously exist.
- Do not proceed if the trunk working tree has uncommitted edits (`DirtyWorkingTree`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Auto tag with feature commit | Last tag `v0.2.0`, commits contain `feat: new thing` | Bumps minor -> `v0.3.0`, updates changelog, commits, tags `v0.3.0` | Ok(()) |
| Auto tag with breaking commit | Last tag `v0.2.0`, commits contain `fix!: break api` | Bumps major -> `v1.0.0`, updates changelog, commits, tags `v1.0.0` | Ok(()) |
| Auto tag with fix only | Last tag `v0.2.0`, commits contain `fix: small bug` | Bumps patch -> `v0.2.1`, updates changelog, commits, tags `v0.2.1` | Ok(()) |
| Auto tag in fresh repo (no tags) | No tags in repo | Infers base version from manifests or 0.1.0, bumps/tags appropriately | Ok(()) |
| Explicit version given | Run `git claw tag v0.5.0` | Uses explicit version v0.5.0, updates manifests, commits, tags | Ok(()) |
| Dirty working directory | Uncommitted edits present | Aborts before touching files | Err(DirtyWorkingTree) |

</frozen-after-approval>

## Code Map

- `src/cli/args.rs`:
  - `Commands::Tag { version: Option<String> }`
- `src/core/semver.rs`:
  - `SemVer` struct, `parse_semver`, `BumpType`, `bump(&self, bump: BumpType) -> SemVer`
- `src/core/conventional.rs`:
  - `parse_conventional_commits`, `determine_bump`
- `src/infra/git.rs`:
  - `git_get_latest_tag(repo_root: &Path) -> Result<Option<String>, GitError>`
  - `git_get_commits_since(repo_root: &Path, tag: Option<&str>) -> Result<Vec<String>, GitError>`
- `src/workflow/changelog.rs`:
  - `generate_changelog_entry`, `update_changelog_file`
- `src/workflow/tag.rs`:
  - Support `version: Option<&str>` in `TagOptions`, execute auto-bump workflow when `None`.
- `src/main.rs`:
  - Wire `Commands::Tag { version }`.
- `tests/test_tag_workflow.rs`:
  - Integration tests for auto Conventional Commit bump and changelog generation.
- `README.md`:
  - Document `git claw tag` (auto Commitizen-style) and `git claw tag <version>` (explicit).

## Tasks & Acceptance

**Execution:**
- [x] `src/core/semver.rs` -- Implement SemVer parsing and bumping.
- [x] `src/core/conventional.rs` -- Implement conventional commit parser and bump inference.
- [x] `src/infra/git.rs` -- Implement `git_get_latest_tag` and `git_get_commits_since`.
- [x] `src/workflow/changelog.rs` -- Implement changelog generation.
- [x] `src/cli/args.rs` & `src/main.rs` & `src/workflow/tag.rs` -- Integrate automated tag workflow.
- [x] `tests/test_tag_workflow.rs` -- Add tests verifying auto bump and changelog creation.
- [x] Update `README.md`.
- [x] Run `cargo check` and `cargo test`.
- [x] Run `cargo install --path .`.

**Acceptance Criteria:**
- Running `git claw tag` without arguments analyzes conventional commits, bumps version, generates `CHANGELOG.md`, updates manifests, commits, and tags.
- Running `git claw tag v0.5.0` still works with explicit version.
- All tests pass with `cargo test`.
- Binary installed in `~/.cargo/bin/git-claw`.
