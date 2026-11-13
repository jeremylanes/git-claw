---
title: 'Interactive prompts and overwrite guard in git claw init'
type: 'feature'
ticket: 2
created: '2026-11-13'
status: 'done'
route: 'full'
risk: 'low'
review: 'quick'
---

## Intent

**Problem:** Currently `run_init_workflow` ignores `options.yes`, runs non-interactively every time, and silently overwrites any existing `.git-claw.toml`. Documentation promises an interactive setup by default unless `--yes` / `-y` is passed.

**Approach:**
1. Refactor `run_init_workflow` to delegate to an internal testable helper `run_init_workflow_with_io<R: BufRead, W: Write>(options: InitOptions<'_>, reader: &mut R, writer: &mut W) -> Result<InitSummary, WorkflowError>` (where `run_init_workflow` calls it with `stdin().lock()` and `stdout()`).
2. When `!options.yes`:
   - If `.git-claw.toml` already exists in repo root, prompt `[y/N]` before overwriting; if declined, abort cleanly with an explicit `WorkflowError` (or clear cancellation error) without modifying `.git-claw.toml`.
   - Prompt for `main_branch` showing the detected default (e.g. current HEAD branch if `main` or `master` or configured fallback `"main"`, accepting Enter to keep default).
   - Display detected ports, env files, environment (`pyenv`/`uv`/etc.), and Docker compose settings, and ask confirmation `[Y/n]` before writing `.git-claw.toml` (with clear cancellation on `n`).
3. When `options.yes` is `true`:
   - Preserve zero-prompt automatic writing of `.git-claw.toml` (including overwriting when `--yes` is given), keeping all existing tests green.
4. Also make `git claw init` auto-detect `master` vs `main` gracefully if `main` branch isn't defaulted, or detect default branch accurately while keeping `"main"` fallback when repo has no commits yet (check existing `test_init_workflow.rs` assertions so none break!).
5. Strictly follow TDD (Rule 37): write failing unit tests with simulated `Cursor<&[u8]>` inputs first, run `cargo test` to observe failure, then implement.

## Acceptance Criteria
1. Without `--yes`, existing `.git-claw.toml` is preserved if the user answers `n` (or empty Enter default `[y/N]`) to overwrite prompt.
2. Without `--yes`, answering custom `main_branch` (e.g. `develop`) writes `main_branch = "develop"` into `.git-claw.toml`.
3. Answering `n` at final confirmation aborts without writing `.git-claw.toml`.
4. With `--yes` (`-y`), `.git-claw.toml` is written non-interactively with zero stdin reads.
5. `InitSummary` integrates cleanly with `--json` when `git claw init --yes --json` is invoked (note: `--json` without `--yes` fails explicitly or requires `--yes` if `.git-claw.toml` exists / interactive stdin would pollute JSON? Actually `--json` should imply non-interactive fail-safe or document `--yes --json`; let's handle `!options.yes && options.json` cleanly by erroring with a clear message telling automation to pass `--yes` if interactive input would be needed, OR fail-safe guard when `.git-claw.toml` exists!).
6. All unit, integration, `clippy`, and `fmt` checks pass.
