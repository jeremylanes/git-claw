---
title: 'Python / uv project version synchronization on release tag (git claw tag <version>)'
type: 'feature'
ticket: 13
created: '2026-10-17'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-python-project-version-sync/change-python-project-version-sync.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** In Python/Django repositories (particularly those using `uv` with `pyproject.toml` or `project.toml`), running `git claw tag <version>` currently only synchronizes `Cargo.toml`. Python developers must manually update `pyproject.toml`, refresh `uv.lock`, and commit before tagging.

**Approach:** Extend `workflow::tag` to detect `pyproject.toml` and `project.toml`. If present, update `version = "..."` under `[project]` (PEP 621) or `[tool.poetry]`. If `uv.lock` is present, refresh it using `uv lock`. Stage and commit modified files as `chore: release <version>`, and create the annotated release tag.

## Boundaries & Constraints

**Always:**
- Strictly preserve comments, indentation, formatting, and other sections.
- Support `[project]` (used by uv, hatch, flit, setuptools) and `[tool.poetry]`.
- If the file version already matches the target SemVer version, skip updating it.
- If uncommitted changes exist prior to tagging, abort with `DirtyWorkingTree`.
- Document every function/struct in Sphinx/Rustdoc format with at least one usage example (Rule 22).
- Write minimal, clean, robust code (Rules 20, 30, 31).

**Never:**
- Do not run git commands directly in the shell terminal (Rule 26).
- Do not fail or panic if `uv` is not installed on the system; ignore lockfile generation errors gracefully.
- Do not affect projects without `pyproject.toml` or `project.toml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| pyproject.toml exists with `[project]` | `pyproject.toml` v0.1.0, run `git claw tag v0.2.0` | Updates `pyproject.toml` to 0.2.0, commits, creates tag v0.2.0 | Ok(()) |
| pyproject.toml with `uv.lock` | `uv.lock` present | Runs `uv lock`, stages both `pyproject.toml` and `uv.lock`, commits, tags | Ok(()) |
| project.toml exists | `project.toml` v1.0.0, run `git claw tag 1.1.0` | Updates `project.toml` to 1.1.0, commits, tags | Ok(()) |
| pyproject.toml version already matches | `pyproject.toml` v0.2.0, run `git claw tag v0.2.0` | Skips file edit, creates tag v0.2.0 | Ok(()) |
| Neither Cargo.toml nor Python toml exists | Generic git repo | Creates tag directly without commit | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/workflow/tag.rs`:
  - `update_toml_version(content: &str, sections: &[&str], new_version: &str) -> Option<String>`
  - `update_cargo_toml_version(content: &str, new_version: &str) -> Option<String>`
  - `update_pyproject_toml_version(content: &str, new_version: &str) -> Option<String>`
  - Integrate Python manifest detection and `uv.lock` synchronization into `tag_release`.
- `tests/test_tag_workflow.rs`:
  - Integration tests for `pyproject.toml` and `project.toml` version updates and tagging.
- `README.md`:
  - Mention Python / `uv` / `pyproject.toml` version sync in tag documentation.

## Tasks & Acceptance

**Execution:**
- [ ] `src/workflow/tag.rs` -- Implement `update_pyproject_toml_version` and integrate into `tag_release`.
- [ ] `tests/test_tag_workflow.rs` -- Add integration tests for `pyproject.toml` and `project.toml`.
- [ ] Update `README.md`.
- [ ] Run `cargo check` and `cargo test`.
- [ ] Run `cargo install --path .`.

**Acceptance Criteria:**
- Running `git claw tag v0.2.0` in a Python repo with `pyproject.toml` or `project.toml` updates the version, commits `chore: release v0.2.0`, and creates the release tag.
- All 90+ tests pass with `cargo test`.
- Binary installed in `~/.cargo/bin/git-claw`.
