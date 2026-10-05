---
title: 'Cargo scaffold, layered module structure & Clap CLI parser'
type: 'feature'
ticket: 1
created: '2026-10-05'
status: 'built'
followup_review_recommended: false
baseline_revision: '68eb24820f3950ffc28102d78be771c7168787e9'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The `git-claw` repository lacks the base Rust project setup, layered architecture skeleton (`core/`, `infra/`, `workflow/`, `cli/`), and Clap CLI parser required to accept command-line arguments and run as both `git-claw` and `git claw`.

**Approach:** Initialize `Cargo.toml` with required dependencies (Clap 4.5 with derive, thiserror, etc.), establish four layered modules with strict boundary invariants, and implement a robust Clap parser that handles `--help`, `--version`, all subcommands (`start`, `finish`, `spike`, `list`, `run`, `open`, `cd`, `tag`), and strips redundant `claw` subcommand prefixes when executed via Git.

## Boundaries & Constraints

**Always:**
- Keep `core/` completely pure: zero I/O, zero external side-effects, no dependencies on `infra/`, `workflow/`, or `cli/`.
- Support transparent execution as both `git-claw <args>` and `git claw <args>` (or `git-claw claw <args>`).
- Use standard UNIX exit codes: `0` for success, non-zero for errors.
- Ensure `cargo build` and `cargo test` compile cleanly without warnings or errors.

**Never:**
- Do not implement actual workflow logic (worktree creation, registry persistence, git execution) in this story — stub handlers cleanly.
- Do not use dynamic C git bindings (`libgit2`).
- Do not introduce AI metadata or ticket references into source code comments.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Help flag direct | `git-claw --help` | Outputs usage info, description, and list of subcommands with exit code 0 | Clap renders stdout, exits 0 |
| Version flag | `git-claw --version` | Outputs binary name and SemVer version (`git-claw 0.1.0`) with exit code 0 | Clap renders stdout, exits 0 |
| Git subcommand invocation | `git-claw claw --help` | Transparently handles redundant `claw` argument and prints main help | Strips leading `claw` token, exits 0 |
| Unknown subcommand | `git-claw invalid-cmd` | Prints error message showing unrecognized argument and usage hint | Exits with code 2 (Clap error) |
| Feature start help | `git-claw feature start --help` | Displays usage for creating feature worktree: `<name>` argument and options | Exits with code 0 |
| Subcommand execution stub | `git-claw list` | Invokes stub presentation layer, returning success exit code 0 | Clean exit 0 |

</frozen-after-approval>

## Code Map

- `Cargo.toml` -- Project manifest defining binary name `git-claw`, Rust edition 2021, and core dependencies (`clap`, `thiserror`, `colored`, dev-dependencies: `assert_cmd`, `predicates`).
- `src/main.rs` -- Main CLI entrypoint; parses arguments via `cli::Cli::parse_from_env()`, routes to workflow dispatch, maps typed errors to UNIX process exit codes.
- `src/cli/mod.rs` -- Presentation layer module root; re-exports CLI structs, args, and output utilities.
- `src/cli/args.rs` -- Clap parser hierarchy defining `Cli`, subcommands (`feature`, `bugfix`, `hotfix`, `spike`, `finish`, `list`, `run`, `open`, `cd`, `tag`), and argument stripping logic.
- `src/cli/output.rs` -- Presentation helpers for terminal styling and output conventions.
- `src/workflow/mod.rs` -- Application layer orchestrator module; defines workflow dispatch signatures and stubbed execution handlers.
- `src/core/mod.rs` -- Domain layer module root; pure logic placeholder ensuring clean boundary isolation.
- `src/infra/mod.rs` -- Infrastructure layer module root; external side-effects placeholder.
- `tests/test_cli.rs` -- Integration test suite validating CLI argument parsing, `--help`, `--version`, and redundant `claw` subcommand stripping.

## Tasks & Acceptance

**Execution:**
- [x] `Cargo.toml` -- Create Cargo project manifest -- Configures `git-claw` package, Rust edition 2021, and required dependencies.
- [x] `src/core/mod.rs` -- Create core module placeholder -- Establishes pure domain layer root with no I/O dependencies.
- [x] `src/infra/mod.rs` -- Create infra module placeholder -- Establishes infrastructure layer root for external adapters.
- [x] `src/workflow/mod.rs` -- Create workflow module root and stub runners -- Exposes application dispatch functions.
- [x] `src/cli/output.rs` -- Create CLI output helpers -- Provides ANSI formatting and standard error output utilities.
- [x] `src/cli/args.rs` -- Implement Clap CLI arguments and Git subcommand stripping -- Defines command structure supporting direct and `git claw` invocations.
- [x] `src/cli/mod.rs` -- Re-export CLI types -- Organizes presentation layer public API.
- [x] `src/main.rs` -- Implement CLI main entrypoint -- Initializes argument parsing, delegates to workflow dispatch, handles exit codes.
- [x] `tests/test_cli.rs` -- Implement integration tests for CLI -- Verifies `--help`, `--version`, `git claw` invocation transparency, and error exit codes.

