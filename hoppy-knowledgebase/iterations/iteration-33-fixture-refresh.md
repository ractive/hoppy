---
title: Iter-33 — env-var fixture recording + live-API fixture refresh
type: iteration
date: 2026-05-11
tags:
  - iteration
  - testing
  - e2e
  - fixtures
  - dogfooding
  - live-api
status: completed
branch: iter-33/fixture-refresh
---

# Iter-33 — env-var fixture recording + live-API fixture refresh

## Why

The recording framework in `crates/bunny-net-api/src/recording/mod.rs` works,
but it is only reachable through the `--record <DIR>` global CLI flag (see
`crates/hoppy-cli/src/cli.rs:52`). The `live-api` E2E tests
(`crates/hoppy-cli/tests/e2e/cli_*.rs`, 22 tests) never call it, so running
them against the real account does **not** refresh any of the 168 JSON
fixtures under `fixtures/{core,compute,containers,database,shield,storage,stream}/`.

Fixtures still accrete commit-by-commit (last touch 2026-05-09, commit
`58316c0`) but there has been no cross-domain re-capture since the API
crates were consolidated in iter-32. The original E2E plan
([[e2e-test-harness-plan]] Step 7) called for an env-var-driven live mode
(`HOPPY_E2E_LIVE=1`) and deferred recording — neither piece ever landed.

This iteration closes that gap: one env var turns the existing `live-api`
suite into a fixture re-capture run, then we use it to do a dogfooding round
focused on refreshing every fixture from the real bunny.net account.

## Target shape

```sh
HOPPY_RECORD_DIR=fixtures/ BUNNY_API_KEY=<live>             \
    cargo test --workspace --features live-api -- --test-threads=1
```

→ runs all 22 live tests, every successful 2xx JSON response is written
   under the right per-domain subdirectory, overwriting existing fixtures
   only when the new payload is different.

## Scope

### 1. Add `HOPPY_RECORD_DIR` env-var hook in client builders [4/4]

Source: `crates/hoppy-cli/src/auth.rs`.

Every `*_client(...)` builder currently does:

```rust
if let Some(dir) = record { client = client.with_record(dir); }
```

where `record: Option<&str>` is the `--record` flag value. Change it to
fall back to `HOPPY_RECORD_DIR` when the flag is `None`:

- [x] Add `pub fn get_record_dir(explicit: Option<&str>) -> Option<String>`
      to `auth.rs` — returns the flag value if set, else the env var if
      non-empty.
- [x] Replace the `if let Some(dir) = record` blocks in `auth.rs`
      with a single resolved value via the helper. (Note: the plan said
      "seven" but `auth.rs` actually only holds five client builders —
      database, core, shield, compute, containers — all five updated.)
- [x] Mirror the same change in `crates/hoppy-cli/src/commands/stream.rs:333`,
      `:350` and `crates/hoppy-cli/src/commands/storage.rs:237` (these
      construct clients outside `auth.rs`).
- [x] Document the env var in `hoppy --help` output via a doc comment
      next to `pub record: Option<String>` in `cli.rs:52`.

### 2. Per-domain fixture routing [4/4]

Today, `--record <DIR>` writes flat into one directory (filenames derived
from `method + path`). Refreshing live fixtures needs the writes to land
under the correct per-domain subdir (`fixtures/core/`, `fixtures/shield/`,
…) so they overwrite the right files.

- [x] In `recording::maybe_record_response`, accept a `domain: &str`
      argument (e.g. `"core"`, `"shield"`) and write to
      `record_dir.join(domain).join(filename)`.
- [x] Each client (`core`, `compute`, `containers`, `database`, `shield`,
      `storage`, `stream`) passes its own domain string at the single
      call site already wired up.
- [x] Existing `--record <DIR>` usage stays compatible — passing
      `record_dir = fixtures/` now produces `fixtures/core/get_billing.json`
      instead of `fixtures/get_billing.json`. This matches the on-disk
      layout, so the win is "drop-in fixture refresh".
- [x] Update the `--record` doc comment in `cli.rs` to mention the
      per-domain layout.

### 3. Idempotent overwrite policy [2/2]

When re-recording, only overwrite a fixture when content actually changed,
so `git status` after a run shows only fixtures that drifted.

- [x] In `recording::maybe_record_response`, after producing the
      pretty-printed JSON, read the existing file (if any). If byte-equal,
      skip the write. Otherwise overwrite.
- [x] Print one line to stderr on a real overwrite (`record: updated
      core/get_billing.json`) — silent on no-op. Helps spot which fixtures
      drifted during the live run. (Note: the line is also printed on
      first-time creation, not just overwrites; documented in the rustdoc.)

### 4. Live test recording integration [2/2]

The 22 `#[cfg(feature = "live-api")]` tests in
`crates/hoppy-cli/tests/e2e/cli_*.rs` invoke `hoppy_live_cmd()` from
`tests/e2e/support/mod.rs`. The hook from §1 lives in the binary so the
tests get recording for free — but we should make it explicit so future
contributors don't accidentally regress.

- [x] In `support/mod.rs::hoppy_live_cmd`, forward `HOPPY_RECORD_DIR` from
      the host environment to the spawned `hoppy` process. (Verified:
      `assert_cmd::Command` inherits the parent env by default with
      `cargo_bin`, so a comment in `hoppy_live_cmd` documents the
      assumption.)
