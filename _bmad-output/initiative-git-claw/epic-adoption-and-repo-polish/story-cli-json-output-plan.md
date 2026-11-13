---
title: 'Structured --json CLI output for AI agents and scripts'
type: 'feature'
ticket: 1
created: '2026-11-13'
status: 'done'
route: 'full'
risk: 'medium'
review: 'quick'
---

## Intent

**Problem:** `git-claw` positions itself as a deterministic orchestration engine for developers and AI agents, yet lifecycle and inspection commands (`list`, `start`, `finish`, `publish`, `spike drop`, `init`) only output human-formatted ASCII tables or colored text. AI agents (Claude Code, Antigravity, Cursor, Codex) and shell automation scripts must either scrape formatted output or guess paths/ports.

**Approach:**
1. Add `--json` support (`#[arg(long)] pub json: bool`) to `List`, `WorktreeAction::Start`, `WorktreeAction::Publish`, `Finish`, `SpikeAction::Drop`, and `Init` (or a clean global/command-level `--json` flag that leaves raw stdout commands like `cd`, `completion`, and `shell-hook` unaffected — keeping `--json` clean and explicit in `--help`).
2. Introduce clean serializable structs in `src/cli/output.rs` / workflow modules (reusing existing `serde::Serialize` and `serde_json` already in `Cargo.toml`) without duplicating workflow logic:
   - `list --json`: JSON array (or object/array — let's keep a clean top-level JSON object `{ "worktrees": [...] }` or consistent `{ ... }` envelope across commands, e.g., `list --json` outputting `{ "worktrees": [ { "slot_id": 1, "name": "...", "branch": "...", "path": "...", "ports": {...}, "status": "..." } ] }` or document clearly! Actually returning structured objects on every JSON command:
     - `list --json`: `{ "worktrees": [...] }`
     - `start --json`: `{ "slot_id": u32, "name": String, "branch": String, "path": String, "ports": BTreeMap<String, u16>, "test_database": Option<String> }` (suppress informational stdout banners when `json == true` so stdout is 100% pure parseable JSON!)
     - `finish --json`: `{ "name": String, "branch": String, "target_branch": String, "slot_id": Option<u32>, "primary_root": String, "remote_deleted": bool }`
     - `publish --json`: `{ "name": String, "branch": String, "remote": String }`
     - `spike drop --json`: `{ "name": String, "branch": String, "slot_id": Option<u32>, "primary_root": String }`
     - `init --json`: `{ "config_path": String, "main_branch": String, "ports": BTreeMap<String, u16>, "copy_files": Vec<String>, "environment_type": Option<String>, "compose_file": Option<String> }`
3. Ensure human-readable `print_success` / `print_warning` banners on `stdout` do not corrupt JSON output when `json == true` (warnings still go to `stderr` via `eprintln!`, which `print_warning`/`print_error` already use; route success lines cleanly only when `!json` or pass an output mode).
4. Strictly follow TDD (Rule 37): write failing tests first for CLI parsing and end-to-end `--json` payloads before implementing production code.

## Acceptance Criteria
1. Running `git-claw list --json`, `git-claw feature start <name> --json`, `git-claw feature publish <name> --json`, `git-claw finish <name> --json`, `git-claw spike drop <name> --json`, and `git-claw init --yes --json` outputs valid single-document JSON on stdout parseable by `serde_json::from_str`.
2. Non-`--json` output remains 100% backward-compatible with all existing tests.
3. No external crate dependencies added (`serde` + `serde_json` already present).
4. All public types/functions have Sphinx/Rustdoc comments with at least one usage example (Rule 22/23 style adapted cleanly to Rustdoc `/// # Examples`).
5. All unit, integration, `clippy -D warnings`, and `cargo fmt --check` checks pass.
