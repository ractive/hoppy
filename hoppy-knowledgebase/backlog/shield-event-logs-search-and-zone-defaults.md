---
title: Shield event-log search/export, zone defaults and under-attack create not covered
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - shield
  - api-coverage
---

# New Shield surface (Oct 2026)

- `POST /shield/event-logs/{shieldZoneId}/search`: filters, free-text query, `groupBy`, buckets and pagination (`EventLogsSearchRequest`/`Response`)
- `POST /shield/event-logs/{shieldZoneId}/export`: CSV export (stream the body)
- `GET /shield/shield-zone/defaults`: recommended defaults, including suggested access lists
- `POST /shield/shield-zone/under-attack`: create a zone in under-attack mode
- `CreateShieldZoneRequest` gained `accessLists`, `botDetectionExecutionMode`, `csamScanningMode` and `antivirusScanningMode`
- `WafMappedEnum.valueConstraint` (`WafValueConstraint`, `WafValueKind`) can drive client-side validation of rule values

Related: [[backlog/shield-event-logs-discards-error-body]].

- [ ] Add client methods and types in `crates/bunny-net-api/src/shield/`
- [ ] Add `shield event-logs search|export`, `shield zone defaults` and `shield zone create --under-attack`

See [[research/openapi-drift-2026-10-23]].
