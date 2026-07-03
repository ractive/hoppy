---
title: "Gap report: Bunny Shield"
type: research
date: 2026-07-03
status: active
origin: api-coverage deep dive 2026-07-03 (agent-generated, verified against specs + CLI source)
tags:
  - api-coverage
  - gap-analysis
  - shield
  - waf
---

# Shield gap report

Domain: Bunny Shield API (WAF, rate limiting, bot detection, DDoS, access lists, API Guardian, upload scanning, metrics, event logs).
Sources: spec inventory (50 ops), `hoppy shield` clap help dump, `crates/hoppy-cli/src/commands/shield.rs`, `crates/bunny-net-api/src/shield/{client,types}.rs`.

## 1. Endpoint coverage

| # | METHOD path | CLI command | Status | Notes |
|---|---|---|---|---|
| 1 | GET /shield/ddos/enums | — | **missing** | Client method `get_ddos_enums()` exists (client.rs:980) but is not wired to any CLI command |
| 2 | GET /shield/event-logs/{shieldZoneId}/{date}/{continuationToken} | `shield event-logs` | covered | Continuation token optional (empty = first page); `--all` auto-paginates; accepts ISO or legacy US date |
| 3 | GET /shield/metrics/overview/{shieldZoneId} | `shield metrics overview` | covered | |
| 4 | GET /shield/metrics/overview/{shieldZoneId}/detailed | `shield metrics detailed` | **partial** | Query params `StartDate`, `EndDate`, `Resolution` (enum 0–6) not exposed; client method takes no query args either |
| 5 | GET /shield/metrics/rate-limit/{id} | `shield metrics rate-limit` | covered | |
| 6 | GET /shield/metrics/rate-limits/{shieldZoneId} | `shield metrics rate-limits` | covered | |
| 7 | GET /shield/metrics/shield-zone/{shieldZoneId}/bot-detection | `shield metrics bot-detection` | covered | |
| 8 | GET /shield/metrics/shield-zone/{shieldZoneId}/upload-scanning | `shield metrics upload-scanning` | covered | |
| 9 | GET /shield/metrics/shield-zone/{shieldZoneId}/waf-rule/{ruleId} | `shield metrics waf-rule` | covered | `--id` + `--rule-id` |
| 10 | GET /shield/promo/state | — | **missing** | Client method `get_promo_state()` exists (client.rs:1007, returns raw `serde_json::Value`) but no CLI command |
| 11 | POST /shield/rate-limit | `shield rate-limit create` | **partial** | See flag gaps §2 |
| 12 | GET /shield/rate-limit/{id} | `shield rate-limit get` | covered | |
| 13 | DELETE /shield/rate-limit/{id} | `shield rate-limit delete` | covered | Confirmation prompt + `-y` |
| 14 | PATCH /shield/rate-limit/{id} | `shield rate-limit update` | **partial** | Only `--name` mutable; read-modify-write preserves the rest |
| 15 | GET /shield/rate-limits/{shieldZoneId} | `shield rate-limit list` | **partial** | `page`/`perPage` query params not exposed |
| 16 | POST /shield/shield-zone | `shield zone create` | **partial** | Only `--pull-zone-id`; client hardcodes `shield_zone: None`, so the entire `shieldZone` config object cannot be set at create time |
| 17 | PATCH /shield/shield-zone | `shield zone update` | **partial** | 6 flags exposed; many body props missing (§2) |
| 18 | GET /shield/shield-zone/get-by-pullzone/{pullZoneId} | `shield zone get-by-pullzone` | covered | |
| 19 | GET /shield/shield-zone/{shieldZoneId} | `shield zone get` | covered | |
| 20 | GET /shield/shield-zone/{shieldZoneId}/access-lists | `shield access-list list` | covered | Merges managed + custom lists into one table |
| 21 | POST /shield/shield-zone/{shieldZoneId}/access-lists | `shield access-list create` | **partial** | `description`, `checksum` not exposed (hardcoded `None`) |
| 22 | PATCH /shield/shield-zone/{shieldZoneId}/access-lists/configurations/{id} | `shield access-list update-config` | covered | `--is-enabled`, `--action` |
| 23 | GET /shield/shield-zone/{shieldZoneId}/access-lists/enums | — | **missing** | Client method `get_access_list_enums()` exists (client.rs:1031) but no CLI command |
| 24 | GET /shield/shield-zone/{shieldZoneId}/access-lists/{id} | `shield access-list get` | covered | |
| 25 | DELETE /shield/shield-zone/{shieldZoneId}/access-lists/{id} | `shield access-list delete` | covered | Confirmation prompt + `-y` |
| 26 | PATCH /shield/shield-zone/{shieldZoneId}/access-lists/{id} | `shield access-list update` | **partial** | `checksum` not exposed (hardcoded `None`) |
| 27 | GET /shield/shield-zone/{shieldZoneId}/api-guardian | `shield api-guardian get` | covered | |
| 28 | POST /shield/shield-zone/{shieldZoneId}/api-guardian | `shield api-guardian upload` | covered | `content` supplied via `--spec-file` (file-input pattern); `--enforce-authorization` |
| 29 | PATCH /shield/shield-zone/{shieldZoneId}/api-guardian | `shield api-guardian update` | covered | Same `--spec-file` pattern |
| 30 | PATCH /shield/shield-zone/{shieldZoneId}/api-guardian/endpoint/{endpointId} | `shield api-guardian update-endpoint` | covered | All 4 body props exposed |
| 31 | GET /shield/shield-zone/{shieldZoneId}/bot-detection | `shield bot-detection get` | covered | |
| 32 | PATCH /shield/shield-zone/{shieldZoneId}/bot-detection | `shield bot-detection update` | covered | All body props (executionMode, requestIntegrity.sensitivity, ipAddress.sensitivity, browserFingerprint.{sensitivity,aggression,complexEnabled}) flattened into flags |
| 33 | GET /shield/shield-zone/{shieldZoneId}/upload-scanning | `shield upload-scanning get` | covered | |
| 34 | PATCH /shield/shield-zone/{shieldZoneId}/upload-scanning | `shield upload-scanning update` | covered | `--enabled`, `--csam-mode`, `--antivirus-mode` |
| 35 | GET /shield/shield-zones | `shield zone list` | **partial** | `page`/`perPage` not exposed; JSON envelope does report `has_more_items` but there is no way to fetch page 2 |
| 36 | GET /shield/shield-zones/pullzone-mapping | `shield pullzone-mapping` | covered | |
| 37 | POST /shield/waf/custom-rule | `shield waf add-rule` | **partial** | See flag gaps §2 |
| 38 | GET /shield/waf/custom-rule/{id} | `shield waf get-rule` | covered | |
| 39 | PUT /shield/waf/custom-rule/{id} | — | **missing** | Neither client nor CLI issue PUT; functionally redundant — the PATCH variant (#41) is used instead. Low value |
| 40 | DELETE /shield/waf/custom-rule/{id} | `shield waf delete-rule` | covered | Confirmation prompt + `-y` |
| 41 | PATCH /shield/waf/custom-rule/{id} | `shield waf update-rule` | **partial** | Only `--name` mutable; read-modify-write preserves the rest |
| 42 | GET /shield/waf/custom-rules/{shieldZoneId} | `shield waf list-rules` | **partial** | `page`/`perPage` not exposed |
| 43 | GET /shield/waf/engine-config | `shield waf engine-config` | covered | |
| 44 | GET /shield/waf/enums | — | **missing** | No client method, no CLI command. (Note: client's `get_ddos_enums()` reuses the `GetWafEnumsResponse` type but hits `/shield/ddos/enums`, not this path) |
| 45 | GET /shield/waf/profiles | `shield waf profiles` | covered | |
| 46 | GET /shield/waf/rules/plan-segmentation | `shield waf plan-segmentation` | covered | |
| 47 | GET /shield/waf/rules/review-triggered/ai-recommendation/{shieldZoneId}/{ruleId} | `shield waf recommend-triggered-rule` | covered | |
| 48 | GET /shield/waf/rules/review-triggered/{shieldZoneId} | `shield waf triggered-rules` | covered | |
| 49 | POST /shield/waf/rules/review-triggered/{shieldZoneId} | `shield waf review-triggered-rule` | covered | `--rule-id`, `--action` (0/1/2) |
| 50 | GET /shield/waf/rules/{shieldZoneId} | — | **missing** | No client method, no CLI command. This is the listing of *managed/available* WAF rules for a zone — distinct from `custom-rules` (#42). Needed to know valid rule IDs for `wafDisabledRules`/`wafLogOnlyRules` and `metrics waf-rule` |

Sub-resource map (task 3): custom WAF rules CRUD = #37/38/40/41/42 (all present, PUT #39 skipped); rate-limit rules CRUD = #11–15 (all present); managed WAF rules listing = #50 (missing); review-triggered rules = #47–49 (all present); event logs = #2; metrics/statistics = #3–9 (7 endpoints, all present); DDoS enums = #1 (client-only); plan/quota = #46 plan-segmentation (present), #10 promo state (client-only). Access lists = #20–26 (enums #23 client-only); API Guardian = #27–30 (complete); upload scanning = #33/34 (complete); bot detection = #31/32 (complete).

## 2. Flag-level gaps per command

### `shield zone create` (POST /shield/shield-zone)

- `pullZoneId` → `--pull-zone-id` ✓
- `shieldZone` object → **MISSING entirely.** `create_shield_zone()` (client.rs:253) hardcodes `shield_zone: None`, so none of the 11 settable props (premiumPlan, planType, learningMode, learningModeUntil, wafEnabled, wafExecutionMode, wafDisabledRules, wafLogOnlyRules, wafRequestHeaderLoggingEnabled, wafRequestIgnoredHeaders, wafRealtimeThreatIntelligenceEnabled) can be seeded at creation. Users must create-then-update.

### `shield zone update` (PATCH /shield/shield-zone) — grouped by feature area

- **WAF settings**: `wafEnabled` → `--waf-enabled` ✓; `wafExecutionMode` → `--waf-execution-mode` ✓. MISSING: `wafDisabledRules`, `wafLogOnlyRules`, `wafRequestHeaderLoggingEnabled`, `wafRequestIgnoredHeaders`, `wafRealtimeThreatIntelligenceEnabled`. All five exist on `ShieldZoneRequest` in types.rs (lines 721–729) — only CLI flags are missing.
- **WAF engine extras (in types, not in spec inventory)**: `wafProfileId`, `wafRequestBodyLimitAction`, `wafResponseBodyLimitAction` — supported by the type, no flags. Notably there is no way to assign a WAF profile from `shield waf profiles` to a zone.
- **Learning mode**: `learningMode` → `--learning-mode` ✓. MISSING: `learningModeUntil` (type supports it).
- **Plan**: MISSING `planType` (type supports it) and `premiumPlan` (not even on the request type).
- **DDoS**: `--ddos-sensitivity`, `--ddos-execution-mode`, `--ddos-challenge-window` are exposed and map to `dDoSShieldSensitivity`/`dDoSExecutionMode`/`dDoSChallengeWindow` — these props are NOT in the spec's `shieldZone` body schema (see §3).
- Type-only extras with no flag and no spec entry: `blockVpn`, `blockTor`, `blockDatacentre`, `whitelabelResponsePages`.

### `shield waf add-rule` (POST /shield/waf/custom-rule)

- `shieldZoneId` → `--id` ✓; `ruleName` → `--name` ✓
- `ruleDescription` → **MISSING** (hardcoded to `""`, shield.rs:795)
- `ruleConfiguration.actionType` → `--action-type` ✓; `.operatorType` → `--operator-type` ✓; `.severityType` → `--severity-type` ✓; `.value` → `--value` ✓
- `ruleConfiguration.variableTypes` → **MISSING** (hardcoded `Default::default()`, shield.rs:785) — cannot choose which request variables (URI, headers, body, etc.) the rule inspects
- `ruleConfiguration.transformationTypes` → **MISSING** (hardcoded `vec![]`)
- `ruleConfiguration.chainedRuleConditions` → **MISSING** (hardcoded `None`) — multi-condition rules impossible
- No JSON-file input escape hatch for the nested config (the `--spec-file` pattern used by api-guardian is not offered here).

### `shield waf update-rule` (PATCH /shield/waf/custom-rule/{id})

- Only `--name` is mutable. The handler does a read-modify-write (fetches the rule, resends description + configuration unchanged — shield.rs:809–824, comment says the API requires all fields on PATCH). **MISSING**: `ruleDescription` and every `ruleConfiguration` field (actionType, variableTypes, operatorType, severityType, transformationTypes, value, chainedRuleConditions). A rule's matching logic/action cannot be changed after creation; delete + re-create is the only path.

### `shield waf list-rules` (GET /shield/waf/custom-rules/{shieldZoneId})

- `page`, `perPage` → **MISSING** (client sends neither).

### `shield rate-limit create` (POST /shield/rate-limit)

- `shieldZoneId` → `--id` ✓; `ruleName` → `--name` ✓
- `ruleDescription` → **MISSING** (hardcoded `""`)
- config: `actionType` ✓, `operatorType` ✓, `severityType` ✓, `value` ✓, `requestCount` ✓, `counterKeyType` ✓, `timeframe` ✓, `blockTime` ✓
- `variableTypes` → **MISSING** (hardcoded default); `transformationTypes` → **MISSING** (hardcoded `[]`); `chainedRuleConditions` → **MISSING** (hardcoded `None`). No JSON-file input alternative.

### `shield rate-limit update` (PATCH /shield/rate-limit/{id})

- Only `--name` mutable; same read-modify-write pattern as WAF update-rule. **MISSING**: `ruleDescription` + all 10 `ruleConfiguration` props (actionType, variableTypes, operatorType, severityType, transformationTypes, value, requestCount, counterKeyType, timeframe, blockTime, chainedRuleConditions). You cannot tune a limit's threshold or block time without recreating the rule.

### `shield rate-limit list` (GET /shield/rate-limits/{shieldZoneId})

- `page`, `perPage` → **MISSING**.

### `shield zone list` (GET /shield/shield-zones)

- `page`, `perPage` → **MISSING**.

### `shield metrics detailed` (GET /shield/metrics/overview/{id}/detailed)

- `StartDate`, `EndDate`, `Resolution` (enum 0–6) → **MISSING**; always returns the server default window.

### `shield access-list create` (POST .../access-lists)

- `name` ✓, `type` ✓, `content` ✓; `description` → **MISSING** (hardcoded `None`); `checksum` → **MISSING** (hardcoded `None`).

### `shield access-list update` (PATCH .../access-lists/{id})

- `name` ✓, `content` ✓; `checksum` → **MISSING** (hardcoded `None`).

### Fully covered bodies (no gaps)

- `access-list update-config`: `isEnabled` ✓, `action` ✓
- `api-guardian upload`/`update`: `content` ✓ (via `--spec-file`), `enforceAuthorisationValidation` ✓ (`--enforce-authorization`)
- `api-guardian update-endpoint`: all 4 props ✓
- `bot-detection update`: all nested props flattened to 6 flags ✓
- `upload-scanning update`: `isEnabled` ✓, `csamScanningMode` ✓, `antivirusScanningMode` ✓ (`shieldZoneId` from `--id`)
- `event-logs`: all 3 path params ✓ (token optional for first page)

## 3. CLI-only surface

- **`shield zone update --ddos-sensitivity / --ddos-execution-mode / --ddos-challenge-window`** — serialize to `dDoSShieldSensitivity`, `dDoSExecutionMode`, `dDoSChallengeWindow` inside the `shieldZone` body. The spec inventory's `shieldZone` schema (both POST and PATCH) lists only the 12 WAF/learning-mode props; these DDoS props have **no spec counterpart**. Same for the type-level (unflagged) fields `wafProfileId`, `wafRequestBodyLimitAction`, `wafResponseBodyLimitAction`, `blockVpn`, `blockTor`, `blockDatacentre`, `whitelabelResponsePages` on `ShieldZoneRequest`/`ShieldZoneResponse`. Likely dashboard-only fields the published OpenAPI spec omits — worth verifying against the live API rather than removing.
- **`shield event-logs --all`** — client-side auto-pagination convenience; no spec counterpart (spec paginates via the continuationToken path param only).
- **`shield event-logs --date` dual format** — accepts ISO 8601 and converts to the API's legacy US format; spec just says `string`.
- All client URLs in `client.rs` match spec inventory paths — no phantom endpoints.

## 4. Observations

1. **The update commands are name-only.** Both `waf update-rule` and `rate-limit update` already implement the read-modify-write plumbing the API requires ("PATCH requires all fields"), so adding the remaining config flags is mechanically cheap — the hard part is done.
2. **No JSON escape hatch for complex rule bodies.** `variableTypes`, `transformationTypes`, and `chainedRuleConditions` are hardcoded to defaults in both WAF and rate-limit create. The codebase already has a file-input precedent (`api-guardian --spec-file`); a `--config-json <file>` flag on add-rule/update-rule/create/update would close 4 partials at once and unlock chained (multi-condition) rules, which are currently impossible via CLI.
3. **Three endpoints are implemented in the API client but never surfaced**: `get_ddos_enums`, `get_promo_state`, `get_access_list_enums`. Exposing them is a wiring-only task. The enum endpoints matter because Shield commands take raw integers (`--action-type 1`, `--type 3`, …) and the help text hand-maintains the value legends — an `enums` command would make those discoverable and drift-proof.
4. **Two endpoints lack even client support**: `GET /shield/waf/enums` and `GET /shield/waf/rules/{shieldZoneId}`. The latter is the more important one: it lists the managed WAF rules active on a zone — the rule IDs users need for `wafDisabledRules`/`wafLogOnlyRules` (themselves missing flags) and for `shield metrics waf-rule --rule-id`.
5. **Pagination is silently first-page-only** on all three paginated Shield lists (zones, custom WAF rules, rate limits). `zone list --format json` at least reports `has_more_items`, but no command accepts `page`/`perPage`.
6. **`zone create` cannot seed configuration** (client hardcodes `shieldZone: None`), forcing a create-then-update dance and making atomic "create with WAF enabled" impossible.
7. **PUT /shield/waf/custom-rule/{id}** is a spec-level duplicate of the PATCH; skipping it loses nothing.
8. **Spec drift on DDoS**: the CLI's DDoS zone flags (a headline feature of `hoppy shield`) ride on props absent from the published spec (§3). If the spec is regenerated/validated against someday, these would be flagged; keep a note in the KB.

## Summary counts

- **Total operations**: 50
- **Covered**: 32
- **Partial**: 12 (metrics detailed; rate-limit create/update/list; zone create/update/list; access-list create/update; waf add-rule/update-rule/list-rules)
- **Missing**: 6 (ddos enums, promo state, access-list enums — client-ready but unwired; waf enums, waf managed-rules listing — no client support; PUT custom-rule — redundant duplicate of PATCH)
- **5 most impactful gaps**:
  1. `waf update-rule` / `rate-limit update` can only rename — rule conditions, actions, thresholds, and block times are immutable via CLI (delete + recreate is the only workaround).
  2. No way to author advanced rules: `variableTypes`, `transformationTypes`, `chainedRuleConditions` hardcoded to defaults on both create commands, and no `--config-json` file escape hatch.
  3. `metrics detailed` ignores `StartDate`/`EndDate`/`Resolution` — no time-range or resolution control over the flagship metrics endpoint.
  4. `GET /shield/waf/rules/{shieldZoneId}` absent (no client method) — cannot list the managed WAF rules a zone runs, which also blocks meaningful use of `wafDisabledRules`/`wafLogOnlyRules` (also missing) and rule-ID discovery for `metrics waf-rule`.
  5. `page`/`perPage` unexposed on all three paginated lists (zones, custom WAF rules, rate limits) — results silently truncate to the first page.
