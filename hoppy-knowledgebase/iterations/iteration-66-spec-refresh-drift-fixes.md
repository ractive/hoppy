---
title: Iter-66 — spec refresh & drift fixes
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - drift
  - shield
  - database
  - storage
  - stream
status: planned
branch: iter-66/spec-refresh-drift-fixes
priority: 0
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/database
  - research/api-coverage-2026-07/shield
  - research/api-coverage-2026-07/storage
  - research/api-coverage-2026-07/stream
  - research/api-coverage-2026-07/containers
---

# Iter-66 — spec refresh & drift fixes

## Why

Section 3 of [[research/api-coverage-gap-analysis-2026-07]] lists seven
places where hoppy disagrees with the current API — correctness bugs, not
gaps. Fix these before any feature iteration. The refreshed specs already
sit uncommitted in the working tree (4 modified + 3 new files).

## Scope

### 1. Commit refreshed specs

- [ ] Commit the 4 modified specs: `specs/core-platform.json`,
  `specs/shield.json`, `specs/storage.json`, `specs/stream.json`
- [ ] Commit the 3 new specs: `specs/magic-containers.json`,
  `specs/logging.json`, `specs/origin-errors.json`

### 2. `db fork` payload drift

- [ ] Spec body is `{slug, date}` (point-in-time fork); client sends
  `{slug, group}` — add required `date` to `ForkDatabasePayload`
  (`crates/bunny-net-api/src/database/types.rs:225`) + a `--date` flag
- [ ] Live-verify whether the non-spec `group` field is still accepted;
  drop it or document the drift in the KB

### 3. Shield API Guardian rework

- [ ] Retarget `shield api-guardian upload` from the removed
  `POST /shield/shield-zone/{shieldZoneId}/api-guardian` to
  `POST .../api-guardian/spec`; `update` to `PATCH .../api-guardian/spec`
- [ ] New command for `GET .../api-guardian/enums`

### 4. db optimal endpoints

- [ ] Send the spec-required `cdn_server_token` query param in
  `get_optimal` and `get_optimal_single` (`database/client.rs:162-172`);
  add `--cdn-server-token` to `db config optimal`
- [ ] Un-hide and un-stub `db config optimal-single`
  (`commands/database.rs:848-854`) once the param unbreaks the HTTP 400

### 5. Storage download streaming

- [ ] `download_file` buffers the whole body via `response.bytes()`
  (`storage/client.rs:153`); switch to `bytes_stream()` written
  incrementally to disk per the project streaming rule

### 6. Dogfood verifications

- [ ] `stream caption add` sends raw SRT (`stream.rs:1060`); docs say
  `captionsFile` is base64 — verify live, fix encoding if confirmed
- [ ] Regional storage hosts: client builds `{region}.bunnycdn.com`
  (`storage/client.rs:83`); docs list `{region}.storage.bunnycdn.com` —
  verify a non-default `--region` live, fix the host template if confirmed

### 7. Magic Containers KB notes refresh

- [ ] Add the 3 spec-only endpoints missing from `api/magic-containers/`
  notes: `GET /apps/{appId}/summary`, `GET /nodes/plain`,
  `POST /registries/image-config`

## Out of scope

- New endpoint/flag coverage — iters 67–77
- Shield `--ddos-*` / `blockVpn`/`blockTor` reverse drift —
  [[iteration-72-shield-new-surface]]

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated for all changed commands/flags
- [ ] `hyalo lint` clean on touched knowledgebase files
