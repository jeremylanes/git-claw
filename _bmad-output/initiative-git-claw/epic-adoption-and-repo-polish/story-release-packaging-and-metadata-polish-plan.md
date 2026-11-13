---
title: 'Package metadata alignment and cross-platform release workflow'
type: 'chore'
ticket: 3
created: '2026-11-13'
status: 'done'
route: 'full'
risk: 'low'
review: 'quick'
---

## Intent

**Problem:**
1. `Cargo.toml` specifies `license = "GPL-2.0-or-later"`, whereas `LICENSE` is GNU GPL v2 (June 1991) and `README.md` states `GPL-2.0-only`.
2. `CHANGELOG.md` has visible typos (`comitizen mode`, `chetsheet`).
3. `.github/workflows/ci.yml` publishes to `crates.io` on `v*` tags, but there is no workflow producing downloadable prebuilt binaries for Linux, macOS, and Windows on GitHub Releases — blocking adoption by Python/Django/Docker/Node developers who do not have a Rust toolchain installed.

**Approach:**
1. Update `Cargo.toml` line 6 to `license = "GPL-2.0-only"`. Also check `include` in `Cargo.toml` (e.g., include `CHANGELOG.md` and `CHEATSHEET.md` if helpful, keeping package lean).
2. Fix obvious spelling typos in `CHANGELOG.md` (`comitizen mode` -> `Commitizen mode`, `chect sheet` / `chetsheet` -> `cheat sheet`) without changing historical tags, version numbers, or commit references.
3. Add `.github/workflows/release.yml` triggered on tags matching `v*`:
   - Matrix targets:
     - `x86_64-unknown-linux-gnu` (`ubuntu-latest`) -> `.tar.gz`
     - `x86_64-unknown-linux-musl` (`ubuntu-latest`, using `musl-tools` or `cross`/static) -> `.tar.gz`
     - `aarch64-apple-darwin` (`macos-latest`) -> `.tar.gz`
     - `x86_64-apple-darwin` (`macos-latest`) -> `.tar.gz`
     - `x86_64-pc-windows-msvc` (`windows-latest`) -> `.zip`
   - Package binary + `README.md` + `LICENSE` into versioned archive `git-claw-${GITHUB_REF_NAME}-${TARGET}.tar.gz` (or `.zip`), compute `SHA256SUMS.txt`, and create/upload assets to the GitHub Release via `softprops/action-gh-release@v2` (`permissions: contents: write`, `generate_release_notes: true`).
   - Use zero emojis in workflow YAML.

## Acceptance Criteria
1. `Cargo.toml` license matches `README.md` (`GPL-2.0-only`).
2. `CHANGELOG.md` contains no `comitizen` or `chetsheet`/`chect sheet` typos.
3. `.github/workflows/release.yml` is valid YAML following GitHub Actions best practices with checksum verification and multi-OS prebuilt binaries.
4. `cargo test` and `cargo clippy --all-targets --all-features -- -D warnings` pass.
