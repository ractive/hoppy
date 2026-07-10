---
title: Iter-80 — musl static targets (drop vestigial libssl) + winget bootstrap
type: iteration
date: 2026-07-11
tags:
  - iteration
  - ci
  - release
  - packaging
status: planned
branch: iter-80/musl-targets-winget
---

# Iter-80 — musl static targets + winget bootstrap

## Why

Two distribution-parity gaps remain versus hyalo/ff-rdp after iter-79
(shared release pipeline):

1. **No musl static Linux builds.** The long-standing reason — "hoppy links
   OpenSSL" — is no longer true (and may never have been for the current
   dependency tree): the workspace pins
   `reqwest = { default-features = false, features = ["json", "query",
   "rustls"] }`, the only openssl-named crate in `Cargo.lock` is the
   pure-Rust `openssl-probe` (CA-path probing, no libssl linkage), and
   neither `openssl-sys` nor `native-tls` appears anywhere
   (`cargo tree -i openssl-sys` → no match). The `libssl-dev` pre-build
   block in `Cross.toml` is vestigial. No rustls migration is needed —
   it is already done; only the release matrix and Cross.toml lag behind.
2. **No winget distribution.** The shared workflow's `winget` job
   (vedantmgoyal9/winget-releaser) can only *update* packages that already
   exist in microsoft/winget-pkgs — `ractive.hoppy` needs a one-time
   manual bootstrap submission before the automated job can take over.

## Scope A — musl static targets

Tasks:

- [ ] Remove the vestigial `libssl-dev` pre-build block from `Cross.toml`
      (keep the `GIT_COMMIT`/`GIT_COMMIT_DATE` passthrough added in
      iter-79)
- [ ] Add `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`
      (`cross: true`, `run_tests: false`) to the caller's `targets` matrix
      in `.github/workflows/release.yml`
- [ ] Confirm ring/rustls compile under the cross musl images (they ship a
      C toolchain; expected to work out of the box)
- [ ] Dispatch a dry run on the branch and verify the bundle contains
      `hoppy-vX.Y.Z-{x86_64,aarch64}-unknown-linux-musl.tar.gz` with
      completions/man included
- [ ] Note in the PR body: the shared Homebrew job prefers musl artifacts,
      so linuxbrew users switch from gnu-linked to static musl binaries

Acceptance criteria:

- [ ] `cargo tree` shows no `openssl-sys`/`native-tls` after the change
      (CI clean)
- [ ] Dry-run bundle contains both musl tarballs; `file` on an extracted
      musl binary reports a statically linked executable
- [ ] Existing gnu/macOS/Windows artifacts unchanged (naming and contents)

Non-goal: swapping the allocator. musl's malloc is slower under heavy
multithreaded load, but hoppy is an I/O-bound CLI; revisit mimalloc only if
profiling ever says so.

## Scope B — winget bootstrap

Tasks:

- [ ] Add the `WINGET_TOKEN` secret (classic PAT, `public_repo` scope —
      same token hyalo/ff-rdp use) to ractive/hoppy
- [ ] One-time manual submission of `ractive.hoppy` to
      microsoft/winget-pkgs via `komac` or `wingetcreate`, referencing the
      latest release's Windows zips (the
      `hoppy-vX.Y.Z-<arch>-pc-windows-msvc.zip` naming already matches the
      shared workflow's installer regex)
- [ ] Wait for winget-pkgs moderation to merge the bootstrap PR
- [ ] Add `winget-identifier: ractive.hoppy` to the caller

Acceptance criteria:

- [ ] `winget install ractive.hoppy` resolves and installs
- [ ] The next release's `winget` job submits the version-update PR
      automatically (non-blocking job green, CI clean)

## Sequencing

- Depends on iter-79 (PR #91) merging first — the caller `targets` matrix
  this iteration edits only exists on that branch.
- Scope A and B are independent; split into two PRs if winget-pkgs
  moderation lag would otherwise stall the musl work.

## Related

- [[iteration-79-shared-release-workflow]]
- ractive/release-workflows README ("Linux distro publishing", winget notes)
- hyalo knowledgebase DEC-048 (shared-pipeline change protocol)