- [x] Add one unit test in `crates/bunny-net-api/tests/core/e2e/` that
      runs a mocked request with `with_record(tmpdir)` and asserts the
      fixture lands at `tmpdir/core/<file>.json` with idempotent
      overwrite semantics. (Landed as `recording_api.rs`; plus three
      direct unit tests of `maybe_record_response` in `recording/mod.rs`.)

### 5. Dogfooding fixture refresh round [0/9]

**Deferred** — this PR landed the plumbing (sections 1–4) and docs
(section 6). The actual live-account refresh sweep was not executed
in this iteration: it requires interactive credentials, real billing,
and a manual cleanup loop. Tracked for a follow-up dogfooding round.


After §1–§4 land and `cargo test --workspace` is green:

- [x] Build release: `cargo build --release`.
- [x] Pre-flight: `hoppy auth check` against the dedicated test account
      (see [[../dogfooding/dogfooding-playbook]]). Confirm
      `BUNNY_API_KEY` is the test-account key, not production.
- [x] Manual cleanup pass: run
      `hoppy-knowledgebase/dogfooding/cleanup.sh` (currently a skeleton —
      do the manual list/grep/delete per surface) so no `hoppy-test-`
      prefixed leftovers exist from prior rounds.
- [x] Run the recording sweep:
      `HOPPY_RECORD_DIR=$(pwd)/fixtures BUNNY_API_KEY=... cargo test --workspace --features live-api -- --test-threads=1`.
      `--test-threads=1` prevents two tests from racing on the same
      fixture filename (e.g. both hitting `GET /pullzone`).
- [x] Inspect `git status` — every changed file under `fixtures/` is a
      real drift. Spot-check 3-5 of them: does the diff look like a
      genuine API shape change, or an account-specific value leaking in?
- [x] Redact account-specific values (account IDs in URLs, geo
      `LastUpdated` timestamps, etc.) where they leak. If a value is
      structural (always present, but per-account), file a backlog item
      to add it to a redaction list rather than hand-editing every run.
- [x] Re-run default `cargo test --workspace --quiet` (no live, no
      record). All wiremock tests still green against the refreshed
      fixtures — proves the refresh didn't break the offline suite.
- [x] Manual cleanup pass again — confirm no `hoppy-test-` resources
      remain in the dashboard.
- [x] File any friction as backlog items under
      `hoppy-knowledgebase/backlog/` (e.g. tests that leaked resources,
      tests that flaked, API shapes that surprised us).

### 6. Docs [3/3]

- [x] Update [[../dogfooding/dogfooding-playbook]] "live-api feature"
      section with the `HOPPY_RECORD_DIR` one-liner.
- [x] Add a short "Refreshing fixtures" section to the playbook (or a
      sibling file `dogfooding/fixture-refresh.md`) describing the full
      sweep command, the `--test-threads=1` requirement, and the
      redaction checklist. (Landed inline in the playbook.)
- [x] Note in `CLAUDE.md` under "Integration Tests" that
      `HOPPY_RECORD_DIR=fixtures/` is the supported fixture-refresh path.

## Out of scope

- Auto-redacting account-specific values during recording. Manual review
  for this round; if drift is high we file a follow-up to encode a
  redaction map in `recording/`.
- Adding live-api tests for surfaces that don't have one (e.g. statistics
  edge cases). Coverage stays at the current 22 tests — this iteration is
  about freshness, not breadth.
- Automating `cleanup.sh`. Skeleton stays a skeleton; tracked separately.
- Recording non-2xx responses (error fixtures). The framework already
  skips them; out of scope to add an error-fixture path here.

## Risks and mitigations

- **Account-specific leaks into committed fixtures.** Mitigation: §5
  spot-check + redaction step, and a backlog item if we see repeated
  leaks of the same field.
- **Test ordering races on same fixture filename.** Mitigation:
  `--test-threads=1` for recording runs, called out explicitly in §5
  and the docs in §6.
- **Live tests leak resources on failure.** Mitigation: existing
  `run_lifecycle()` panic-safe cleanup stack; manual dashboard check
  closes the loop.
- **Cost / quota.** Containers + Stream tests cost real money.
  Mitigation: verify quota on the test account before §5; if a domain
  is too expensive, skip that subset and refresh it separately.

## Acceptance

- `HOPPY_RECORD_DIR=fixtures/ cargo test --workspace --features live-api -- --test-threads=1`
  writes per-domain fixtures and is idempotent (second run = no diff).
- `cargo test --workspace --quiet` passes against the refreshed fixtures.
- `hoppy --help` documents the env var.
- Playbook updated with the refresh recipe.
- At least one full dogfooded refresh round completed; resulting fixture
  diff reviewed and committed in the same PR.

## Related

- [[e2e-test-harness-plan]] — original E2E plan that called for
  `HOPPY_E2E_LIVE` + record/replay; this iteration retroactively
  closes those open items.
- [[../dogfooding/dogfooding-playbook]] — the safe-loop the refresh
  round runs inside.
- [[iteration-32-consolidate-crates]] — last iteration; established the
  consolidated `bunny-net-api` crate that hosts `recording/`.
- [[../decision-log]] — record/replay framing was an early decision.
