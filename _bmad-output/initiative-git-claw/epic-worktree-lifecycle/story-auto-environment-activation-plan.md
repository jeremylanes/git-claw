# Story Plan: Automatic Environment Detection and Propagation (`pyenv`, `uv`, `venv`)

## Ticket 18
- **Epic**: `epic-worktree-lifecycle`
- **Component**: Environment detection, configuration, and worktree initialization (`src/core/config.rs`, `src/workflow/init.rs`, `src/workflow/start.rs`, `src/infra/env_file.rs`)

## Roles
- **John (Product Manager)**: Requirements specification & Story plan.
- **Amelia (Senior Systems Developer)**: Core implementation, data models, scanner, file propagation, and unit tests.
- **Murat (QA & Test Architect)**: Independent verification, cross-platform matrix testing, edge-case validation, and sign-off.

## Implementation Details
1. **`src/core/config.rs`**:
   - Define `EnvironmentConfig`:
     ```rust
     #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
     pub struct EnvironmentConfig {
         #[serde(default)]
         pub r#type: Option<String>,
         #[serde(default)]
         pub name: Option<String>,
         #[serde(default)]
         pub copy_files: Vec<String>,
     }
     ```
   - Add `pub environment: EnvironmentConfig` to `Config`.
   - Update `Config::default_for_repo` and `Config::merge_with`.
2. **`src/workflow/init.rs`**:
   - Implement `scan_environment(root: &Path) -> (EnvironmentConfig, Option<String>)`:
     - Checks `.python-version`, `.venv`, `uv.lock`, `.nvmrc`, `.node-version`.
     - Returns `(EnvironmentConfig, Option<cache_dir_to_share>)`.
   - Update `generate_init_toml` to include `[environment]` and `cache.directories = [".venv"]` when detected.
3. **`src/workflow/start.rs` & `src/infra/env_file.rs`**:
   - In `start_worktree`:
     - Propagate declared and standard environment files (`.python-version`, `.node-version`, `.nvmrc`, `.envrc`) from `toplevel` to `worktree_path`.
     - If `config.environment.name` is set and `.python-version` does not exist in worktree, write `name` to `.python-version`.
   - In `write_env_worktree`:
     - If `config.environment.name` is Some, write `PYENV_VERSION=<name>`.
     - If `.venv` exists in `worktree_path`, write `VIRTUAL_ENV` and update `PATH`.
4. **Testing (`tests/test_init_workflow.rs`, `tests/test_start_workflow.rs`, `tests/test_env_file.rs`)**:
   - Verify `git-claw init` detects `.python-version` and emits `[environment]`.
   - Verify `start_worktree` propagates `.python-version` into the worktree.
   - Verify `.env.worktree` contains `PYENV_VERSION` and `VIRTUAL_ENV` when appropriate.
