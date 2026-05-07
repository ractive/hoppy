---
title: Iteration 18 — Shield Advanced Features
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - shield
  - security
  - waf
status: completed
branch: iter-18/shield-advanced
---

# Iteration 18 — Shield Advanced Features

**Goal:** Add API Guardian, upload scanning, event logs, and WAF triggered rule review to the Shield service. These are the remaining Shield API gaps after iter 5 (base) and iter 13 (metrics).

## Context

Shield has several advanced features beyond the base WAF/rate-limit/access-list/bot-detection that was implemented in iter 5:

1. **API Guardian** — protects API endpoints by enforcing expected schema/behavior. Relatively new Bunny feature.
2. **Upload Scanning** — malware scanning on uploaded files.
3. **Event Logs** — historical log of security events (blocked requests, WAF triggers, rate limit hits).
4. **WAF Triggered Rule Review** — review which WAF rules are being triggered, with optional AI recommendations for tuning.

**OpenAPI ref:** `specs/shield.json`

## Scope

### 1. API Guardian

- [x] API client (`bunny-api-shield`): `GET /shield/shield-zone/{shieldZoneId}/api-guardian` — get API Guardian config
- [x] API client: `POST /shield/shield-zone/{shieldZoneId}/api-guardian` — create/configure API Guardian
- [x] API client: `PATCH /shield/shield-zone/{shieldZoneId}/api-guardian` — update API Guardian config
- [x] API client: `PATCH /shield/shield-zone/{shieldZoneId}/api-guardian/endpoint/{endpointId}` — update specific endpoint config
- [x] Add `ApiGuardianConfig`, `ApiGuardianEndpoint` types — check spec for full shape
- [x] CLI: `hoppy shield api-guardian get --shield-zone-id <id>`
- [x] CLI: `hoppy shield api-guardian upload --shield-zone-id <id> --spec-file <path>` (POST)
- [x] CLI: `hoppy shield api-guardian update --shield-zone-id <id> --spec-file <path>` (PATCH)
- [x] CLI: `hoppy shield api-guardian update-endpoint --shield-zone-id <id> --endpoint-id <id> [flags]`
- [x] Wiremock + fixture tests (4 tests covering all 4 endpoints)
- [ ] Capture fixtures via `--record` (skipped — no live env)
- [ ] Live E2E test (skipped — no live env)

### 2. Upload Scanning

- [x] API client: `GET /shield/shield-zone/{shieldZoneId}/upload-scanning` — get upload scanning config
- [x] API client: `PATCH /shield/shield-zone/{shieldZoneId}/upload-scanning` — update upload scanning config
- [x] Add `UploadScanningConfigurationState`, `UploadScanningScannerMode` types
- [x] CLI: `hoppy shield upload-scanning get --shield-zone-id <id>`
- [x] CLI: `hoppy shield upload-scanning update --shield-zone-id <id> [--enabled] [--antivirus-mode] [--csam-mode]`
- [x] Wiremock + fixture tests
- [ ] Capture fixtures via `--record` (skipped — no live env)
- [ ] Live E2E test (skipped — no live env)

### 3. Event Logs

- [x] API client: `GET /shield/event-logs/{shieldZoneId}/{date}/{continuationToken}` — paginated event log retrieval
- [x] Add `EventLog`, `EventLogLabels`, `WafLoggingResponse` types
- [x] CLI: `hoppy shield event-logs --shield-zone-id <id> --date <MM-dd-yyyy>` — list events for a date
- [x] Support pagination via `--continuation-token` flag and auto-paginate with `--all`
- [x] Wiremock + fixture test
- [ ] Capture fixture via `--record` (skipped — no live env)
- [ ] Live E2E test (skipped — no live env)

### 4. WAF Triggered Rule Review

- [x] API client: `GET /shield/waf/rules/review-triggered/{shieldZoneId}` — list triggered WAF rules
- [x] API client: `POST /shield/waf/rules/review-triggered/{shieldZoneId}` — acknowledge/review triggered rules
- [x] API client: `GET /shield/waf/rules/review-triggered/ai-recommendation/{shieldZoneId}/{ruleId}` — get AI recommendation for rule tuning
- [x] Add `TriggeredRuleItem`, `GetTriggeredRulesResponse`, `ReviewActionType`, `UpdateReviewTriggeredRuleRequest/Response`, `TriggeredRuleRecommendationResponse` types
- [x] CLI: `hoppy shield waf triggered-rules --shield-zone-id <id>` — list triggered rules
- [x] CLI: `hoppy shield waf review-triggered-rule --shield-zone-id <id> --rule-id <id> --action <n>`
- [x] CLI: `hoppy shield waf recommend-triggered-rule --shield-zone-id <id> --rule-id <id>`
- [x] Wiremock + fixture tests (3 tests)
- [ ] Capture fixtures via `--record` (skipped — no live env)
- [ ] Live E2E test (skipped — no live env)

### 5. Supplementary Endpoints (Low Priority)

- [x] API client: `GET /shield/waf/rules/plan-segmentation` — WAF rule plan limits
- [x] API client: `GET /shield/waf/engine-config` — WAF engine configuration
- [x] API client: `GET /shield/ddos/enums` — DDoS configuration enums
- [x] API client: `GET /shield/shield-zones/pullzone-mapping` — mapping of shield zones to pull zones
- [x] API client: `GET /shield/promo/state` — promotional state
- [x] API client: `GET /shield/shield-zone/{shieldZoneId}/access-lists/enums` — access list enum values
- [x] CLI: `hoppy shield waf plan-segmentation`, `hoppy shield waf engine-config`, `hoppy shield pullzone-mapping`
- [x] Wiremock tests for all 6 supplementary endpoints

---

## Implementation Order

1. **Upload Scanning** — smallest scope (2 endpoints), simple get/update pattern
2. **Event Logs** — single read endpoint, most useful for debugging
3. **WAF Triggered Rules** — 3 endpoints, moderate complexity
4. **API Guardian** — 4 endpoints, most complex (new resource type)
5. **Supplementary endpoints** — if time permits

## Implementation Notes

- API Guardian is a newer Bunny feature — the spec may be less stable. Verify response shapes against live API carefully.
- Event logs use date-based pagination with continuation tokens — different from the `page`/`per_page` pattern used elsewhere. The CLI should handle this gracefully (show first page by default, offer `--continuation-token` for next pages).
- WAF AI recommendations may not always be available — handle 404/empty gracefully.
- The supplementary enum endpoints are primarily useful for building dynamic CLI help or validation — lower priority but nice-to-have.
- Check `specs/shield.json` carefully for each endpoint's full request/response types before implementing.

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| API Guardian | 4 | 4 | Medium |
| Upload Scanning | 2 | 2 | Small |
| Event Logs | 1 | 1 | Small |
| WAF Triggered Rules | 3 | 3 | Small-Medium |
| Supplementary | 6 | 2-3 | Small |
| **Total** | **16** | **12-13** | **Medium-Large** |

If the iteration is too large, split: Upload Scanning + Event Logs + WAF Triggered Rules in 18a, API Guardian + Supplementary in 18b.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-5-shield]] — base Shield implementation
- [[iterations/iteration-13-statistics]] — Shield metrics (iter 13)
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-api-client-patterns]] — client patterns
