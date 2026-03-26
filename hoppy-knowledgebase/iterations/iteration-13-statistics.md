---
title: "Iteration 13 — Statistics & Analytics"
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - statistics
  - analytics
status: completed
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

- [x] API client (`bunny-api-shield`): `GET /shield/metrics/overview/{shieldZoneId}/detailed`
- [x] API client: `GET /shield/metrics/rate-limits/{shieldZoneId}` (zone-level)
- [x] API client: `GET /shield/metrics/rate-limit/{id}` (single rule)
- [x] API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/waf-rule/{ruleId}`
- [x] API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/bot-detection`
- [x] API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/upload-scanning`
- [x] CLI: `hoppy shield metrics detailed <shield-zone-id>`
- [x] CLI: `hoppy shield metrics rate-limits <shield-zone-id>`
- [x] CLI: `hoppy shield metrics rate-limit <id>` (single rule)
- [x] CLI: `hoppy shield metrics waf-rule <shield-zone-id> --rule-id <id>`
- [x] CLI: `hoppy shield metrics bot-detection <shield-zone-id>`
- [x] CLI: `hoppy shield metrics upload-scanning <shield-zone-id>`
- [x] Wiremock tests (fixtures for all 6 endpoints)
- [x] Created `metrics_upload_scanning.json` fixture
- [x] Created `metrics_rate_limit.json` fixture (single rule)
- [x] Live E2E test: extend existing `live_shield_lifecycle` to fetch all metrics after zone creation

### 2. Account-Level Statistics

**OpenAPI ref:** `specs/core-platform.json` — `GET /statistics`

Response type `StatisticsModel` — comprehensive account-wide CDN stats: bandwidth, cache hit rate, requests served, origin traffic, error charts, geo distribution. Accepts many optional params: `dateFrom`, `dateTo`, `pullZone`, `serverZoneId`, `hourly`, `loadErrors`, `loadOriginResponseTimes`, `loadOriginTraffic`, `loadRequestsServed`, `loadBandwidthUsed`, `loadOriginShieldBandwidth`, `loadGeographicTrafficDistribution`, `loadUserBalanceHistory`.

- [x] API client (`bunny-api-core`): `GET /statistics` — add `AccountStatistics` type
- [x] CLI: `hoppy statistics --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD> --pull-zone <id> --hourly`
- [x] Created fixture `account_statistics.json`
- [x] Wiremock test
- [x] Live E2E test: standalone test (no resource lifecycle needed)

### 3. Storage Zone Statistics

**OpenAPI ref:** `specs/core-platform.json` — `GET /storagezone/{id}/statistics`

Response contains storage used, files stored, requests served, bandwidth.

- [x] API client (`bunny-api-core`): `GET /storagezone/{id}/statistics` — add `StorageZoneStatistics` type
- [x] CLI: `hoppy storage-zone statistics --id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- [x] Created fixture `storagezone_statistics.json`
- [x] Wiremock test
- [x] Live E2E test: extend `live_storage_zone_lifecycle` to fetch stats after creation

### 4. DNS Zone Statistics

**OpenAPI ref:** `specs/core-platform.json` — `GET /dnszone/{id}/statistics`

Response contains queries resolved by type, NXDomain counts, etc.

- [x] API client (`bunny-api-core`): `GET /dnszone/{id}/statistics` — add `DnsZoneStatistics` type
- [x] CLI: `hoppy dns zone statistics --id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- [x] Created fixture `dnszone_statistics.json`
- [x] Wiremock test
- [x] Live E2E test: extend `live_dns_zone_lifecycle` to fetch stats after zone creation

### 5. Pull Zone Statistics (3 endpoints)

**OpenAPI ref:** `specs/core-platform.json`

- [x] API client (`bunny-api-core`): `GET /pullzone/{id}/optimizer/statistics` — add `OptimizerStatistics` type
- [x] API client: `GET /pullzone/{id}/originshield/queuestatistics` — add `OriginShieldQueueStatistics` type
- [x] API client: `GET /pullzone/{id}/safehop/statistics` — add `SafeHopStatistics` type
- [x] CLI: `hoppy pull-zone statistics --id <id> --type optimizer|origin-shield|safehop`
- [x] Created 3 fixtures
- [x] Wiremock tests (one per endpoint)
- [x] Live E2E test: extend `live_pull_zone_lifecycle` to fetch each stat type

### 6. Stream Library Statistics

**OpenAPI ref:** `specs/stream.json` — `GET /library/{libraryId}/statistics`

Response contains views and watch time, with optional `dateFrom`/`dateTo`/`hourly` params.

- [x] API client (`bunny-api-stream`): `GET /library/{libraryId}/statistics` — add `VideoStatistics` type
- [x] CLI: `hoppy stream library statistics --library-id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD> --hourly`
- [x] Created fixture `library_statistics.json`
- [x] Wiremock test
- [x] Live E2E test: extend `live_stream_library_lifecycle` to fetch stats after creation

### 7. Video Library DRM & Transcribing Statistics

**OpenAPI ref:** `specs/core-platform.json`

`VideoLibraryDrmStatisticsModel` — fields: `TotalLicensesIssued`, `LicensesIssuedChart`
`VideoLibraryTranscriptionStatisticsModel` — fields: `TotalTranscriptionSeconds`, `TranscriptionSecondsChart`

- [x] API client (`bunny-api-core`): `GET /videolibrary/{id}/drm/statistics` — add `VideoLibraryDrmStatistics` type
- [x] API client: `GET /videolibrary/{id}/transcribing/statistics` — add `VideoLibraryTranscribingStatistics` type
- [x] CLI: `hoppy video-library drm-statistics --id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- [x] CLI: `hoppy video-library transcribing-statistics --id <id> --date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- [x] Created 2 fixtures
- [x] Wiremock tests (one per endpoint)
- [x] Live E2E test: extend `live_stream_library_lifecycle` (video libraries are created there)

---

## Implementation Order

1. **Shield detailed metrics** — fixtures exist, lowest effort, unblocks deferred iter 12 work
2. **Account-level statistics** — high-value, standalone (no resource dependency)
3. **Storage zone stats** — simple response shape
4. **DNS zone stats** — similar pattern
5. **Pull zone stats** — three endpoints but same pattern
6. **Stream library stats** — depends on stream API key auth
7. **Video library DRM & transcribing stats** — small types, pairs with stream library work

## Implementation Notes

- All stats endpoints are GET-only, no side effects — safe to test against live API
- Response types may have many optional numeric fields — check the OpenAPI spec for nullable fields
- Table output should show key metrics; full JSON available via `--format json`
- Query params `dateFrom`/`dateTo` use `YYYY-MM-DD` format across all services
- The account-level `/statistics` endpoint has many boolean `load*` flags to selectively include chart data — expose these as CLI flags
- Video library DRM/transcribing stats are in `core-platform.json` (not stream spec) since they go through the core API, not the stream API
- Follow [[adding-a-feature]] checklist for each endpoint

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| Shield detailed metrics | 6 | 6 | Small (fixtures exist) |
| Account-level statistics | 1 | 1 | Small (many optional params) |
| Storage zone stats | 1 | 1 | Small |
| DNS zone stats | 1 | 1 | Small |
| Pull zone stats | 3 | 1 (with --type flag) | Small |
| Stream library stats | 1 | 1 | Small |
| Video library DRM & transcribing | 2 | 2 | Small |
| **Total** | **15** | **13** | **Medium-Large** |

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-12-api-coverage]] — previous iteration (deferred shield metrics)
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-api-client-patterns]] — client patterns
