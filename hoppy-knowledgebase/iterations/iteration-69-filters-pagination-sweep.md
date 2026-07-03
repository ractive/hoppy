---
title: Iter-69 — filters & pagination sweep
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - shield
  - script
  - stream
  - statistics
status: planned
branch: iter-69/filters-pagination-sweep
priority: 2
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/shield
  - research/api-coverage-2026-07/script
  - research/api-coverage-2026-07/pullzone-misc
  - research/api-coverage-2026-07/storage
  - research/api-coverage-2026-07/database
  - research/api-coverage-2026-07/stream
---

# Iter-69 — filters & pagination sweep

## Why

Per [[research/api-coverage-gap-analysis-2026-07]] §4, server-side
filters and pagination params are silently dropped across seven command
groups: shield lists always return page 1, `hoppy statistics` drops 9 of
13 params, `purge` can't control exact-path semantics, and `stream video
upload` drops all 10 per-upload encoding params.

## Scope

### 1. Shield pagination

- [ ] `--page` / `--per-page` on `shield zone list`
  (`GET /shield/shield-zones`), `shield waf list-rules`
  (`GET /shield/waf/custom-rules/{shieldZoneId}`), and
  `shield rate-limit list` (`GET /shield/rate-limits/{shieldZoneId}`)

### 2. Script filters

- [ ] `script list --type` (repeatable), `--integration-id`,
  `--include-linked-pullzones` (`GET /compute/script` query params)
- [ ] `script statistics --load-latest` (`loadLatest` query param)

### 3. `hoppy statistics` missing params

- [ ] Add the 9 missing `GET /statistics` query params:
  `--server-zone-id` plus the 8 `load*` selectors (`loadErrors`,
  `loadOriginResponseTimes`, `loadOriginTraffic`, `loadRequestsServed`,
  `loadBandwidthUsed`, `loadOriginShieldBandwidth`,
  `loadGeographicTrafficDistribution`, `loadUserBalanceHistory`);
  client method (`core/client.rs:731`) needs the params too

### 4. Purge

- [ ] `purge --exact-path` and `purge --async` (`POST /purge`;
  client hardcodes neither, `core/client.rs:185`)

### 5. Storage zone list

- [ ] `storage-zone list --include-deleted` (client supports it;
  handler passes `None` at `storage_zone.rs:74,108`)

### 6. DB versions windowing

- [ ] `db versions --older-than` / `--newer-than` (body fields hardcoded
  `None` at `commands/database.rs:429-430`)

### 7. Shield metrics time range

- [ ] `shield metrics detailed --start-date` / `--end-date` /
  `--resolution` (enum 0–6) — client method currently takes no query args

### 8. Stream per-upload params

- [ ] Add all 10 `PUT /library/{lib}/videos/{vid}` query params to
  `stream video upload`: `jitEnabled`, `enabledResolutions`,
  `enabledOutputCodecs`, `transcribeEnabled`, `transcribeLanguages`,
  `sourceLanguage`, `generateTitle`, `generateDescription`,
  `generateChapters`, `generateMoments` (client `upload_video()`
  accepts none today)

## Out of scope

- `stream collection list/get --include-thumbnails`, `pull-zone
  list/get --include-certificate` — cheap follow-ups, backlog if wanted

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated for all new flags
- [ ] `hyalo lint` clean on touched knowledgebase files
