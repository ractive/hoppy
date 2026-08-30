---
title: Iter-85 — embed git SHA in `-V` for crates.io installs via .cargo_vcs_info.json
type: iteration
date: 2026-08-30
tags:
  - iteration
  - dx
  - release
status: in-progress
branch: iter-85/vcs-info-provenance
---

# Iter-85 — provenance for `cargo install hoppy-cli`

## Why

`hoppy -V` prints `hoppy 0.7.0 (3d32c922ab92 2026-08-13)` for Homebrew/CI builds
(`GIT_COMMIT` env or a live `.git` tree), but a `cargo install hoppy-cli` from
crates.io prints bare `hoppy 0.7.0`: the registry tarball has no `.git`, so
`build.rs` degrades to empty strings. The tarball *does* contain
`.cargo_vcs_info.json`, written by `cargo publish` with the packaging commit.

## Design

`build.rs` resolution order becomes:

1. `CARGO_HOPPY_FORCE_NO_GIT=1` → empty (unchanged)
2. `GIT_COMMIT` env → SHA from env, date from `GIT_COMMIT_DATE` or git (unchanged)
3. **new:** `.cargo_vcs_info.json` next to `Cargo.toml` → first 12 hex chars of
   `git.sha1`, `+dirty` if the file says `"dirty":true`; date from
   `GIT_COMMIT_DATE` only (the file carries no date) → `hoppy 0.7.0 (3d32c922ab92)`
4. git shell-out (unchanged)

The file is checked *before* git so a tarball extracted inside some unrelated
repo can't pick up that repo's HEAD. Parsing is a dependency-free string scan.

## Tasks

- [x] `.cargo_vcs_info.json` fallback in `crates/hoppy-cli/build.rs`
- [x] Verified: `cargo package` → extract to scratch → build → `hoppy 0.7.0 (f4f878570bd7)`
- [x] fmt / clippy / test gates
- [x] PR

## Acceptance criteria

- [ ] `cargo install hoppy-cli` (next release) prints the SHA in `-V`
- [x] Git-checkout and `GIT_COMMIT` builds unchanged
