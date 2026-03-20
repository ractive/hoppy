---
title: "Iteration 14 — Pull Zone Edge Rules"
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - cdn
  - edge-rules
status: planned
branch: iter-14/edge-rules
---

# Iteration 14 — Pull Zone Edge Rules

**Goal:** Add edge rule management to pull zones. Edge rules are the primary mechanism for custom request/response handling at the CDN edge — URL rewrites, redirects, header manipulation, origin overrides, caching overrides, etc.

## Context

Edge rules are a powerful CDN feature that lets users define conditional actions on requests. Each rule has:
- **Triggers**: conditions like URL pattern, country, request header, cookie, query string, etc.
- **Actions**: what to do when triggers match — redirect, override origin, set header, block request, force cache, bypass cache, etc.
- **Enabled state**: rules can be toggled without deletion.

The `PullZone` response already includes an `EdgeRules` array, so the types partially exist. The three endpoints manage the lifecycle.

**OpenAPI ref:** `specs/core-platform.json`

## Scope

### API Client (`bunny-api-core`)

Edge rule types from the OpenAPI spec:

```
EdgeRule:
  Guid, ActionType, ActionParameter1, ActionParameter2,
  Triggers (array of EdgeRuleTrigger), TriggerMatchingType (MatchAny/MatchAll/MatchNone),
  Description, Enabled

EdgeRuleTrigger:
  Type (enum: Url, RequestHeader, ResponseHeader, UrlExtension, CountryCode,
        RemoteIP, UrlQueryString, RandomChance, StatusCode, RequestMethod,
        CookieValue, CountryStateCode),
  PatternMatches (array of strings), PatternMatchingType (MatchAny/MatchAll/MatchNone),
  Parameter1

ActionType (enum):
  ForceSSL, Redirect, OriginUrl, OverrideCacheTime, BlockRequest,
  SetResponseHeader, SetRequestHeader, ForceDownload, DisableTokenAuthentication,
  EnableTokenAuthentication, OverrideCacheTimePublic, IgnoreQueryString,
  DisableOptimizer, ForceCompression, SetStatusCode, BypassPermaCache,
  SetNetworkRateLimit, SetConnectionLimit, SetRequestsPerSecondLimit
```

- [ ] Add `EdgeRule`, `EdgeRuleTrigger`, `TriggerMatchingType`, `ActionType`, `TriggerType` types to `bunny-api-core/src/types.rs`
- [ ] API client: `POST /pullzone/{pullZoneId}/edgerules/addOrUpdate` — creates new or updates existing rule (upsert by Guid)
- [ ] API client: `DELETE /pullzone/{pullZoneId}/edgerules/{edgeRuleId}` — delete a rule
- [ ] API client: `POST /pullzone/{pullZoneId}/edgerules/{edgeRuleId}/setEdgeRuleEnabled` — enable/disable

### CLI Commands

Edge rules are complex objects. The CLI should support:

- [ ] `hoppy pull-zone edge-rule list --id <pull-zone-id>` — list rules (from PullZone.EdgeRules, no separate list endpoint)
- [ ] `hoppy pull-zone edge-rule add --id <pull-zone-id> --description <text> --action-type <type> --action-param1 <val> --action-param2 <val> --trigger-matching-type <type> --trigger <type>:<pattern>[,<pattern>...]` (repeatable `--trigger` flag)
- [ ] `hoppy pull-zone edge-rule update --id <pull-zone-id> --rule-id <guid> [same flags as add]` — updates via same upsert endpoint
- [ ] `hoppy pull-zone edge-rule delete --id <pull-zone-id> --rule-id <guid>` — with confirmation
- [ ] `hoppy pull-zone edge-rule enable --id <pull-zone-id> --rule-id <guid> --enabled <true|false>`

### Testing

- [ ] Capture fixture: `pullzone_get.json` already contains `EdgeRules` array — verify it has data or capture a new one with rules
- [ ] Capture fixture: `pullzone_edgerule_add.json` via `--record`
- [ ] Wiremock + insta snapshot tests for add, delete, enable/disable
- [ ] Live E2E test: create PZ → add redirect rule → list (via get PZ) → verify rule → enable/disable → delete rule → delete PZ

---

## Implementation Notes

- The add/update endpoint is an upsert: if `Guid` is set it updates, if empty/null it creates. The CLI `add` command should omit Guid; `update` should require it.
- Edge rule `list` comes from `GET /pullzone/{id}` response `.EdgeRules` field — no separate list endpoint.
- Trigger patterns are arrays of strings (e.g., `["*.jpg", "*.png"]` for URL extension triggers).
- `--trigger` flag syntax suggestion: `--trigger url-extension:*.jpg,*.png --trigger country-code:US,DE` — parse `type:patterns` format in CLI.
- ActionType and TriggerType should be enums with human-readable CLI names mapped to API integer values.

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| Edge rule types | 0 (types only) | 0 | Small |
| Add/update rule | 1 | 2 (add + update) | Medium |
| Delete rule | 1 | 1 | Small |
| Enable/disable | 1 | 1 | Small |
| List (from PZ get) | 0 | 1 | Small |
| **Total** | **3** | **5** | **Medium** |

## Related

- [[development-roadmap]] — project roadmap
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-api-client-patterns]] — client patterns
- [[api/bunny-api-quirks]] — API quirks (upsert pattern)
