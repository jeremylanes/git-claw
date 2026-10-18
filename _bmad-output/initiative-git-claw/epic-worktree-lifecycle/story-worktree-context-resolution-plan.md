---
title: 'Primary Repository Root Resolution from Worktree Context'
type: 'bugfix'
ticket: 15
created: '2026-10-18'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-worktree-context-resolution/change-worktree-context-resolution.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Running `git claw finish <name>` from inside a linked worktree causes `git-claw` to attempt checking out `main` inside the worktree directory itself, which fails with `fatal: 'main' is already used by worktree at '<primary-repo>'`.

**Approach:** Implement `resolve_primary_repo_root` using `git rev-parse --git-common-dir`. Use `resolve_primary_repo_root` across workflow commands (`finish`, `start`, `list`, `spike`, `tag`, etc.) so operations always target the primary repository even when executed from inside a linked worktree.

## Boundaries & Constraints

**Always:**
- Keep all existing functions backward compatible.
- Document functions in Sphinx/Rustdoc format with usage examples (Rule 22).
- Write minimal, clean, robust code (Rules 20, 30, 31).

**Never:**
- Do not run git commands directly in the shell terminal (Rule 26).
- Do not break standard single-worktree repositories.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Finish from worktree | CWD inside worktree, run `git claw finish` | Resolves primary repo root, checks out main on primary repo, merges, deletes worktree | Ok(()) |
| Finish from primary repo | CWD in primary repo, run `git claw finish <name>` | Works identically as before | Ok(()) |
| List from worktree | CWD inside worktree, run `git claw list` | Loads config from primary repo, displays slots | Ok(()) |
| Tag from worktree | CWD inside worktree, run `git claw tag` | Tags main on primary repo | Ok(()) |

</frozen-after-approval>

## Code Map

- `src/infra/git.rs`: Implement `resolve_primary_repo_root`.
- `src/infra/mod.rs`: Export `resolve_primary_repo_root`.
- `src/workflow/finish.rs`: Use `resolve_primary_repo_root`.
- `src/workflow/start.rs`, `src/workflow/spike.rs`, `src/workflow/list.rs`, `src/workflow/run.rs`, `src/workflow/nav.rs`, `src/workflow/tag.rs`, `src/workflow/publish.rs`: Use `resolve_primary_repo_root`.
- `tests/test_finish_workflow.rs`: Integration test verifying finish executed with worktree context.

## Tasks & Acceptance

**Execution:**
- [ ] Implement `resolve_primary_repo_root` in `src/infra/git.rs`.
- [ ] Update `src/workflow/finish.rs` and other workflows.
- [ ] Add integration test in `tests/test_finish_workflow.rs`.
- [ ] Run `cargo check`, `cargo fmt --check`, `cargo clippy`, and `cargo test`.
- [ ] Run `cargo install --path .`.

**Acceptance Criteria:**
- Running `finish_worktree` with `repo_root` set to a linked worktree completes successfully without `fatal: 'main' is already used by worktree`.
- All tests pass with `cargo test`.
- Binary installed in `~/.cargo/bin/git-claw`.
