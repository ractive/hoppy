---
title: "Iteration 3 — DNS"
type: iteration
date: 2026-03-18
tags:
  - iteration
  - dns
  - records
status: completed
branch: iter-3/dns
---

# Iteration 3 — DNS

**Goal:** Manage DNS zones and records.

- [x] DNS zone commands:
  - [x] `dns zone list|get|create|update|delete`
  - [x] Zone update flags: custom nameservers, SOA email, logging, IP anonymization
  - [x] Pagination and search support
- [x] DNS record commands:
  - [x] `dns record list --zone-id <id>`
  - [x] `dns record add --zone-id <id> --type <A|AAAA|CNAME|...> --name <name> --value <value> [--ttl <seconds>]`
  - [x] `dns record update --zone-id <id> --record-id <id> [options]`
  - [x] `dns record delete --zone-id <id> --record-id <id> [--yes]`
  - [x] All record types: A, AAAA, CNAME, TXT, MX, SRV, CAA, PTR, NS, SVCB, HTTPS, TLSA + bunny-specific (Redirect, Flatten, PullZone, Script)
  - [x] Record add supports priority, weight, port, flags, tag, comment
- [x] Confirmation prompts for destructive operations (--yes to skip)
- [x] 15 wiremock integration tests with fixture-based responses
- [ ] Import/export zone files — deferred (API supports export as BIND file, import endpoint exists but not documented well enough for safe implementation)

**Deliverable:** Full DNS management via CLI.

## Related

- [[development-roadmap]] — project roadmap
- [[iterations/iteration-2-storage]] — previous iteration
- [[api/bunny-api-quirks]] — DNS-specific API quirks
- [[decision-log]] — DNS record creation uses PUT, records embedded in zone response
