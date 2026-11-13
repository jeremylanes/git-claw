---
name: git-claw
description: Orchestrate isolated Git worktrees, deterministic slots and ports, Docker Compose overrides, and per-slot test databases with git-claw in --json mode. Use whenever developing features, bugfixes, hotfixes, or spikes in a repository managed by git-claw or running parallel agent tasks.
---

# git-claw Agent Playbook

Use `git-claw` to isolate every branch in its own physical worktree directory with deterministic slot IDs, collision-free ports, and isolated test databases.

## Core Rules

1. **Never use `git checkout` or `git switch`** in the primary repository for feature, bugfix, hotfix, or spike work.
2. **Always pass `--json`** on lifecycle and inspection commands (`list`, `start`, `publish`, `finish`, `spike drop`, `init --yes`) so stdout is machine-parseable.
3. **Never edit the primary repository `.env`** or manually assign ports; read the worktree's generated `.env.worktree` and JSON output.
4. **Always clean up** completed work with `git claw finish <name> --json` or discard throwaway spikes with `git claw spike drop <name> --json`.

## Standard Workflow

### 1. Inspect Active Worktrees

```bash
git claw list --json
```

Returns:
```json
{
  "worktrees": [
    {
      "slot_id": 1,
      "name": "auth-flow",
      "branch": "feature/auth-flow",
      "path": "/home/dev/.git-claw/worktrees/repo/auth-flow",
      "ports": { "PORT_WEB": 8001 },
      "status": "active"
    }
  ]
}
```

### 2. Initialize Configuration (If `.git-claw.toml` Is Missing)

Interactive prompts hang non-interactive agents, so always pass `--yes` with `--json`:

```bash
git claw init --yes --json
```

### 3. Create an Isolated Worktree

Choose the appropriate branch prefix (`feature`, `bugfix`, `hotfix`, or `spike`):

```bash
git claw feature start <kebab-case-name> --json
```

Returns:
```json
{
  "slot_id": 1,
  "name": "<kebab-case-name>",
  "branch": "feature/<kebab-case-name>",
  "path": "/path/to/worktree",
  "ports": { "PORT_WEB": 8001 },
  "test_database": "test_<repo>_slot_1"
}
```

- Perform all edits inside `path`.
- Use `ports` and `test_database` (`TEST_DATABASE_NAME` / `CLAW_TEST_DB` in `.env.worktree`) for servers and test suites.
- Pass `--isolated` if shared cache symlinks (`.venv`, `node_modules`, `target`) must be skipped.

### 4. Run Commands With Worktree Environment Injected

```bash
git claw run <kebab-case-name> <command...>
```

Automatically changes into the worktree directory and loads `.env.worktree` variables (`CLAW_SLOT_ID`, `CLAW_WORKTREE_NAME`, `CLAW_WORKTREE_PATH`, `COMPOSE_PROJECT_NAME`, `TEST_DATABASE_NAME`, `CLAW_TEST_DB`, effective ports, `PYENV_VERSION`, `VIRTUAL_ENV`).

### 5. Publish (Optional)

```bash
git claw feature publish <kebab-case-name> --json
```

### 6. Merge and Clean Up

For `feature`, `bugfix`, or `hotfix` (runs `pre_finish` hooks before merging into `main_branch`):

```bash
git claw finish <kebab-case-name> --json
```

Options:
- `--keep-remote` (`-k`): keep remote branch after merging.
- `--force`: proceed even if `pre_finish` hook fails (only when explicitly requested).

For disposable `spike` experiments (discards branch without merging):

```bash
git claw spike drop <kebab-case-name> --json
```
