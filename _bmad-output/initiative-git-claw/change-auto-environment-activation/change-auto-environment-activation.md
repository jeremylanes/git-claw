# Proposal: Automatic Environment Detection and Worktree Activation (`pyenv`, `uv`, `venv`)

## Problem Statement
When developers create a worktree (e.g. `git claw feature start <name>`), the new worktree directory lacks the project's runtime environment configuration:
1. **Pyenv / pyenv-virtualenv**: Pyenv detects virtual environments via `.python-version`. In repositories like `tune`, `.python-version` defines the target virtualenv (`tune`). Because Git worktree only creates a checkout of tracked files, untracked `.python-version` files (or isolated worktrees) lack `.python-version`, forcing developers to manually run `pyenv activate <name>` repeatedly.
2. **uv / virtualenv (`.venv`)**: Modern Python workflows rely on `.venv`. Without automated symlinking and `VIRTUAL_ENV` environment variable injection, tools and subshells fail to detect the active virtual environment.
3. **Node.js (`.nvmrc`, `.node-version`)**: Node version declarations are often not propagated into worktrees.

## Proposed Solution
1. **Repository Scanning during `git claw init`**:
   - Detect Python environment:
     - Check `.python-version` -> infer `pyenv` and virtualenv/python version name.
     - Check `uv.lock` or `pyproject.toml` -> infer `uv`.
     - Check `.venv` -> infer `venv` and automatically add `.venv` to `cache.directories` (symlinked shared cache).
   - Detect Node environment:
     - Check `.nvmrc` or `.node-version`.
   - Populate `[environment]` in generated `.git-claw.toml`:
     ```toml
     [environment]
     type = "pyenv"        # "pyenv" | "venv" | "uv" | "none"
     name = "tune"         # optional virtualenv or version name
     copy_files = [".python-version"]
     ```
2. **Automatic Environment Propagation on Worktree Start (`src/workflow/start.rs`)**:
   - Automatically copy or generate `.python-version` in the worktree root.
   - Automatically propagate `.nvmrc`, `.node-version`, and `.envrc` if present in the primary repository.
   - Automatically symlink `.venv` if `cache.directories` contains `.venv` or if `type = "venv"` / `"uv"`.
   - In `.env.worktree`:
     - Inject `PYENV_VERSION=<name>` if configured.
     - Inject `VIRTUAL_ENV=<worktree_path>/.venv` and prepend `<worktree_path>/.venv/bin` to `PATH` if `.venv` exists.
3. **Zero-Overhead Native Shell Activation**:
   - With `.python-version` present in the worktree root, Pyenv and Starship natively and automatically activate the correct environment the moment any shell, editor, or multiplexer enters the worktree directory.
