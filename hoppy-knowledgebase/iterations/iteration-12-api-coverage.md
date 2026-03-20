---
title: "Iteration 12 — High-Value API Coverage Gaps"
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - cdn
  - dns
  - stream
  - shield
status: completed
branch: iter-12/api-coverage-gaps
---

# Iteration 12 — High-Value API Coverage Gaps

**Goal:** Close the highest-value gaps between the hoppy CLI and the bunny.net APIs — features that users reach for daily in CDN, DNS, streaming, and security workflows.

## Scope

Five topic areas, ordered by user impact:

### 1. URL Purge (`hoppy purge`)

Single most common CDN operation missing. Currently we only support zone-level purge.

- [x]API client: `POST /purge` with URL parameter
- [x]CLI: `hoppy purge <url>` — purge a single URL from CDN cache
- [x]Wiremock + snapshot test
- [x]Live E2E test (purge a URL on an existing pull zone)

### 2. Pull Zone Hostname & SSL Management

Core CDN setup — adding custom domains and enabling SSL is the first thing after creating a pull zone.

- [x]API client: `POST /pullzone/{id}/addHostname`
- [x]API client: `DELETE /pullzone/{id}/removeHostname`
- [x]API client: `GET /pullzone/loadFreeCertificate` (query param: hostname)
- [x]API client: `POST /pullzone/{id}/addCertificate`
- [x]API client: `DELETE /pullzone/{id}/removeCertificate`
- [x]API client: `POST /pullzone/{id}/setForceSSL`
- [x]CLI: `hoppy pull-zone hostname add <id> <hostname>`
- [x]CLI: `hoppy pull-zone hostname remove <id> <hostname>`
- [x]CLI: `hoppy pull-zone hostname load-free-cert <hostname>`
- [x]CLI: `hoppy pull-zone hostname add-cert <id> --certificate <file> --key <file>`
- [x]CLI: `hoppy pull-zone hostname remove-cert <id> <hostname>`
- [x]CLI: `hoppy pull-zone hostname force-ssl <id> <hostname> <on|off>`
- [x]Wiremock + snapshot tests for each subcommand
- [x]Live E2E test: add hostname → load free cert → force SSL → remove hostname

### 3. DNS Export & Import

Essential for migration workflows and backup/restore.

- [x]API client: `GET /dnszone/{id}/export`
- [x]API client: `POST /dnszone/{zoneId}/import` (multipart form body)
- [x]CLI: `hoppy dns zone export <id>` — prints BIND zone file to stdout
- [x]CLI: `hoppy dns zone import <id> <file>` — imports records from file (or stdin)
- [x]Wiremock + snapshot tests
- [x]Live E2E test: create zone → add records → export → verify format

### 4. Stream Video Captions

Key for accessibility and SEO — captions are a common video workflow.

- [x]API client: `POST /library/{libId}/videos/{videoId}/captions/{srclang}` (add caption)
- [x]API client: `DELETE /library/{libId}/videos/{videoId}/captions/{srclang}` (delete caption)
- [x]CLI: `hoppy stream video caption add <library-id> <video-id> <srclang> --file <srt-file>`
- [x]CLI: `hoppy stream video caption delete <library-id> <video-id> <srclang>`
- [x]Wiremock + snapshot tests
- [x]Live E2E test: upload video → add caption → delete caption → delete video

### 5. Shield Metrics

Observability for security features — without metrics the shield commands are fire-and-forget.

- [x]API client: `GET /shield/metrics/overview/{shieldZoneId}`
- [x]API client: `GET /shield/metrics/overview/{shieldZoneId}/detailed`
- [x]API client: `GET /shield/metrics/rate-limits/{shieldZoneId}`
- [x]API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/waf-rule/{ruleId}`
- [x]API client: `GET /shield/metrics/shield-zone/{shieldZoneId}/bot-detection`
- [x]CLI: `hoppy shield metrics <shield-zone-id>` — overview table
- [x]CLI: `hoppy shield metrics <shield-zone-id> --detailed` — full breakdown
- [x]CLI: `hoppy shield metrics rate-limits <shield-zone-id>`
- [x]CLI: `hoppy shield metrics waf-rule <shield-zone-id> <rule-id>`
- [x]CLI: `hoppy shield metrics bot-detection <shield-zone-id>`
- [x]Wiremock + snapshot tests
- [x]Live E2E test: create PZ → create SZ → fetch metrics → cleanup

---

## Implementation Order

1. **URL Purge** — smallest scope, immediate payoff, warms up the iteration
2. **Pull Zone Hostnames** — most API endpoints but straightforward CRUD pattern
3. **DNS Export/Import** — different payload pattern (BIND zone file, multipart form)
4. **Stream Captions** — small scope, depends on understanding the caption body format
5. **Shield Metrics** — read-only endpoints, just needs response type modeling

## Approach

Follow [[adding-a-feature]] checklist for each topic:
1. Research the endpoint (request/response shapes, quirks)
2. Add types + client method to `bunny-api-*` crate
3. Add wiremock unit test for the client
4. Wire CLI command with clap
5. Add CLI E2E snapshot test
6. Add live E2E test

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| URL Purge | 1 | 1 | Small |
| Pull Zone Hostnames | 6 | 6 | Medium |
| DNS Export/Import | 2 | 2 | Medium |
| Stream Captions | 2 | 2 | Small |
| Shield Metrics | 5 | 5 | Medium |
| **Total** | **16** | **16** | **Medium-Large** |

If the iteration is too large, topics 4 and 5 (captions + metrics) can be deferred to iter 13.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-9-gap-analysis]] — previous gap analysis iteration
- [[api/bunny-api-client-patterns]] — established client implementation patterns
- [[api/bunny-api-quirks]] — known API quirks to watch for
- [[adding-a-feature]] — implementation checklist
