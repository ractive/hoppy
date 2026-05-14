---
title: Iter-37 — drift-tolerant CLI e2e snapshots
type: iteration
date: 2026-05-14
tags:
  - iteration
  - testing
  - fixtures
status: completed
branch: iter-37/cli-snapshot-filters
---

# Iter-37 — drift-tolerant CLI e2e snapshots

## Why

Iter-36 made the `bunny-net-api` wiremock tests drift-tolerant, but the
end-to-end validation revealed a **second** layer of value-coupling: 10
CLI e2e tests in `crates/hoppy-cli/tests/e2e/` that use `insta` snapshots
or `stdout.contains("150000")` style asserts on values that came from
the hand-authored fixtures. Until these are loosened, `fixture-refresh
--apply` after a live sweep still breaks the offline suite.

See [[../backlog/cli-snapshot-tests-value-coupled]] for the diagnosis
(filed during iter-36).

## Target shape

- `cargo test --workspace --quiet` passes against the current fixtures
  AND against fixtures refreshed by `fixture-refresh --apply` from a
  live sweep.
- CLI tests assert on structure (column headers, JSON keys, types,
  presence) rather than specific values that came from the fixture.
- `insta` snapshots either disappear or use filters that rewrite
  volatile values to placeholders before comparison.

## Scope

### 1. Identify volatile patterns in the 10 failing tests

The 10 tests (from iter-36 §6 validation):
- `cli_auth::auth_check_{json,table}` — balance and charges
- `cli_dns::dns_zone_{create_json, list_json, list_table}` — zone IDs
- `cli_pull_zone::pull_zone_{create_json, get_json, get_table}` — IDs, names
- `cli_statistics::account_statistics_{json,table}` — bandwidth values

- [x] For each failing test, list the specific values that come from
      the fixture and could plausibly drift (IDs, balances, counts,
      timestamps, hostnames).
- [x] Classify each: insta snapshot, `assert!(stdout.contains(...))`,
      `assert_eq!(json[...], ...)`.

### 2. Rewrite per failure mode

- [x] **insta snapshots**: prefer per-snapshot `with_settings!(filters
      => …)` that rewrite volatile patterns to placeholders before
      comparison. Common filters:
  - IDs: `r"\b\d{4,}\b"` → `"[id]"`
  - Money: `r"\$\d+(?:\.\d+)?"` → `"$[amount]"`
  - Bandwidth (GB): `r"\d+(?:\.\d+)?\s*GB"` → `"[bandwidth]"`
  - Domain names that came from live recordings: per-test filter.
- [x] **`stdout.contains("150000")`**: replace with a regex check that
      the column header / numeric field shape is present, not the
      specific value.
- [x] **`assert_eq!(json[...], specific_number)`**: replace with
      `assert!(json[...].is_number())` and (optionally) an invariant
      like `>= 0` or `is_finite()`.

### 3. Re-snapshot

- [x] `cargo insta accept` for any legitimate snapshot updates (filters
      added → snapshot bodies become "[id]" instead of "1001").
- [x] Confirm the new snapshot is drift-tolerant by hand — does it
      still reject regressions in column ordering, missing keys, etc.?

### 4. Verify against simulated drift

- [x] Apply the 14 drifts from iter-34's last dogfooding round (see
      [[iteration-34-fixture-mapper#Outcome]]).
- [x] Run `cargo test --workspace --quiet`. **All affected CLI tests
      must pass.**
- [x] Revert: `git checkout -- fixtures/`.

### 5. Document

- [x] Extend the "Shape-first asserts" section of the dogfooding
      playbook (added in iter-36) with a sibling subsection on
      drift-tolerant `insta` snapshots and `stdout.contains` checks.

### 6. End-to-end dogfooding

- [x] Once §1–§5 land: run a fresh live sweep, `fixture-refresh
      --apply`, and `cargo test --workspace --quiet`. If green, commit
      the resulting fixture drift — closing iter-34's deferred §5 goal.

## Out of scope

- Rewriting CLI tests that aren't in the 10 failing list (we'll fix the
  next layer when the next refresh surfaces them).
- Switching from `insta` to a different snapshot library.
- Changing the recording framework or the fixture-refresh tool.

## Acceptance

- All 10 affected CLI tests use shape-tolerant assertions / filtered
  snapshots.
- `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
  cargo test --workspace --quiet` clean.
- Iter-34's deferred drift commit either lands here or is explicitly
  re-deferred with a reason.

## Related

- [[../backlog/cli-snapshot-tests-value-coupled]] — motivating backlog item.
- [[iteration-36-shape-asserts]] — first half of the fix (bunny-net-api crate).
- [[iteration-34-fixture-mapper]] — the tool whose final step this unblocks.
