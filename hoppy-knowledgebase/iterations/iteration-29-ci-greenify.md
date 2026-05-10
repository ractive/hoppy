---
title: Iter-29 — make CI green on all five targets
type: iteration
date: 2026-05-10
tags:
  - iteration
  - ci
  - infrastructure
status: in-progress
branch: iter-29/ci-greenify
---

# Iter-29 — make CI green on all five targets

For at least the last several iterations, two CI jobs have been failing on
every PR and merging anyway:

- `build (x86_64-pc-windows-msvc, windows-latest)` — fails one snapshot
  test
- `build (aarch64-unknown-linux-gnu, ubuntu-latest, true)` — fails before
  it can compile because `openssl-sys` doesn't recognise the OpenSSL in
  `cross`'s docker image
- `build (x86_64-apple-darwin, macos-13)` — frequently shown as
  `cancelled` rather than starting; investigate if this also blocks once
  the others are green

Goal of this iter: every CI job passes on a fresh PR with no manual
hand-waving.

## Scope

### 1. Windows snapshot brittleness (~10 min)

Failure (verbatim from the most recent run):

```
-Usage: hoppy container logs [OPTIONS] --app-id <APP_ID>
+Usage: hoppy.exe container logs [OPTIONS] --app-id <APP_ID>
```

clap renders `argv[0]` differently between Windows (`hoppy.exe`) and
Unix-likes (`hoppy`). Snapshots were captured on Unix.

- [x] Add an `insta` filter that normalises `\.exe$` (or replaces
      `hoppy.exe` → `hoppy`) on the binary name in CLI help/output
      snapshots. Apply repo-wide via `insta`'s settings file or a small
      test helper, not per-test.
- [x] Re-run the failing test locally on Linux to confirm it still passes,
      then push and verify Windows job passes on PR CI.
- [x] Audit other `_help` and CLI-output snapshots for the same pattern
      so a future Windows-only divergence doesn't catch us again.

### 2. aarch64-linux — switch reqwest to rustls (~15 min)

Root cause: workspace dep `reqwest = { version = "0.12", features = ["json"] }`
defaults to **native-tls** → pulls **openssl-sys** → fails when `cross`'s
docker image OpenSSL doesn't match what openssl-sys 0.9.x knows.

The fix is to drop the openssl-sys dep entirely by switching reqwest to
rustls. Pure-Rust TLS, identical build on every platform, no system
dependency.

- [x] Update workspace `Cargo.toml`:
      ```toml
      reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
      ```
      And the per-binary override that adds `"stream"` should keep
      `default-features = false` consistent.
- [x] Run `cargo build --workspace --release` locally to confirm
      openssl-sys disappears from `cargo tree`.
- [ ] Sanity-check a real API call: `BUNNY_API_KEY=$TEST_BUNNY_API_KEY
      ./target/release/hoppy auth check` — succeeds against
      `api.bunny.net`'s LE-issued cert (rustls webpki roots cover Let's
      Encrypt, so this should just work).
- [ ] If a future caller needs to honour the OS trust store, add
      `rustls-tls-native-roots`. Not needed for bunny.net.
- [x] Remove any `openssl` / `openssl-sys` mentions from `Cargo.lock`
      (they should no longer be in the graph at all) and from
      `decision-log.md` if referenced.

### 3. macos-13 — investigate the "cancelled" pattern

The macos-13 job has been "cancelled" rather than running on the last few
PRs (iter-27, iter-28). Plausible causes:

- The macos-13 image is being phased out by GitHub Actions and the runner
  pool is busy / unavailable.
- Some other job in the matrix is using `fail-fast: true` (it isn't —
  matrix has `fail-fast: false`), so this is an Actions-side issue rather
  than ours.

- [ ] Check the cancellation reason on the next CI run after the rustls
      switch lands. If the macos-13 runner is being deprecated, drop it
      from the matrix (`x86_64-apple-darwin` will then be only built as
      a release artifact on macos-13 in `release.yml` — review there
      too).
- [ ] If macos-13 is fine but cancelled by some other CI workflow
      (concurrency group mis-configured), fix the concurrency rule.

## Out of scope

- The `aarch64-unknown-linux-gnu` matrix entry already has the `cross: true`
  suffix in its name. After rustls, drop the suffix-only "(true)" naming
  artifact only if it becomes confusing — currently it's harmless.
- No new functionality. No CLI changes. Pure CI hygiene.

## Acceptance

- [ ] All five CI jobs report `pass` on a fresh PR.
- [x] `openssl-sys` no longer appears in `cargo tree --workspace` output.
- [x] Locally: `cargo fmt`, `cargo clippy --workspace --all-targets --
      -D warnings`, `cargo test --workspace --quiet` all green.
- [ ] One e2e API call (e.g. `auth check`) succeeds against bunny.net
      with the rustls build.

## Related

- [[iteration-27-dogfooding-bugfixes]]
- [[iteration-28-dogfooding-ux-polish]]
- [[../decision-log]]
