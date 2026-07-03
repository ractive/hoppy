---
title: Iter-68 — updates that update (shield, dns, db)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - shield
  - dns
  - database
status: planned
branch: iter-68/updates-that-update
priority: 1
depends-on: iter-66/spec-refresh-drift-fixes
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/shield
  - research/api-coverage-2026-07/dns
  - research/api-coverage-2026-07/database
---

# Iter-68 — updates that update

## Why

Per [[research/api-coverage-gap-analysis-2026-07]] §4, several update
commands can't actually update: shield `waf update-rule` / `rate-limit
update` only rename (delete + recreate is the only way to change a rule),
`dns record update` is lossy for SRV/CAA, and `db group update` /
`db v2 update` don't exist at all.

## Scope

### 1. Shield `waf update-rule` full field set

- [ ] Expose `ruleDescription` and all `ruleConfiguration` fields
  (`actionType`, `variableTypes`, `operatorType`, `severityType`,
  `transformationTypes`, `value`, `chainedRuleConditions`) on
  `PATCH /shield/waf/custom-rule/{id}` — the read-modify-write plumbing
  already exists (`commands/shield.rs:809-824`)

### 2. Shield `rate-limit update` full field set

- [ ] Same as above plus `requestCount`, `counterKeyType`, `timeframe`,
  `blockTime` on `PATCH /shield/rate-limit/{id}`

### 3. `--config-json` escape hatch (create AND update)

- [ ] `--config-json <file>` on `waf add-rule`, `waf update-rule`,
  `rate-limit create`, `rate-limit update` for the nested
  `variableTypes` / `transformationTypes` / `chainedRuleConditions`
  (file-input precedent: `api-guardian --spec-file`)
- [ ] Expose `--description` on both create commands (currently
  hardcoded `""`)

### 4. `dns record update` parity

- [ ] Add `--port` / `--flags` / `--tag` (present on `add`, missing from
  `UpdateDnsRecord`, `core/types.rs:3149-3164`) — unblocks SRV/CAA updates
- [ ] Stop forcing `--type` / `--value` re-specification: read-modify-write
  merge from `GET /dnszone/{id}` so partial updates aren't lossy
- [ ] `--disabled <true|false>` on both `record add` and `record update`
  (spec `Disabled` prop; list view already shows the column)

### 5. `db group update`

- [ ] New command for `PATCH /v1/groups/{group_id}`; extend the client
  payload with `primary_regions` / `replicas_regions` (only
  `display_name` is modelled today)

### 6. `db v2 update`

- [ ] New command for `PATCH /v2/databases/{db_id}`; fix the non-spec
  client payload: `UpdateDatabaseV2Payload.name`
  (`database/types.rs:338-341`) has no spec counterpart — remove or
  live-verify; add the spec's `primary_regions` / `replicas_regions`

## Out of scope

- Shield pagination and `metrics detailed` time range —
  [[iteration-69-filters-pagination-sweep]]
- DNS smart-routing/linked-record fields —
  [[iteration-71-dns-completeness]]

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated for all new commands/flags
- [ ] `hyalo lint` clean on touched knowledgebase files
