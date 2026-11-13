---
tracker_id: ""
type: epic
title: "Repository Polish, Agent JSON Output, Interactive Init & Binary Releases"
parent: initiative-git-claw
covers: [FR-6, FR-10, FR-15]
after: ["epic-dx-docker-and-init"]
assignee: ""
risk: low
---

# Repository Polish, Agent JSON Output, Interactive Init & Binary Releases

## Description

Brings the repository to release-ready, contributor-friendly, and AI-agent-ready quality across code, packaging, CI/CD, and documentation:
1. Structured `--json` output on core lifecycle and inspection commands (`list`, `start`, `finish`, `publish`, `spike drop`, `init`) for AI coding agents and automation scripts.
2. Real interactive setup and overwrite protection in `git claw init` when `--yes` is omitted, matching documented behavior while preserving instant `-y`/`--yes` execution.
3. Metadata consistency (`Cargo.toml` license `GPL-2.0-only`, `CHANGELOG.md` cleanup) and automated cross-platform prebuilt binary releases on GitHub Releases (`.github/workflows/release.yml`).
4. Full synchronization of `README.md`, `CHEATSHEET.md`, `docs/index.html`, and a new copy-pasteable `skills/git-claw/SKILL.md` so documentation matches every implemented flag/config field and contains zero emojis.

## Done when

1. `--json` emits valid, structured JSON on stdout for `list`, `start`, `finish`, `publish`, `spike drop`, and `init` without altering default human output.
2. `git claw init` without `--yes` prompts interactively before overwriting existing `.git-claw.toml` and lets users confirm/customize defaults; `git claw init --yes` stays non-interactive.
3. `Cargo.toml` specifies `GPL-2.0-only`, `CHANGELOG.md` typos are fixed, and `.github/workflows/release.yml` defines cross-platform binary builds and SHA256 checksums on `v*` tags.
4. `README.md`, `CHEATSHEET.md`, `docs/index.html`, and `skills/git-claw/SKILL.md` accurately reflect current CLI/TOML features with zero emojis.
5. `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `tickets.py status` pass cleanly.
