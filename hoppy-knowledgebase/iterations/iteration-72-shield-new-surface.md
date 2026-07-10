---
title: Iter-72 — shield new surface (bot categorization, custom pages, metrics)
type: iteration
date: 2026-07-03
tags:
  - iteration
  - api-coverage
  - shield
  - waf
status: in-progress
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

**Lesson carried from [[iteration-71-dns-completeness]]**: that iteration's
own carried lesson (diff the full spec field/param list against the CLI
arg surface before calling a section done, not just against what the
handler consumes) paid off — all 6 scope sections landed with no gaps
found in review. Apply the same discipline here, especially for §4
(`ShieldZoneRequest` already models `--waf-disabled-rules` /
`--waf-log-only-rules` — confirm no other modeled-but-unexposed fields
exist on that struct before marking §4 done) and §5's drift
investigation (verify against the live API, don't guess from the spec
diff alone).

## Scope

### 1. Bot categorization

- [x] Implement the 3 bot-categorization ops from the fresh
  `specs/shield.json` as `shield bot-categorization` subcommands
  (client methods + CLI)

### 2. Custom pages

- [x] Implement the 3 custom-page ops (custom block/challenge pages)
  from the fresh spec as `shield custom-page` subcommands

### 3. New metrics endpoints

- [x] `shield metrics overages` →
  `GET /shield/metrics/overages/{shieldZoneId}`
- [x] `shield api-guardian metrics` — the 2 new API Guardian metrics ops

### 4. Managed WAF rules listing

- [x] New client method + `shield waf managed-rules` for
  `GET /shield/waf/rules/{shieldZoneId}` — the rule-ID discovery path
- [x] Expose `--waf-disabled-rules` / `--waf-log-only-rules` on
  `shield zone update` (`ShieldZoneRequest` already models them,
  `shield/types.rs:721-729`)

### 5. Resolve undocumented drift

- [x] `--ddos-sensitivity` / `--ddos-execution-mode` /
  `--ddos-challenge-window` and type-only `blockVpn` / `blockTor` /
  `blockDatacentre` / `whitelabelResponsePages` have no counterpart in
  the fresh spec's `shieldZone` schema — verify against the live API,
  then either keep with a KB drift note or remove

### 6. Enums endpoints

- [x] Wire client-ready `get_ddos_enums` (`shield/client.rs:980`) and
  `get_access_list_enums` (`client.rs:1031`) to CLI commands
- [x] New client method + command for `GET /shield/waf/enums` — makes
  the raw-integer flags (`--action-type` etc.) discoverable

## Out of scope

- `shield zone create` seeding the `shieldZone` config object — backlog
- `PUT /shield/waf/custom-rule/{id}` — redundant duplicate of PATCH

## Acceptance

- [x] `cargo fmt` clean, `cargo clippy --workspace --all-targets -- -D warnings`
  clean, `cargo test --workspace --quiet` green
- [x] e2e tests cover every new/changed command (`tests/e2e/` pattern)
- [x] Help text updated; drift outcome documented in the KB
- [x] `hyalo lint` clean on touched knowledgebase files
