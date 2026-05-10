---
title: >-
  Iter-32 — consolidate `bunny-api-*` into one `bunny-api` crate, split CLI as
  `hoppy-cli`
type: iteration
date: 2026-05-10
tags:
  - iteration
  - refactor
  - workspace
  - release
status: completed
branch: iter-32/consolidate-crates
---

# Iter-32 — consolidate the `bunny-api-*` crates

The current 9-crate split (`bunny-api-core`, `-compute`, `-containers`,
`-database`, `-recording`, `-shield`, `-storage`, `-stream`, plus
`bunny-syslog-receiver`) was structured for a future that hasn't arrived:
no downstream consumer wants just one service, every release bumps all
nine in lockstep, and the per-crate boilerplate (client struct, retry
config, base URL) shows up multiplied by seven any time a cross-cutting
change lands.

Mirror the hyalo project's shape:

- one `bunny-api` library crate with services behind cargo features
- `bunny-syslog-receiver` stays separate (genuinely standalone)
- the binary moves to `crates/hoppy-cli` (decoupled crate name `hoppy-cli`,
  binary name `hoppy` — same trick hyalo uses for `hyalo-cli`/`hyalo`)

Now is the right window: nothing has been published to crates.io yet, so
no downstream user breaks. After this lands, `cargo install hoppy-cli`
becomes the install line.

## Target shape

```
crates/
  bunny-api/                 # one lib, services as modules behind features
    src/
      lib.rs                 # re-exports per feature
      core/                  # ex bunny-api-core: auth, redact, statistics, dns, etc.
      compute/               # ex bunny-api-compute: pull zones, edge scripts, storage zones
      containers/            # ex bunny-api-containers: magic containers
      database/              # ex bunny-api-database: libSQL
      recording/             # ex bunny-api-recording
      shield/                # ex bunny-api-shield: WAF/security
      storage/               # ex bunny-api-storage: storage files
      stream/                # ex bunny-api-stream: video
  bunny-syslog-receiver/     # unchanged — no bunny.net API surface
  hoppy-cli/                 # binary crate, replaces root-level hoppy crate
    src/                     # was hoppy/src
    tests/                   # was hoppy/tests
```

Workspace `Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = ["crates/*"]
default-members = ["crates/hoppy-cli"]

[workspace.dependencies]
bunny-api = { path = "crates/bunny-api", version = "0.3.0" }
bunny-syslog-receiver = { path = "crates/bunny-syslog-receiver", version = "0.3.0" }
# ...external deps unchanged
```

## Scope

### 1. Create `crates/bunny-api`

- [x] `cargo new --lib crates/bunny-api`.
- [x] Move each `crates/bunny-api-<name>/src/` into
      `crates/bunny-api/src/<name>/`.
- [x] Update `crates/bunny-api/src/lib.rs` to expose each module
      gated by a feature:
      ```rust
      #[cfg(feature = "core")]    pub mod core;
      #[cfg(feature = "compute")] pub mod compute;
      // ... etc
      ```
- [x] In `Cargo.toml [features]`:
      ```toml
      default = ["core", "compute", "containers", "database",
                 "recording", "shield", "storage", "stream"]
      core = []
      compute = ["core"]
      containers = ["core"]
      database = ["core"]
      recording = ["core"]
      shield = ["core"]
      storage = ["core"]
      stream = ["core"]
      ```
- [x] Move shared types currently in `bunny-api-core` (auth helpers,
      error mappers, redact config, retry, etc.) so each service module
      consumes them as `crate::core::auth` rather than
      `bunny_api_core::auth`. Update all `use` paths.
- [x] Pull each crate's external deps into the new `Cargo.toml`'s
      `[dependencies]`. Audit for duplicates / version drift.
- [x] Move tests from each `crates/bunny-api-*/tests/` into
      `crates/bunny-api/tests/<service>/`.

### 2. Move binary to `crates/hoppy-cli`

- [x] Move `src/`, `tests/`, `completions/`, `man/` from the workspace
      root into `crates/hoppy-cli/`.
- [x] Create `crates/hoppy-cli/Cargo.toml` that mirrors hyalo-cli's:
      ```toml
      [package]
      name = "hoppy-cli"
      description = "CLI for bunny.net cloud and edge services"
      keywords = ["bunny", "cdn", "cli", "dns", "edge"]
      categories = ["command-line-utilities"]
      readme = "../../README.md"
      version.workspace = true
      edition.workspace = true
      license.workspace = true
      repository.workspace = true

      [[bin]]
      name = "hoppy"
      path = "src/main.rs"

      [[test]]
      name = "e2e"
      path = "tests/e2e/mod.rs"

      [dependencies]
      bunny-api = { workspace = true }
      bunny-syslog-receiver = { workspace = true }
      # ...rest unchanged
      ```
- [x] Remove the top-level `[package]` / `[[bin]]` / `[[test]]` /
      `[package.metadata.deb]` / `[package.metadata.generate-rpm]` blocks
      from the root `Cargo.toml`. The root becomes pure `[workspace]`.
