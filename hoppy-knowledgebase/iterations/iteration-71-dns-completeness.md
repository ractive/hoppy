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
status: planned
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

## Scope

### 1. Smart routing / monitoring / geo record fields

- [ ] Add `SmartRoutingType`, `MonitorType`, `LatencyZone`,
  `GeolocationLatitude`, `GeolocationLongitude` to `AddDnsRecord`
  (`core/types.rs:3059-3079`) and `UpdateDnsRecord`, with flags on
  `dns record add` / `update`

### 2. Linked-record fields

- [ ] Add `PullZoneId`, `ScriptId`, `Accelerated`, `AutoSslIssuance`
  to both record bodies + flags — makes `--type PullZone` / `Script`
  functional and removes the CNAME-workaround help text

### 3. Zone-level field

- [ ] `dns zone update --log-anonymization-type` — add
  `LogAnonymizationType` (enum) to `UpdateDnsZone`

### 4. New records endpoint

- [ ] `GET /dnszone/{zoneId}/records` (new in July spec) — back
  `dns record list` with the dedicated endpoint instead of projecting
  `Records` out of `GET /dnszone/{id}`

### 5. checkavailability endpoints (all three in one go)

- [ ] `dns zone check --domain` → `POST /dnszone/checkavailability`
- [ ] `pull-zone check --name` → `POST /pullzone/checkavailability`
- [ ] `storage-zone check --name` → `POST /storagezone/checkavailability`

### 6. New core-platform reference/statistics ops

- [ ] `pull-zone count` → `GET /pullzone/count`
- [ ] `storage-zone regions` → `GET /storagezone/regions`
- [ ] `storage-zone statistics --egress` (or `storage-zone egress`) →
  `GET /storagezone/{id}/statistics/egress`

## Out of scope

- `EnviromentalVariables` (sic) for Script records — niche, backlog
- `CertificateKeyType` on zone update and the `Records` array on zone
  create (`zone import` covers bulk-load)
- External-DNS certificate flow — [[iteration-74-pull-zone-body-completeness]]

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated (drop the PullZone/Script CNAME-workaround note)
- [ ] `hyalo lint` clean on touched knowledgebase files
