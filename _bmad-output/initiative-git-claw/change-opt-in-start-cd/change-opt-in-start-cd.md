# Proposal: Opt-in Navigation on Worktree Start (`--cd`)

## Problem Statement
Previously, `git-claw shell-hook` (for Fish, Bash, and Zsh) automatically navigated (`cd`) into any newly created worktree upon running `git claw [type] start <name>`.
While convenient in some environments, in modern developer workflows (especially when using terminal multiplexers such as Herdr), the worktree is already opened automatically in a new multiplexer window or background tab.
Navigating the primary terminal window into the worktree disrupts the developer's context in the primary repository.
By default, the active terminal session should remain in the primary repository (`main`/`master`), and only navigate into the new worktree when explicitly requested via the `--cd` flag.

Conversely, `git claw finish` and `git claw spike drop` must continue to navigate automatically back to the primary repository, because the worktree directory is deleted.

## Proposed Changes
1. **CLI Argument Parser (`src/cli/args.rs`)**:
   - Add `#[arg(long)] cd: bool` to `WorktreeAction::Start` and `SpikeAction::Start`.
   - Document `--cd` flag: "Automatically navigate (cd) to the worktree directory after creation (default: false)".
2. **Shell Integration Hooks (`src/workflow/shell.rs`)**:
   - In `generate_posix_hook` and `generate_fish_hook`, parse arguments to detect the presence of `--cd`.
   - If `--cd` is absent, do not execute `cd "$dest"`. Remain in current working directory.
   - If `--cd` is present, resolve `dest="$(command git-claw cd "$target_name")"` and navigate into it.
3. **Shell Hook Configuration Update**:
   - Update `~/.config/fish/conf.d/git_claw.fish` with the updated hook logic.
4. **Integration Tests**:
   - Update `tests/test_shell_hook.rs`:
     - Test that `claw feature start <name>` without `--cd` leaves the terminal in the primary repository.
     - Test that `claw feature start <name> --cd` navigates the terminal into the new worktree.
     - Test that `git claw finish <name>` continues to navigate back to primary repo root automatically.
