---
title: 'Shell auto-completion engine (git claw completion <shell>)'
type: 'feature'
ticket: 10
created: '2026-10-10'
status: 'ready-to-build'
route: 'full'
risk: 'low'
review: 'quick'
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
  - '_bmad-output/initiative-git-claw/change-shell-completion/change-shell-completion.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Users must recall subcommands and options manually (`feature start`, `finish --keep-remote`, `publish`, etc.). Auto-completion in modern shells (especially Fish, Bash, Zsh) speeds up navigation and eliminates errors.

**Approach:** Leverage `clap_complete` to dynamically generate shell-specific auto-completion scripts. Expose this through `git claw completion <shell>` (with alias `completions`), streaming completion definitions directly to standard output.

## Boundaries & Constraints

**Always:**
- Support standard shells: `fish`, `bash`, `zsh`, `powershell`, `elvish`.
- Stream the completion script cleanly to standard output without stderr pollution.
- Document every struct/function with Sphinx/Rustdoc format and usage examples (Rule 22).
- Write as few lines of clean code as possible (Rules 30, 31).

**Never:**
- Do not execute git commands via terminal/shell (Rule 26).
- Do not add unnecessary external dependencies beyond `clap_complete`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Fish completion | `git claw completion fish` | Generates fish completion containing `complete -c git-claw` to stdout | Ok(()) |
| Bash completion | `git claw completion bash` | Generates bash completion script containing `_git-claw()` to stdout | Ok(()) |
| Zsh completion | `git claw completion zsh` | Generates zsh completion script containing `#compdef git-claw` to stdout | Ok(()) |
| Invalid shell | `git claw completion unknown` | Clap validation error listing supported shells | Exit code 2 |

</frozen-after-approval>

## Code Map

- `Cargo.toml`: Add `clap_complete = "4.5"`.
- `src/cli/args.rs`: Add `Commands::Completion { shell: clap_complete::Shell }`.
- `src/workflow/completion.rs`: Implement `run_completion_workflow(shell: clap_complete::Shell, mut writer: W)`.
- `src/workflow/mod.rs`: Export `completion` module and `run_completion_workflow`.
- `src/main.rs`: Wire `Commands::Completion`.
- `tests/test_completion_workflow.rs`: Integration tests for Fish, Bash, and Zsh completion output.
- `README.md`: Document usage and configuration for Fish, Bash, and Zsh.

## Tasks & Acceptance

**Execution:**
- [ ] `Cargo.toml` -- Add `clap_complete = "4.5"`.
- [ ] `src/cli/args.rs` -- Add `Completion` variant with `clap_complete::Shell` to `Commands`.
- [ ] `src/workflow/completion.rs` -- Implement `run_completion_workflow`.
- [ ] `src/workflow/mod.rs` & `src/main.rs` -- Wire completion workflow.
- [ ] `tests/test_completion_workflow.rs` -- Add tests verifying shell completion output for fish, bash, zsh.
- [ ] Update `README.md` with installation guides for fish, bash, zsh.

**Acceptance Criteria:**
- `git claw completion fish` outputs valid Fish completion commands (e.g. `complete -c git-claw ...`).
- `git claw completion bash` and `git claw completion zsh` output valid completion scripts.
- All tests pass with `cargo test`.
- `cargo install --path .` succeeds.
