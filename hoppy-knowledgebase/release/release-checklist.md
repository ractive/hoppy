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

The release pipeline runs on the shared reusable workflow in
[ractive/release-workflows](https://github.com/ractive/release-workflows)
(pinned in `.github/workflows/release.yml`); the repo-local `release.yml` is a
thin caller. Publishing a GitHub Release triggers it automatically.

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
  - `CLOUDSMITH_API_KEY` — pushes the `.deb`/`.rpm` to the hosted apt/yum repos
    (`ractive/hoppy` on Cloudsmith); non-blocking if absent
  - `AUR_SSH_PRIVATE_KEY` — pushes the `hoppy-bin` PKGBUILD to the AUR
    (`aur-package: hoppy-bin` in `release.yml`); non-blocking if absent
- [ ] (Optional) Rehearse with a dry run: `gh workflow run release.yml`
  (`workflow_dispatch` sets `dry-run: true` — builds everything, publishes nothing)

## Cut the release

- [ ] Publish the release, which creates the tag and generated notes in one step:
  `gh release create vX.Y.Z --generate-notes` (do **not** tag manually)
- [ ] The `release` workflow triggers automatically on publish and builds the
  full 7-target matrix (incl. `x86_64`/`aarch64` musl statics), attaches
  archives + `.deb`/`.rpm` + SBOMs + build-provenance attestations, publishes
  crates, and updates the Homebrew tap, Scoop bucket, and Cloudsmith repos
- [ ] Watch it: `gh run watch` (or the Actions tab)

## Verify after release

- [ ] All GitHub Actions jobs green
- [ ] `cargo install hoppy-cli` succeeds on a clean machine (binary is `hoppy`)
- [ ] `brew install ractive/tap/hoppy` works on macOS (and Linux — installs the musl static)
- [ ] apt: `curl -sLf 'https://dl.cloudsmith.io/public/ractive/hoppy/cfg/setup/bash.deb.sh' | sudo bash && sudo apt install hoppy`
- [ ] dnf: `curl -sLf 'https://dl.cloudsmith.io/public/ractive/hoppy/cfg/setup/bash.rpm.sh' | sudo bash && sudo dnf install hoppy`
- [ ] `scoop bucket add ractive https://github.com/ractive/scoop-bucket && scoop install hoppy` works on Windows
- [ ] AUR: check the AUR job's log even when the run is green — it is
  `continue-on-error`, so a failed push won't fail the workflow. The first
  release after enabling creates the `hoppy-bin` package; verify at
  <https://aur.archlinux.org/packages/hoppy-bin> (`yay -S hoppy-bin`)
- [ ] `hoppy --version` prints the new version string
- [ ] Release assets on GitHub include: per-target `.tar.gz`/`.zip` (incl. musl),
  `.deb`, `.rpm`, `SHA256SUMS`, SBOMs, and attestations
- [ ] `gh attestation verify hoppy-vX.Y.Z-<target>.tar.gz --owner ractive` passes for a native archive
- [ ] crates.io shows the new version for `hoppy-cli`, `bunny-net-api`, and `bunny-syslog-receiver`

### Recovery

- [ ] If crates.io publish failed but the release is otherwise good, re-run
  `publish-crates.yml` (`gh workflow run publish-crates.yml`)
- [ ] If the Cloudsmith upload failed, re-run `cloudsmith-republish.yml`

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
