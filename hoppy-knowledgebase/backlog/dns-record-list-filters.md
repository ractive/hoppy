---
title: DNS record list search/type filters and zone get view=Lite not exposed
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - dns
  - api-coverage
---

# New DNS read parameters

- `GET /dnszone/{zoneId}/records` gained `search` (string) and `type` (`DnsRecordTypes2`, 0–15)
- `GET /dnszone/{id}` gained `view` (`DnsViewType`: `0 Full`, `1 Lite`)

`list_dns_zone_records` (`crates/bunny-net-api/src/core/client.rs:858`) sends
only `page` and `perPage`, and `get_dns_zone` sends no query parameters.

- [ ] Add `--search` and `--type` to `dns record list`, passed to the server
- [ ] Consider `view=Lite` for commands that do not need the records

See [[research/openapi-drift-2026-10-23]].
