---
title: Iter-72 — shield new surface (bot categorization, custom pages, metrics)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - shield
  - waf
status: planned
branch: iter-72/shield-new-surface
priority: 3
depends-on: iter-66/spec-refresh-drift-fixes
related:
  - research/api-coverage-gap-analysis-2026-07
  - research/api-coverage-2026-07/shield
---

# Iter-72 — shield new surface

## Why

The July shield spec grew from 50 to 61 ops
([[research/api-coverage-gap-analysis-2026-07]] §1): bot categorization,
custom block/challenge pages, overages and API Guardian metrics are all
new and all missing. Also resolves the reverse drift flagged in
[[research/api-coverage-2026-07/shield]] §3 and wires the three
client-ready enums endpoints.

## Scope

### 1. Bot categorization

- [ ] Implement the 3 bot-categorization ops from the fresh
  `specs/shield.json` as `shield bot-categorization` subcommands
  (client methods + CLI)

### 2. Custom pages

- [ ] Implement the 3 custom-page ops (custom block/challenge pages)
  from the fresh spec as `shield custom-page` subcommands

### 3. New metrics endpoints

- [ ] `shield metrics overages` →
  `GET /shield/metrics/overages/{shieldZoneId}`
- [ ] `shield api-guardian metrics` — the 2 new API Guardian metrics ops

### 4. Managed WAF rules listing

- [ ] New client method + `shield waf managed-rules` for
  `GET /shield/waf/rules/{shieldZoneId}` — the rule-ID discovery path
- [ ] Expose `--waf-disabled-rules` / `--waf-log-only-rules` on
  `shield zone update` (`ShieldZoneRequest` already models them,
  `shield/types.rs:721-729`)

### 5. Resolve undocumented drift

- [ ] `--ddos-sensitivity` / `--ddos-execution-mode` /
  `--ddos-challenge-window` and type-only `blockVpn` / `blockTor` /
  `blockDatacentre` / `whitelabelResponsePages` have no counterpart in
  the fresh spec's `shieldZone` schema — verify against the live API,
  then either keep with a KB drift note or remove

### 6. Enums endpoints

- [ ] Wire client-ready `get_ddos_enums` (`shield/client.rs:980`) and
  `get_access_list_enums` (`client.rs:1031`) to CLI commands
- [ ] New client method + command for `GET /shield/waf/enums` — makes
  the raw-integer flags (`--action-type` etc.) discoverable

## Out of scope

- `shield zone create` seeding the `shieldZone` config object — backlog
- `PUT /shield/waf/custom-rule/{id}` — redundant duplicate of PATCH

## Acceptance

- [ ] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [ ] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [ ] Help text updated; drift outcome documented in the KB
- [ ] `hyalo lint` clean on touched knowledgebase files
