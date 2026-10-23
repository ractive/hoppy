---
title: External HTTP certificate request/complete endpoints not covered
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - pull-zone
  - certificates
  - api-coverage
---

# External HTTP certificate flow

New operations: `POST /pullzone/requestExternalHttpCertificate` and
`POST /pullzone/completeExternalHttpCertificate` (body
`ExternalHttpCertificateRequest { Hostname }`). These are the HTTP-01
counterpart of the DNS flow that hoppy already wraps
(`crates/bunny-net-api/src/core/client.rs:356`). The DNS variants now also
document 429 responses.

- [ ] Add client methods and a `pull-zone certificate external-http request|complete` command

See [[research/openapi-drift-2026-10-23]].
