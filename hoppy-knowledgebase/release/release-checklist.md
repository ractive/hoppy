---
title: Release Checklist
type: docs
date: 2026-05-09
tags:
  - release
  - checklist
status: active
---

# Release Checklist

Step-by-step checklist for cutting a hoppy release. Follow in order.

## Pre-flight

- [ ] All planned iteration PRs merged to `main`
- [ ] `CHANGELOG.md` has a `## [X.Y.Z] - YYYY-MM-DD` entry (not `[Unreleased]`)
- [ ] `[workspace.package].version` in root `Cargo.toml` matches the planned tag
- [ ] `cargo fmt` — no formatting issues
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` — no warnings
- [ ] `cargo test --workspace --quiet` — all tests pass
- [ ] `cargo deny check` — licenses and advisories clean
- [ ] Secrets set in GitHub repository settings:
  - `CARGO_TOKEN` — crates.io API token (owner: james)
  - `HOMEBREW_TAP_TOKEN` — fine-grained PAT with `contents: write` on `ractive/homebrew-tap`
  - `SCOOP_BUCKET_TOKEN` — fine-grained PAT with `contents: write` on `ractive/scoop-bucket`

## Cut the release

- [ ] Tag the commit: `git tag vX.Y.Z && git push origin vX.Y.Z`
- [ ] Create a GitHub Release from the tag; paste the CHANGELOG entry as release notes
- [ ] Watch the Actions run: `version-check` → `security` → `build` + `linux-packages` → `release` → `crates-io` + `homebrew` + `scoop`

## Verify after release

- [ ] All GitHub Actions jobs green
- [ ] `cargo install hoppy` succeeds on a clean machine
- [ ] `brew tap ractive/tap && brew install hoppy` works on macOS
- [ ] `scoop bucket add ractive https://github.com/ractive/scoop-bucket && scoop install hoppy` works on Windows
- [ ] `hoppy --version` prints the new version string
- [ ] Release assets on GitHub include: per-target `.tar.gz`/`.zip`, `.deb`, `.rpm`, `SHA256SUMS`
- [ ] crates.io shows the new version for `hoppy`, `bunny-api-core`, and all sibling crates

## Post-release dogfood

- [ ] Install via Homebrew on a clean Mac
- [ ] Run the dogfooding playbook (`[[dogfooding/dogfooding-playbook]]`)
- [ ] File friction points as backlog items in `hoppy-knowledgebase/backlog/`

## Credential ownership

| Secret | Owner | Scope |
|--------|-------|-------|
| `CARGO_TOKEN` | james (crates.io account) | publish to crates.io |
| `HOMEBREW_TAP_TOKEN` | james (GitHub PAT) | `contents: write` on `ractive/homebrew-tap` |
| `SCOOP_BUCKET_TOKEN` | james (GitHub PAT) | `contents: write` on `ractive/scoop-bucket` |

## Version bump rule

hoppy follows [Semantic Versioning](https://semver.org):

- **Patch** (`0.1.x`): bug fixes, documentation, dependency updates
- **Minor** (`0.x.0`): new commands or flags, backwards-compatible API additions
- **Major** (`x.0.0`): breaking changes to CLI surface or crate APIs

To bump: edit `version` in `[workspace.package]` (root `Cargo.toml`) — all crates inherit it.
Run `cargo check` to confirm the version propagates cleanly before tagging.