**Acceptance Criteria:**
- Given `git-claw` binary compiled, when invoked with `--help`, then it exits with code 0 and displays application description and available subcommands.
- Given `git-claw` binary compiled, when invoked with `--version`, then it exits with code 0 and prints `git-claw` with version `0.1.0`.
- Given `git-claw` binary compiled, when invoked as `git-claw claw --help`, then it strips `claw` and renders the same top-level help text with exit code 0.
- Given `git-claw` binary compiled, when invoked with an invalid subcommand `foo`, then it outputs an error to stderr and terminates with exit code 2.
- Given the codebase, when running `cargo test`, then all tests pass with zero warnings.

## Implementation Notes
- Implemented base Cargo package configuration for `git-claw` 0.1.0 with clap 4.5, thiserror 2.0, colored 2.2, serde, toml, and test dependencies.
- Created layered architecture boundaries: `core/`, `infra/`, `workflow/`, and `cli/`.
- Implemented `sanitize_args` to ensure transparent handling of redundant `claw` token in `git claw <args>` and direct `git-claw <args>`.
- Verified all 6 matrix test scenarios in `tests/test_cli.rs`; all tests passing cleanly.

## Plan Change Log

## Review Triage Log

### 2026-10-06 — Review pass
- verdicts: 0 findings — high 0, medium 0, low 0, false 0, maybe-false 0
- findings: []

## Design Notes

Clap 4.5 supports custom argument parsing before invoking `parse()` or `try_parse()`. When Git invokes `git claw <args>`, the executable invoked is `git-claw` with arguments `["git-claw", "<args>"]`. If a user or script executes `git-claw claw <args>`, the argument vector contains `"claw"` as the first positional argument. A wrapper method `Cli::parse_from_env()` inspects `std::env::args_os()`; if the second argument is `"claw"`, it removes it from the iterator before delegating to `Cli::parse_from()`.

```rust
pub fn sanitize_args<I, T>(args: I) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    T: AsRef<std::ffi::OsStr> + Clone,
{
    let mut vec: Vec<T> = args.into_iter().collect();
    if vec.len() > 1 && vec[1].as_ref() == "claw" {
        vec.remove(1);
    }
    vec
}
```

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test` -- expected: all integration tests pass
- `cargo run -- --help` -- expected: prints CLI help text with exit code 0
- `cargo run -- --version` -- expected: prints git-claw 0.1.0 with exit code 0
- `cargo run -- claw --help` -- expected: prints CLI help text transparently with exit code 0

## Auto Run Result

### Summary of implemented change
Implemented the initial Rust project scaffold for `git-claw`, structured with clean layered boundaries (`core/`, `infra/`, `workflow/`, `cli/`), and built the Clap presentation parser supporting `--help`, `--version`, all planned subcommands, and transparent execution via `git claw`.

### Files changed
- `Cargo.toml` - Project manifest with package metadata, edition 2021, and dependencies (`clap`, `thiserror`, `colored`, `serde`, `toml`, `fd-lock`, test fixtures).
- `src/main.rs` - Application entrypoint routing CLI arguments and formatting errors to process exit codes.
- `src/cli/mod.rs` - Presentation module root exporting CLI types.
- `src/cli/args.rs` - Clap command hierarchies and `sanitize_args` for `git claw` compatibility.
- `src/cli/output.rs` - ANSI color and stderr/stdout formatting helpers.
- `src/core/mod.rs` - Pure domain layer module root.
- `src/infra/mod.rs` - Infrastructure layer module root.
- `src/workflow/mod.rs` - Application layer module root with stub dispatcher.
- `tests/test_cli.rs` - Integration test suite covering CLI flags, subcommand routing, and transparent `claw` prefix stripping.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false` (no patches required; clean first pass)

### Verification performed
- `cargo check`: Passed with 0 errors and 0 warnings.
- `cargo test`: 6 passed, 0 failed, 0 warnings.
- `cargo run -- --help`: Output correct usage with code 0.
- `cargo run -- --version`: Output `git-claw 0.1.0` with code 0.
- `cargo run -- claw --help`: Handled `claw` transparently with code 0.

### Residual risks
- None. Module boundaries are established and baseline tests pass.
