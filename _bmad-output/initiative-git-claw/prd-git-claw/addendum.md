# Addendum & Downstream Architecture Notes: `git-claw`

## 1. Technical Rationale & Evaluated Alternatives

### 1.1 Git CLI Invocations vs `git2-rs` (libgit2 bindings)
- **Decision**: Rely on `std::process::Command::new("git")` for Git operations rather than linking `git2` / `libgit2`.
- **Rationale**:
  - Worktree features in Git evolve rapidly; `git worktree` support in `git2-rs` is notoriously complex, sometimes lagging behind upstream Git.
  - Using system Git keeps the binary lightweight, statically linkable (musl/glibc), fast to compile, and guarantees 100% behavioral parity with user's Git configurations (hooks, config, credentials).
  - Calling `git` via `Command` adds minimal overhead (< 5-10ms) well below the 500ms budget.

### 1.2 Slot Registry Persistence: `.git/claw/slots.json`
- **Decision**: Store active slots state in the primary repository's Git directory under `.git/claw/slots.json`.
- **Rationale**:
  - The `.git` directory is already ignored by version control, ensuring slot allocations on one machine are never committed or pushed to remotes.
  - Worktrees share access to the common Git directory (via `git rev-parse --git-common-dir`), ensuring all worktrees and the main repo view the exact same registry file.
  - JSON format allows straightforward atomic writes (write to temp file then rename) to prevent race conditions.

### 1.3 Cache Sharing: Symlinks vs Hardlinks vs OverlayFS
- **Decision**: POSIX symlinks.
- **Rationale**:
  - Hardlinks cannot span directories or cross filesystems.
  - OverlayFS requires root/sudo privileges which violates the requirement for an unprivileged CLI tool.
  - Symlinks are universally supported, standard, transparent to tooling (Cargo, Node, Python venv), and instant.

## 2. Downstream Handoff Recommendations

### To Software Architect (`bmad-architecture`)
- Recommended Rust Crates:
  - `clap` with `derive` feature for CLI arg parsing.
  - `serde` and `serde_json` for slots registry serialization.
  - `toml` for `.git-claw.toml` parsing.
  - `colored` or `anstream` for beautiful, terminal-aware colored output.
  - `tempfile` and `assert_cmd` for integration testing.

### To QA & Dev (`bmad-ticket` / `bmad-build`)
- Slicing recommendations:
  - Epic 1: Configuration & Slot Allocation Engine (Core logic).
  - Epic 2: Worktree Lifecycle (`start`, `finish`, `spike drop`).
  - Epic 3: Hooks Execution & Environment Injection (`.env.worktree`).
  - Epic 4: Dashboard, Inspection & Helpers (`list`, `run`, `open`, `cd`).
