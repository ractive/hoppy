---
title: Iter-91 - statistics float crash, missing container zone, search coverage
type: iteration
date: 2026-10-27
tags: [iteration, statistics, pull-zone, search, containers]
status: completed
branch: iter-91/demo-fixes
---

# Iter-91 - statistics float crash, missing container zone, search coverage

Backlog: [[backlog/statistics-origin-response-time-chart-float]],
[[backlog/pull-zone-list-misses-second-container-zone]],
[[backlog/search-misses-containers-and-databases]]. Found while preparing a demo on the private account.

## Tasks

### statistics
- [x] `AccountStatistics.origin_response_time_chart` deserializes fractional values
- [x] Audit the other `*_chart` maps and averages in `AccountStatistics` and the per-zone, script and container statistics types against the OpenAPI spec; widen any that can be fractional
- [x] Fixture and test with a fractional `OriginResponseTimeChart` value; table output still renders
- [x] `Cache Hit Rate` showed 877% because `CacheHitRate` is already a percentage (live `8.77`) and was multiplied by 100; fixture now has `8.77…`, test asserts `8.77%`

### pull-zone list / count
- [x] Zones whose `MagicContainersAppId` belongs to a container app but that no endpoint `pullZoneId` names are listed and counted
- [x] Fixture and e2e test: two pull zones on one endpoint, both listed with the `App` column, no duplicates

### search
- [x] `search` also matches container app names and database names (types `container`, `db`)
- [x] Pull zones of matched container apps are added (type `cdn`), deduplicated against `/search` hits
- [x] `--server-only` keeps the single `/search` request; a failed client-side lookup prints a warning to stderr instead of failing
- [x] Unit and e2e tests; MANUAL.md updated

### Gates and dogfooding
- [x] `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --quiet`
- [x] Read-only dogfood against the real account: `statistics`, `pull-zone list`, `pull-zone count`, `search study2go`

## Acceptance criteria
- [x] `hoppy statistics` succeeds on an account with fractional origin response times
- [x] `hoppy pull-zone list` shows `6442623 study2go` on the private account
- [x] `hoppy search study2go` shows the container app, both databases and both pull zones
