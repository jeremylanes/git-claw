# Story Plan: Opt-in Navigation on Worktree Start (`--cd`)

## Ticket 17
- **Epic**: `epic-worktree-lifecycle`
- **Component**: CLI argument parsing & Shell integration (`src/cli/args.rs`, `src/workflow/shell.rs`, `src/main.rs`)

## Acceptance Criteria
1. `git-claw feature start <name>`, `bugfix start <name>`, `hotfix start <name>`, and `spike start <name>` accept `--cd` flag without error.
2. In Bash, Zsh, and Fish shell hooks:
   - Running `git claw feature start <name>` (without `--cd`) keeps the active shell in the primary repository.
   - Running `git claw feature start <name> --cd` navigates the active shell into the worktree directory.
3. `git claw finish` and `git claw spike drop` continue to automatically navigate back to the primary repository root.
4. Shell hook installed in `~/.config/fish/conf.d/git_claw.fish` is updated.
5. All automated unit and integration tests pass cleanly (`cargo test`).
