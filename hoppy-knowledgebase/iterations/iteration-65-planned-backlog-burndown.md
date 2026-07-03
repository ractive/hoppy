---
title: >-
  Iter-65 — planned-backlog burndown (hourly hint, last 4 toggles, live-test
  fixes)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - cli
  - dx
  - live-api
status: completed
branch: iter-65/planned-backlog-burndown
---

# Iter-65 — planned-backlog burndown

## Why

A feasibility re-check of all 13 `status: planned` backlog items (2026-07-03)
showed 9 were already fixed by iterations 44–64 but never marked resolved.
The remaining 4 are bundled here:

- [[backlog/statistics-hourly-table-no-effect]] — `--hourly` changes
  nothing visible in table output
- [[backlog/pull-zone-update-toggle-coverage-gap]] — 29/33 toggles landed
  in iters 44–47; 4 never landed in any layer
- [[backlog/live-dns-scan-flake]] — lifecycle test still requires a
  terminal scan state within ~30s
- [[backlog/live-stream-collection-401]] — original hypothesis disproven
  (per-library key is already used); real cause undiagnosed

## Scope

### 1. `statistics --hourly` visible effect

- [x] Print a hint after the table when `--hourly` is set and format is
  table: hourly buckets are only in `--format json` (suppressed by
  `--quiet`, consistent with the global hints gate)
- [x] e2e test locking the hint (present with `--hourly`, absent without,
  absent with `--quiet` / `--format json`)

### 2. Last 4 pull-zone toggles

All four confirmed present in `specs/core-platform.json`.

- [x] `EnableBunnyImageAi`, `EnableLogging`, `EnableExtendedLogging`,
  `EnableWebSockets` added to `PullZone` read struct
- [x] Same four added to `UpdatePullZone` body (`skip_serializing_if`)
- [x] CLI flags on `pull-zone update` with dashboard-style help text,
  forwarded in `commands/pull_zone.rs`
- [x] Wiremock serialize/deserialize coverage + e2e `--help` snapshot refresh
- [x] Fixture `fixtures/core/pullzone_get.json` carries the new keys

### 3. DNS scan lifecycle flake

- [x] Relax `live_dns_zone_record_scan_lifecycle`: keep polling for a
  terminal state but stop failing on a scan that stays Pending — assert
  scan started + status is a well-typed known value (terminal-state timing
  is the API's behaviour, not hoppy's)
- [x] Verified with a live run (`TEST_BUNNY_API_KEY`)

### 4. Stream collection 401 diagnosis

- [x] Reproduce `live_stream_collection_lifecycle` in isolation against the
  test account; distinguish cleanup race vs plan feature-gate
- [x] Fix accordingly: cleanup ordering, or skip-with-message when the
  account lacks the feature
- [x] Correct the stale hypothesis in the backlog note

## Out of scope

- Non-toggle pull-zone field gaps (strings/numbers) — see
  [[research/spec-coverage/pull-zone]]
- Rendering per-bucket chart data in table format (hint is good-enough per
  the backlog item)

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean,
  `cargo test --workspace --quiet` green
- [x] Both live tests pass (or skip gracefully with a clear message) with
  `BUNNY_API_KEY=$TEST_BUNNY_API_KEY --features live-api`
- [x] All four backlog items moved to `resolved`/documented follow-up

## Related

- [[iteration-44-pull-zone-security-compliance]] … [[iteration-47-pull-zone-firewall-and-rate-limiting]]
- [[iteration-55-deterministic-chart-ordering]]
- [[iteration-57-quiet-flag-contract]]
