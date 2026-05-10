---
title: Iter-30 — drop x86_64-apple-darwin from CI and release
type: iteration
date: 2026-05-10
tags:
  - iteration
  - ci
  - release
  - infrastructure
status: completed
branch: iter-30/drop-x86-macos
---

# Iter-30 — drop x86_64-apple-darwin from CI and release

GitHub's `macos-13` runner pool has been wound down — jobs targeting it
sit indefinitely in "Waiting for a runner to pick up this job" and either
start very late or get cancelled by the scheduler. This was the cause of
the cancelled / non-starting macos-13 builds noted in iter-29.

Apple Silicon (`aarch64-apple-darwin` on `macos-latest`) already proves
macOS builds and tests pass. The Intel binary adds no CI signal and has
no obvious distribution use case for hoppy: Intel Mac users can install
via `cargo install hoppy` from crates.io, the same way the sibling
[hyalo](https://github.com/ractive/hyalo) project handles it. Drop it
entirely.

## Scope

### 1. ci.yml — drop the `x86_64-apple-darwin` matrix entry

- [ ] In `.github/workflows/ci.yml` remove the lines:
      ```yaml
      - target: x86_64-apple-darwin
        os: macos-13
      ```
- [ ] Keep `aarch64-apple-darwin` on `macos-latest`.
- [ ] No other CI changes.

### 2. release.yml — drop the same matrix entry and brew formula block

- [ ] In `.github/workflows/release.yml` remove:
      ```yaml
      - target: x86_64-apple-darwin
        os: macos-13
      ```
- [ ] Remove the `SHA_MACOS_X86=$(get_sha "hoppy-${TAG}-x86_64-apple-darwin.tar.gz")`
      line near the brew-formula step.
- [ ] In the brew-formula heredoc (`on_macos do … end`) remove the
      `on_intel do … end` block; mirror hyalo's pattern where macOS only
      lists `on_arm`.
- [ ] Audit any release-notes / changelog text that still mentions
      `x86_64-apple-darwin` and remove it.

### 3. README — add the "Intel Mac users" hint

Mirror hyalo's wording (its README reads):
> **Intel Mac users:** Homebrew bottles are only provided for Apple
> Silicon. Use `cargo install` above.

- [ ] Add the same one-liner under the install section in
      `README.md`, immediately after the cargo install / direct download
      blocks.

### 4. Verify

- [ ] Push branch, open PR. Confirm:
      - `ci.yml` runs without a `macos-13` job at all.
      - `aarch64-apple-darwin` on `macos-latest` still passes.
      - All five remaining matrix entries report `pass`.
- [ ] On the next release (out of scope for this iter), confirm the
      brew formula renders without an `on_intel` macos block and that
      `brew install ractive/tap/hoppy` succeeds on Apple Silicon.

## Out of scope

- Any other macos runner changes (e.g. moving `macos-latest` to a fixed
  version like `macos-15`). `macos-latest` already targets Apple Silicon
  and is fine.
- Re-introducing x86_64-apple-darwin via cross-compilation from arm64
  runners. If demand surfaces later, that's a separate iter.

## Acceptance

- [ ] No `macos-13` references remain anywhere under `.github/workflows/`.
- [ ] No `x86_64-apple-darwin` references in `.github/workflows/`,
      release-notes templates, or the brew formula.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings
      && cargo test --workspace --quiet` clean.
- [ ] PR CI shows all five matrix entries green (linux x86_64, linux
      aarch64 cross, darwin aarch64, windows x86_64, windows aarch64) and
      no perpetually-queued jobs.
- [ ] README mentions cargo install as the supported path for Intel Mac.

## Related

- [[iteration-29-ci-greenify]]
- [[../decision-log]]
- hyalo's pattern: `aarch64-apple-darwin` only, README points Intel Macs
  to `cargo install`.