- [x] Move the `.deb` / `.rpm` package metadata from the root
      `Cargo.toml` to `crates/hoppy-cli/Cargo.toml`. Update asset paths
      (relative paths shift — the `target/release/hoppy` binary still
      lands in workspace `target/release/`, but completions/man paths
      now relative to `crates/hoppy-cli/`).
- [x] Update all `use bunny_api_*::…` imports across the CLI to
      `use bunny_api::<service>::…`.

### 3. Update `release.yml` and `ci.yml`

- [x] Replace the 9 sequential `cargo publish --package bunny-api-*`
      blocks with a much shorter sequence:
      1. `cargo publish --package bunny-api`
      2. wait-for-index loop (the pattern already in release.yml)
      3. `cargo publish --package bunny-syslog-receiver`
      4. wait-for-index loop
      5. `cargo publish --package hoppy-cli`
- [x] Update version-check job: it currently looks up the `hoppy`
      package. Switch to `hoppy-cli`.
- [x] Update Homebrew formula section:
      - `def install` block stays `bin.install "hoppy"` (binary name
        unchanged).
      - The release-notes hint about `cargo install hoppy` becomes
        `cargo install hoppy-cli`.
- [x] CI `--help` smoke test invocation is via the workspace target
      and unchanged (`cargo run -- --help` still works because of
      `default-members = ["crates/hoppy-cli"]`).

### 4. Update README and docs

- [x] `README.md`: change `cargo install hoppy` →
      `cargo install hoppy-cli`. Note the binary is still named `hoppy`.
- [x] `CLAUDE.md` "New crates go in `crates/` with naming convention
      `bunny-api-<domain>`" — update to reflect the consolidation.
      Document the new convention: services are modules of
      `bunny-api`, gated by features.
- [x] `decision-log.md`: add an entry for this iteration with the
      rationale (premature splitting, hyalo's pattern, no downstream
      consumer affected because nothing's published).
- [x] Update any iteration plans that reference the removed crates by
      name (mostly historical — leave them be unless a still-active
      plan references them).

### 5. Version bump and `cargo update`

- [x] Bump workspace to `0.3.0` (semantic: structural breaking change
      to the public Rust API surface; the CLI binary itself is
      backwards-compatible with v0.2.0).
- [x] Re-run `cargo update`, `cargo audit`, `cargo deny check` — all
      should still pass; no new external deps were introduced, only
      moved.

## Out of scope

- Renaming the `hoppy` binary itself. It stays `hoppy`.
- Changing the public API of any service module beyond namespace
  renames. Any breaking type changes belong in a separate iteration.
- Splitting `bunny-syslog-receiver` further — it stays as-is.
- Adding any new bunny.net API coverage.

## Migration risks and mitigations

| Risk | Mitigation |
|---|---|
| Hidden cross-crate `pub(crate)` items become inaccessible after merge | Compiler will surface every one — fix as they appear |
| Tests buried in `crates/bunny-api-*/tests/` reference internals that aren't `pub` | Either lift tests into the corresponding service module as `#[cfg(test)] mod tests`, or expose under `#[cfg(any(test, feature = "test-utils"))]` |
| Test fixture file paths break after move | `git mv` preserves history; relative `include_str!` paths get a search-and-replace |
| `Cargo.lock` ordering shifts cause weirdness | `rm Cargo.lock && cargo build` once mid-iteration to clean-slate |
| Multiple `crates/bunny-api-*` crates hold conflicting external dep versions today | Audit with `cargo tree -p bunny-api-* -e features` before consolidation; pick one version per dep going in |

## Acceptance

- [x] `crates/bunny-api/` exists, `crates/bunny-api-*` directories gone.
- [x] `crates/hoppy-cli/` exists, root-level `src/` / `tests/` gone.
- [x] Root `Cargo.toml` is `[workspace]`-only with
      `default-members = ["crates/hoppy-cli"]`.
- [x] `cargo run -- --help` from workspace root still prints the CLI
      help (proof that `default-members` is wired).
- [x] `./target/release/hoppy --version` prints `hoppy 0.3.0 (...)`.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings
      && cargo test --workspace --quiet` all clean.
- [x] `cargo audit && cargo deny check` clean.
- [x] `release.yml`'s crates-io job is reduced to 3 publish steps
      (bunny-api, bunny-syslog-receiver, hoppy-cli) instead of 10.
- [x] No reference to `bunny-api-core` / `bunny-api-compute` / etc.
      remains anywhere outside historical iteration plans (which we
      leave as-is, since they're a record of past work).
- [x] Local test against the test bunny.net account with
      `BUNNY_API_KEY=$TEST_BUNNY_API_KEY ./target/release/hoppy auth check`
      succeeds — the consolidation hasn't broken any wire-level behaviour.

## Related

- [[iteration-29-ci-greenify]]
- [[iteration-30-drop-x86-macos]]
- [[iteration-31-release-prep]] (released as v0.2.0; this iteration
  prepares v0.3.0)
- [[../decision-log]]
- hyalo's pattern: `hyalo-core` (lib) + `hyalo-cli` (bin), one publish
  step per crate, binary name decoupled from package name.
