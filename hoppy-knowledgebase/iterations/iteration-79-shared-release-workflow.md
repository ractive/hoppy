---
title: Iter-79 — migrate release pipeline to shared ractive/release-workflows
type: iteration
date: 2026-07-10
tags:
  - iteration
  - ci
  - release
  - tooling
status: in-progress
branch: iter-79/shared-release-workflow
---

# Iter-79 — migrate release pipeline to shared ractive/release-workflows

## Why

hoppy, hyalo, and ff-rdp maintained near-duplicate `release.yml` files that
had to be manually patched in lockstep for every pipeline fix (hermetic
build provenance, per-target rust-cache keys, "already exists on crates.io
index" handling, etc). A new reusable-workflow repo,
[ractive/release-workflows](https://github.com/ractive/release-workflows),
extracts the shared pipeline behind a `workflow_call` interface so fixes
propagate by bumping a pinned version tag instead of three manual ports.

## Scope

- [x] Replace `.github/workflows/release.yml` with a thin caller of
      `ractive/release-workflows/.github/workflows/release.yml@v0.1.0`,
      preserving hoppy's real completions/man generation, deb/rpm asset
      layout, 5-target matrix (no musl — hoppy links OpenSSL), and Homebrew
      bore-cli caveats.
- [x] Add `workflow_dispatch` trigger with `dry-run` wired to
      `github.event_name == 'workflow_dispatch'` for pre-release validation.
- [x] Add `.github/workflows/publish-crates.yml` — a thin caller of the
      shared standalone crates.io recovery workflow. hoppy had no equivalent
      recovery path before this change.
- [x] Add `[build.env] passthrough = ["GIT_COMMIT", "GIT_COMMIT_DATE"]` to
      `Cross.toml` for the `aarch64-unknown-linux-gnu` cross target —
      `crates/hoppy-cli/build.rs` reads those vars for hermetic build
      provenance, and without the passthrough the cross container either
      shells out to its own git or silently falls back to empty values.
- [x] Validate both new workflow files with `actionlint`.
- [ ] Confirm CI quality gates (fmt, clippy, test) still pass after the
      `Cross.toml` change.
- [ ] Open PR, do not merge.

## Behavior deltas vs the old workflow

- Archive naming unchanged in substance: old workflow used
  `hoppy-${{ github.ref_name }}-<target>.tar.gz`; hoppy tags are already
  `vX.Y.Z`, so this equals the shared workflow's
  `hoppy-v<version>-<target>.tar.gz` exactly.
- New: CycloneDX SBOM + `actions/attest-build-provenance` on native targets
  (hoppy had neither).
- Completions/man generation unified into one `pre-package-command` (was
  split bash/pwsh per-OS steps in the matrix build, plus a third copy in
  `linux-packages`). The Windows step no longer uses `pwsh` — the shared
  workflow always runs `pre-package-command` under bash (git bash on
  Windows runners).
- Homebrew formula `desc` now derives from `hoppy-cli`'s Cargo.toml
  `description` field (matches the old hardcoded string) and only emits
  platform blocks for artifacts actually present in `SHA256SUMS`.
- deb/rpm packaging unchanged in substance: same
  `[package.metadata.deb]`/`[package.metadata.generate-rpm]` asset paths
  under `crates/hoppy-cli/`, same `cargo deb -p hoppy-cli --no-build
  --no-strip` / `cargo generate-rpm -p crates/hoppy-cli` invocations.
- Scoop publishing is new to the *thin caller* surface (it was already
  present in hoppy's old workflow; unchanged).

## Related

- [[decision-log]]
- Shared workflow repo: `ractive/release-workflows`
