---
title: "Iteration 13 — Statistics & Analytics"
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - statistics
  - analytics
status: planned
branch: iter-13/statistics
---

# Iteration 13 — Statistics & Analytics

**Goal:** Add statistics/analytics endpoints across all services. This is the single biggest cross-cutting gap — every service has stats endpoints that are currently missing, making the CLI blind to usage data.

## Context

Statistics are read-only GET endpoints returning time-series or summary data. They all follow a similar pattern: accept optional `dateFrom`/`dateTo` query params and return JSON with counters/charts. The response types vary per service but are straightforward to model.

Fixtures already exist for shield detailed metrics (`fixtures/shield/metrics_overview_detailed.json`, `metrics_rate_limits.json`, `metrics_waf_rule.json`, `metrics_bot_detection.json`) — these were captured during iter 12 but the client methods were deferred.

## Scope

### 1. Shield Detailed Metrics (deferred from iter 12)

Fixtures already captured. Client methods and CLI commands need to be wired up.

**OpenAPI ref:** `specs/shield.json`

- [ ] API client (`bunny-api-shield`): `GET /shield/metrics/overview/{shieldZoneId}/detailed`
- [ ] API client: `GET /shield/metrics/rate-limits/{shieldZoneId}`
- [ ] API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/waf-rule/{ruleId}`
- [ ] API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/bot-detection`
- [ ] API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/upload-scanning`
- [ ] CLI: `hoppy shield metrics detailed <shield-zone-id>`
- [ ] CLI: `hoppy shield metrics rate-limits <shield-zone-id>`
- [ ] CLI: `hoppy shield metrics waf-rule <shield-zone-id> --rule-id <id>`
- [ ] CLI: `hoppy shield metrics bot-detection <shield-zone-id>`
- [ ] CLI: `hoppy shield metrics upload-scanning <shield-zone-id>`
- [ ] Wiremock + insta snapshot tests (fixtures already exist for 4 of 5)
- [ ] Capture `metrics_upload_scanning.json` fixture from live API
- [ ] Live E2E test: extend existing `live_shield_lifecycle` to fetch all metrics after zone creation

### 2. Storage Zone Statistics

**OpenAPI ref:** `specs/core-platform.json` — `GET /storagezone/{id}/statistics`

Response contains storage used, files stored, requests served, bandwidth.

- [ ] API client (`bunny-api-core`): `GET /storagezone/{id}/statistics` — add `StorageZoneStatistics` type
- [ ] CLI: `hoppy storage-zone statistics --id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- [ ] Capture fixture via `hoppy --record=fixtures/core/ storage-zone statistics --id <id>`
- [ ] Wiremock + insta snapshot test
- [ ] Live E2E test: extend `live_storage_zone_lifecycle` to fetch stats after creation

### 3. DNS Zone Statistics

**OpenAPI ref:** `specs/core-platform.json` — `GET /dnszone/{id}/statistics`

Response contains queries resolved by type, NXDomain counts, etc.

- [ ] API client (`bunny-api-core`): `GET /dnszone/{id}/statistics` — add `DnsZoneStatistics` type
- [ ] CLI: `hoppy dns zone statistics --id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test
- [ ] Live E2E test: extend `live_dns_zone_lifecycle` to fetch stats after zone creation

### 4. Pull Zone Statistics (3 endpoints)

**OpenAPI ref:** `specs/core-platform.json`

- [ ] API client (`bunny-api-core`): `GET /pullzone/{id}/optimizer/statistics` — add `OptimizerStatistics` type
- [ ] API client: `GET /pullzone/{id}/originshield/queuestatistics` — add `OriginShieldStatistics` type
- [ ] API client: `GET /pullzone/{id}/safehop/statistics` — add `SafeHopStatistics` type
- [ ] CLI: `hoppy pull-zone statistics --id <id> --type optimizer|origin-shield|safehop`
- [ ] Capture 3 fixtures via `--record`
- [ ] Wiremock + insta snapshot tests (one per endpoint)
- [ ] Live E2E test: extend `live_pull_zone_lifecycle` to fetch each stat type

### 5. Stream Library Statistics

**OpenAPI ref:** `specs/stream.json` — `GET /library/{libraryId}/statistics`

Response contains views and watch time, with optional `dateFrom`/`dateTo`/`hourly` params.

- [ ] API client (`bunny-api-stream`): `GET /library/{libraryId}/statistics` — add `VideoLibraryStatistics` type
- [ ] CLI: `hoppy stream library statistics --library-id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD> --hourly`
- [ ] Capture fixture via `--record`
- [ ] Wiremock + insta snapshot test
- [ ] Live E2E test: extend `live_stream_library_lifecycle` to fetch stats after creation

---

## Implementation Order

1. **Shield detailed metrics** — fixtures exist, lowest effort, unblocks deferred iter 12 work
2. **Storage zone stats** — simple response shape
3. **DNS zone stats** — similar pattern
4. **Pull zone stats** — three endpoints but same pattern
5. **Stream library stats** — depends on stream API key auth

## Implementation Notes

- All stats endpoints are GET-only, no side effects — safe to test against live API
- Response types may have many optional numeric fields — check the OpenAPI spec for nullable fields
- Table output should show key metrics; full JSON available via `--format json`
- Query params `dateFrom`/`dateTo` use `YYYY-MM-DD` format across all services
- Follow [[adding-a-feature]] checklist for each endpoint

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| Shield detailed metrics | 5 | 5 | Small (fixtures exist) |
| Storage zone stats | 1 | 1 | Small |
| DNS zone stats | 1 | 1 | Small |
| Pull zone stats | 3 | 1 (with --type flag) | Small |
| Stream library stats | 1 | 1 | Small |
| **Total** | **11** | **9** | **Medium** |

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-12-api-coverage]] — previous iteration (deferred shield metrics)
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-api-client-patterns]] — client patterns
