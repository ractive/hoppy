---
title: Iter-71 — DNS completeness + new core-platform endpoints
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - dns
  - pull-zone
  - storage-zone
status: completed
branch: iter-71/dns-completeness
priority: 2
depends-on: iter-66/spec-refresh-drift-fixes
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/dns
  - research/api-coverage-2026-07/storage
  - research/api-coverage-2026-07/pullzone-misc
---

# Iter-71 — DNS completeness

## Why

Per [[research/api-coverage-2026-07/dns]], bunny's headline smart-DNS
features (smart routing, monitoring, geo, linked records) are unusable
from the CLI — `--type PullZone/Script` is accepted but non-functional.
Also picks up the 6 new core-platform ops from the July spec refresh
([[research/api-coverage-gap-analysis-2026-07]] §1).

**Lesson carried from [[iteration-70-log-retrieval-services]]**: that
PR's initial pass wired a client-level params struct with every query
param from the spec, but the CLI arg surface only exposed a subset
(`--start`/`--end`, dropping `sort`/`status`/`search`/`download`) —
caught in review, not before. When adding flags for the field lists in
§1–3 below (`AddDnsRecord`/`UpdateDnsRecord`, `UpdateDnsZone`), diff
the new `#[arg(...)]` list against the spec's full field/param list
before calling a section done, not just against what the handler
function consumes.

## Scope

### 1. Smart routing / monitoring / geo record fields [1/1]

- [x] Add `SmartRoutingType`, `MonitorType`, `LatencyZone`,
  `GeolocationLatitude`, `GeolocationLongitude` to `AddDnsRecord`
  (`core/types.rs:3059-3079`) and `UpdateDnsRecord`, with flags on
  `dns record add` / `update`

### 2. Linked-record fields [1/1]

- [x] Add `PullZoneId`, `ScriptId`, `Accelerated`, `AutoSslIssuance`
  to both record bodies + flags — makes `--type PullZone` / `Script`
  functional and removes the CNAME-workaround help text

### 3. Zone-level field [1/1]

- [x] `dns zone update --log-anonymization-type` — add
  `LogAnonymizationType` (enum) to `UpdateDnsZone`

### 4. New records endpoint [1/1]

- [x] `GET /dnszone/{zoneId}/records` (new in July spec) — back
  `dns record list` with the dedicated endpoint instead of projecting
  `Records` out of `GET /dnszone/{id}`

### 5. checkavailability endpoints (all three in one go) [3/3]

- [x] `dns zone check --domain` → `POST /dnszone/checkavailability`
- [x] `pull-zone check --name` → `POST /pullzone/checkavailability`
- [x] `storage-zone check --name` → `POST /storagezone/checkavailability`

### 6. New core-platform reference/statistics ops [3/3]

- [x] `pull-zone count` → `GET /pullzone/count`
- [x] `storage-zone regions` → `GET /storagezone/regions`
- [x] `storage-zone statistics --egress` (or `storage-zone egress`) →
  `GET /storagezone/{id}/statistics/egress`

## Out of scope

- `EnviromentalVariables` (sic) for Script records — niche, backlog
- `CertificateKeyType` on zone update and the `Records` array on zone
  create (`zone import` covers bulk-load)
- External-DNS certificate flow — [[iteration-74-pull-zone-body-completeness]]

## Acceptance [4/4]

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [x] Help text updated (drop the PullZone/Script CNAME-workaround note)
- [x] `hyalo lint` clean on touched knowledgebase files
