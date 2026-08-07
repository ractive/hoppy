---
title: "Iteration 8 — Release Readiness"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - release
  - ci-cd
  - packaging
status: completed
branch: iter-8/release
---

# Iteration 8 — Release Readiness

**Goal:** Everything needed to ship v0.1.0 as a proper open-source release.

## Foundation

- [x] LICENSE file (MIT)
- [x] CHANGELOG.md for v0.1.0 (summarize all iterations)
- [x] Cargo.toml metadata: `repository`, `homepage`, `keywords`, `categories`, `readme`
- [x] Update CI workflow: trigger on push to main + PRs (not just `workflow_dispatch`), `--workspace` for clippy and test

## GitHub Actions Release Workflow

- [x] Trigger on tag push matching `v*` (e.g. `v0.1.0`)
- [x] Build matrix (6 targets):
  - `x86_64-unknown-linux-gnu` (ubuntu-latest, native)
  - `aarch64-unknown-linux-gnu` (ubuntu-latest, cross-rs)
  - `x86_64-apple-darwin` (macos-13, native)
  - `aarch64-apple-darwin` (macos-latest, native)
  - `x86_64-pc-windows-msvc` (windows-latest, native)
  - `aarch64-pc-windows-msvc` (windows-latest, native)
- [x] Package artifacts: `.tar.gz` (linux/macOS), `.zip` (Windows)
- [x] Each archive includes: binary, shell completions (bash/zsh/fish), man page, LICENSE, README
- [x] Generate `sha256sums.txt` for all archives
- [x] Create GitHub Release from tag, upload all archives + checksums
- [x] Pinned versions: cross@0.2.5, cargo-deb@3, cargo-generate-rpm@0.20 (all with --locked)

## Man Page Generation

- [x] Add `clap_mangen` dependency (xtask crate)
- [x] xtask generates 159 man pages from clap command tree
- [x] Bundle in release archives and packages

## Shell Completions

- [x] Keep stdout approach (`hoppy completions <shell>`) — industry standard
- [x] Bundle pre-generated completions in release archives
- [x] Include completions in deb/rpm/Homebrew packages (auto-installed to correct paths)
- [x] Document redirect commands in README

## Packaging

- [x] **Homebrew**: `ractive/homebrew-hoppy` tap repo created; formula auto-updated by release workflow
- [x] **cargo install**: `cargo install --git https://github.com/ractive/hoppy` documented in README
- [x] **deb**: `cargo-deb` with `[package.metadata.deb]` — completions + man pages as assets
- [x] **rpm**: `cargo-generate-rpm` with `[package.metadata.generate-rpm]` — same assets
- [x] **winget**: Submit manifest to `microsoft/winget-pkgs` after first release — done in iter-80 (microsoft/winget-pkgs#400670, merged 2026-07-24)

## README Overhaul

- [x] Installation section: Homebrew, cargo install --git, direct download, deb/rpm, build from source
- [x] Feature overview with service list
- [x] Usage examples organized by service
- [x] Shell completions with per-shell paths
- [x] Global options
- [x] Environment variables section
- [x] Badges: CI status, license
- [x] Contributing section (skipped — not needed for v0.1.0) — dropped: README still has no Contributing section as of 2026-08-07

## Not in scope for v0.1.0

- crates.io publishing (requires publishing all 6 sub-crates with proper versioning)
- Signed binaries / macOS notarization
- AUR / Nix / Scoop packages
- Auto-update mechanism
- `hoppy completions install` subcommand (stdout + package managers is sufficient)

**Deliverable:** Tagged v0.1.0 release with binaries for linux (x86_64, aarch64), macOS (x86_64, aarch64), windows (x86_64, aarch64). Installable via Homebrew, cargo install --git, direct download, deb, rpm, or winget.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-7-cleanup]] — previous iteration
- [[release/release-setup-checklist]] — one-time release setup steps
- [[research/release-engineering-research]] — research behind release decisions
- [[decision-log]] — release & packaging decisions
