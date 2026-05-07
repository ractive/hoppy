---
title: Iteration 17 — DNS Security & Record Scanning
type: iteration
date: 2026-03-20
tags:
  - iteration
  - api-coverage
  - dns
  - security
  - dnssec
status: completed
branch: iter-17/dns-security
---

# Iteration 17 — DNS Security & Record Scanning

**Goal:** Add DNSSEC management, wildcard certificate issuance, and DNS record scanning to the DNS service. These are important security and migration features.

## Context

DNSSEC prevents DNS spoofing by cryptographically signing zone records. Bunny.net manages the DNSSEC keys — the user just needs to enable/disable it and copy DS records to their registrar.

Record scanning auto-discovers existing DNS records for a domain (useful during migration). Certificate issuance provisions a wildcard TLS certificate for a DNS zone.

**OpenAPI ref:** `specs/core-platform.json`

## Scope

### 1. DNSSEC Management

- [x] API client (`bunny-api-core`): `POST /dnszone/{id}/dnssec` — enable DNSSEC. Check spec for request body (may be empty or contain configuration).
- [x] API client: `DELETE /dnszone/{id}/dnssec` — disable DNSSEC
- [x] Add response types — enabling DNSSEC likely returns DS record info (digest, key tag, algorithm) needed for registrar configuration
- [x] CLI: `hoppy dns zone dnssec enable --id <zone-id>` — enable and display DS record details
- [x] CLI: `hoppy dns zone dnssec disable --id <zone-id>` — with confirmation (disabling DNSSEC can break resolution if DS records are at registrar)
- [x] CLI: `hoppy dns zone dnssec status --id <zone-id>` — show current DNSSEC status (may come from `get_dns_zone()` response)
- [x] Capture fixtures via `--record`
- [x] Wiremock + insta snapshot tests
- [x] Live E2E test: create zone → enable DNSSEC → verify status → disable → verify → delete zone

### 2. Wildcard Certificate Issuance

- [x] API client: `POST /dnszone/{zoneId}/certificate/issue` — issue wildcard certificate for zone
- [x] Check spec for response — likely returns certificate status or async job ID
- [x] CLI: `hoppy dns zone issue-cert --id <zone-id>` — issue wildcard certificate
- [x] Capture fixture via `--record`
- [x] Wiremock + insta snapshot test
- [x] Live E2E test: include in DNSSEC lifecycle if feasible (requires zone to be properly delegated — may need to be a manual test or skipped in CI)

### 3. DNS Record Scanning

- [x] API client: `POST /dnszone/records/scan` — trigger background scan for existing DNS records. Check spec for request body (likely contains zone ID).
- [x] API client: `GET /dnszone/{zoneId}/records/scan` — get latest scan results
- [x] Add `DnsRecordScanResult` response type
- [x] CLI: `hoppy dns zone scan --id <zone-id>` — trigger scan and display results
- [x] Consider: should the CLI poll for results? The scan is async — trigger returns immediately, results come later. Two options:
  - (a) Separate `scan start` / `scan results` subcommands
  - (b) Single `scan` command that triggers and polls (with timeout)
  - Recommendation: option (a) for simplicity, matching the API shape
- [x] CLI: `hoppy dns zone scan start --id <zone-id>` — trigger scan
- [x] CLI: `hoppy dns zone scan results --id <zone-id>` — show latest scan results
- [x] Capture fixtures via `--record`
- [x] Wiremock + insta snapshot tests
- [x] Live E2E test: create zone → trigger scan → poll for results → verify → delete zone

---

## Implementation Order

1. **DNSSEC** — most impactful security feature
2. **Certificate issuance** — related to DNS zone security, small scope
3. **Record scanning** — useful for migrations, async pattern

## Implementation Notes

- DNSSEC enable/disable has real consequences — disabling DNSSEC on a zone with DS records at the registrar will break resolution. The CLI should show a clear warning in the confirmation prompt.
- Certificate issuance requires the zone to be properly delegated to Bunny nameservers — live tests may fail if using a test zone that isn't delegated. Consider handling this gracefully.
- Record scan is async — the POST returns immediately and results are available later via GET. The live E2E test should include a short poll loop with timeout.
- Check the `DnsZone` response type — it may already include DNSSEC fields (like `DnssecEnabled`, `DnssecStatus`) that we just haven't been displaying.

## Estimated Complexity

| Topic | New API methods | New CLI commands | Complexity |
|-------|----------------|-----------------|------------|
| DNSSEC enable/disable | 2 | 3 (enable, disable, status) | Small-Medium |
| Certificate issuance | 1 | 1 | Small |
| Record scanning | 2 | 2 (start, results) | Small-Medium |
| **Total** | **5** | **6** | **Medium** |

## Related

- [[development-roadmap]] — project roadmap
- [[adding-a-feature]] — implementation checklist
- [[api/bunny-api-client-patterns]] — client patterns
- [[api/bunny-api-quirks]] — known API quirks
