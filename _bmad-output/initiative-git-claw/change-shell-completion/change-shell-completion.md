# Sprint Change Proposal: Shell Auto-Completion Engine

**Date:** 2026-10-10  
**Status:** Approved  
**Author:** John (Product Manager 📋)  
**Trigger:** User request for shell auto-completion: *"cool, ce qui me manque maintenent c'est l'auto-completion"*. Active shell environment is Fish (`~/.config/fish/completions/`), with broader team support needed for Bash, Zsh, PowerShell, and Elvish.

---

## 1. Issue Summary

`git-claw` offers a rich set of subcommands (`feature`, `bugfix`, `hotfix`, `spike`, `finish`, `publish`, `list`, `run`, `open`, `cd`, `tag`, `init`, `shell-hook`) with various options and sub-actions. However, users must currently remember all arguments manually.

Shell auto-completion drastically improves developer experience (DX) and reduces typos. Modern CLI tools like `cargo`, `docker`, `gh`, and `rustup` provide a built-in `completion` command to stream completion definitions directly into shell configuration paths.

---

## 2. Requirement Specification

1. **Subcommand Definition:**
   - Command: `git claw completion <shell>` (with alias `git claw completions <shell>`).
   - Supported shells (via `clap_complete::Shell`):
     - `fish`
     - `bash`
     - `zsh`
     - `powershell`
     - `elvish`
   - Output: Streams the raw completion script to standard output (`stdout`).

2. **Ecosystem & Shell Integration:**
   - For **Fish**:
     - Redirection target: `~/.config/fish/completions/git-claw.fish`.
     - Fish's built-in git completion engine automatically routes `git claw ...` to `git-claw`, enabling both `git claw <TAB>` and `git-claw <TAB>`.
   - For **Bash**:
     - `git claw completion bash > ~/.local/share/bash-completion/completions/git-claw` (or `eval "$(git claw completion bash)"`).
   - For **Zsh**:
     - `git claw completion zsh > ~/.zfunc/_git-claw`.

3. **Performance & Footprint:**
   - Use official `clap_complete = "4.5"`.
   - Zero runtime overhead for non-completion commands.
   - Clean, idiomatic Rust with docstrings and usage examples (Rule 22).

---

## 3. Impact Analysis

- **PRD Impact:** Adds FR-14 (Shell completion integration).
- **Architecture Impact:**
  - `Cargo.toml`: Add `clap_complete = "4.5"`.
  - `src/cli/args.rs`: Add `Completion { shell: clap_complete::Shell }` to `Commands`.
  - `src/workflow/completion.rs`: Implement `run_completion_workflow(shell: clap_complete::Shell, writer: &mut W)`.
  - `src/workflow/mod.rs`: Export `run_completion_workflow`.
  - `src/main.rs`: Wire `Commands::Completion`.
  - `tests/test_completion_workflow.rs`: Integration tests for Fish, Bash, and Zsh completion output.
  - `README.md`: Document activation commands for Fish, Bash, and Zsh.

---

## 4. Implementation Handoff

- **Recipient:** Amelia (Senior Systems Developer 💻).
- **Target Deliverable:** Complete implementation, integration tests, `cargo test` passing, and binary installation.
