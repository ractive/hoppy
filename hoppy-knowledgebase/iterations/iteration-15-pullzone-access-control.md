---
title: Iteration 15 — Pull Zone Access Control
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - cdn
  - security
status: completed
branch: iter-15/pullzone-access-control
---

# Iteration 15 — Pull Zone Access Control

**Goal:** Add referrer allow/block lists and IP blocking to pull zones. These are core CDN security features for preventing hotlinking and blocking abusive traffic.

## Context

Pull zones support two types of access control lists managed via dedicated endpoints:

1. **Referrer control** — allow or block requests based on the `Referer` header. Used to prevent hotlinking.
2. **IP blocking** — block requests from specific IP addresses or CIDR ranges.

Each operation is a single POST call that adds/removes one entry. The current referrer/IP lists are visible in the `PullZone` GET response (`AllowedReferrers`, `BlockedReferrers`, `BlockedIps` arrays).

**OpenAPI ref:** `specs/core-platform.json`

## Scope

### API Client (`bunny-api-core`)

Six new endpoints, all POST with simple JSON bodies:

- [x] `POST /pullzone/{id}/addAllowedReferrer` — body: `{ "Hostname": "*.example.com" }`
- [x] `POST /pullzone/{id}/removeAllowedReferrer` — body: `{ "Hostname": "*.example.com" }`
- [x] `POST /pullzone/{id}/addBlockedReferrer` — body: `{ "Hostname": "*.example.com" }`
- [x] `POST /pullzone/{id}/removeBlockedReferrer` — body: `{ "Hostname": "*.example.com" }`
- [x] `POST /pullzone/{id}/addBlockedIp` — body: `{ "BlockedIp": "1.2.3.4" }`
- [x] `POST /pullzone/{id}/removeBlockedIp` — body: `{ "BlockedIp": "1.2.3.4" }`

Body shapes are taken from `specs/core-platform.json`: the referrer endpoints take a single required `Hostname` field (no `Value`), and the IP endpoints take a single required `BlockedIp` field.

### CLI Commands

- [x] `hoppy pull-zone referrer list --id <pull-zone-id>` — show allowed + blocked referrers (from PZ get response)
- [x] `hoppy pull-zone referrer allow --id <pull-zone-id> --value <pattern>` — add allowed referrer
- [x] `hoppy pull-zone referrer remove-allowed --id <pull-zone-id> --value <pattern>` — remove allowed referrer
- [x] `hoppy pull-zone referrer block --id <pull-zone-id> --value <pattern>` — add blocked referrer
- [x] `hoppy pull-zone referrer remove-blocked --id <pull-zone-id> --value <pattern>` — remove blocked referrer
- [x] `hoppy pull-zone ip list --id <pull-zone-id>` — show blocked IPs (from PZ get response)
- [x] `hoppy pull-zone ip block --id <pull-zone-id> --value <ip>` — block an IP
- [x] `hoppy pull-zone ip unblock --id <pull-zone-id> --value <ip>` — unblock an IP

### Testing

- [x] Capture fixtures via `--record` for each of the 6 endpoints (response is likely the updated PullZone or empty 204)
- [x] Wiremock + insta snapshot tests for all 6 API methods
- [x] Wiremock + insta snapshot tests for all 8 CLI commands
- [x] Live E2E test: create PZ → add allowed referrer → add blocked referrer → verify via get → remove both → add blocked IP → verify → unblock → delete PZ

---

## Implementation Notes

- The response for add/remove endpoints may be `204 No Content` or the updated `PullZone` — check the OpenAPI spec and verify with live API.
- If 204, the list commands need to do a `get_pull_zone()` call to show current state.
- Referrer patterns support wildcards (e.g., `*.example.com`).
- IP values can be single IPs or CIDR ranges — validate format in CLI if the API doesn't.
- The `PullZone` type already has `AllowedReferrers` and `BlockedReferrers` (Vec<String>) and `BlockedIps` (Vec<String>) fields — verify these are deserialized correctly, add if missing.

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| Referrer allow/block | 4 | 5 (list + 4 actions) | Small |
| IP block/unblock | 2 | 3 (list + 2 actions) | Small |
| **Total** | **6** | **8** | **Small** |

## Related

- [[development-roadmap]] — project roadmap
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-api-client-patterns]] — client patterns
