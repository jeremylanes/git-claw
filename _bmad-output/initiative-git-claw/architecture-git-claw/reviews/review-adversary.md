# Architecture Review: Adversarial Decomposition & Incompatible Subunit Analysis

**Target Document:** `_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md`  
**Review Lens:** Adversary (Construct two units one level down that each obey every AD to the letter yet still build incompatibly)  
**Date:** 2026-10-05  
**Verdict:** **FAIL (Resolved in Finalize)**

---

## Findings

1. **Premature Slot Reallocation & Auto-GC Race**: Holding the lock only during JSON write allows concurrent commands to purge a slot while `git worktree add` is still creating the directory.
   - *Fix:* Extend lock holding across the `git worktree add` execution, and add a creation grace period.
2. **Untracked Artifacts Teardown Deadlock**: `.env.worktree` and cache symlinks cause `git worktree remove` to fail unless `--force` is used.
   - *Fix:* Mandate `git worktree remove --force` in worktree teardown.
3. **Inverted Branch Teardown Sequence**: Git forbids deleting a branch checked out in an active worktree.
   - *Fix:* Reorder sequence: remove worktree first, then delete local branch.
4. **Subdirectory Context Blindness**: In a linked worktree, `.git` is a file pointing to gitdir, causing `ENOTDIR` on relative `.git/claw` paths.
   - *Fix:* Add AD-9 mandating common git directory resolution via `git rev-parse --git-common-dir`.
5. **Trunk Merge Mutex & Dirty Workspace Clashing**: Merging into `main` requires verifying that the primary checkout is clean.
   - *Fix:* Mandate primary working tree clean check before trunk merge.
6. **Registry Schema Disambiguation**: `slots.json` lacked explicit `name` and `branch_type`.
   - *Fix:* Enrich schema with `name` and `branch_type` and enforce uniqueness on `name`.
