---
title: Partner Portal API (new spec, 13 ops) not covered
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - partner-portal
  - api-coverage
  - new-service
---

# Partner Portal API

This is a new spec at
`https://core-api-public-docs.b-cdn.net/docs/v3/PartnerPortal.json`, server
`https://api.bunny.net`. It is not in `specs/`. It has 13 operations under
`/partner/*`: create, invite and list customers; customer overview, billing,
invoice PDF, pricing, products (list/set) and audit log; and the partner
overview, contract and pricing.

It is only relevant to reseller or partner accounts, so it is low priority
unless a user asks for it.

- [ ] Add `specs/partner-portal.json` with the next spec refresh
- [ ] Decide whether to support it (feature-gated `partner` module)

See [[research/openapi-drift-2026-10-23]].
