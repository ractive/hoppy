---
title: Why pull-zone list omits Magic Containers zones
type: research
date: 2026-10-23
status: active
tags:
  - research
  - pull-zone
  - magic-containers
origin: iter-86
---

# `GET /pullzone` never returns Magic Containers zones

Findings for [[backlog/pull-zone-list-omits-magic-container-zones]].

- The OpenAPI spec (`specs/core-platform.json`, and the 2026-10-23 re-download) documents only `page`, `perPage`, `search`, `includeCertificate` on `GET /pullzone`. No parameter includes origin-type-5 zones.
- Live probe with the test account (read-only): `page=1&perPage=100` plus guessed flags (`includeMagicContainers`, `includeContainers`, `IncludeMagicContainers`, `includeMC`, `includeAll`) all return identical results; unknown parameters are ignored. The test account has zero container apps and zero pull zones, so the omission itself could not be reproduced there; the evidence for it is the dogfooding report (list returns 4 of 6, `search=mc-...` yields `TotalItems:1` with empty `Items`, `GET /pullzone/{id}` works).
- `GET /pullzone/count` shows the same blind spot (reported 4 for 6 zones).
- Global `GET /search` does find the zones, so `hoppy search` is unaffected.

## Decision (no parameter -> fallback)

Complete the list from the container side: `GET /apps` (cursor-paged) -> `GET /apps/{id}/endpoints` (each endpoint has `pullZoneId`) -> `GET /pullzone/{id}` for ids not already listed. Requests are bounded to 8 in flight; zones are deduplicated by id. Zones are appended on the last page; rows carry an `App` column (table) / `ContainerApp` member (JSON). `count` lists all ordinary zones and merges only when container zones exist. `--no-container-zones` opts out. Failure of the container lookup degrades to a stderr warning.

## Other places that enumerate pull zones

Audited in iter-86: only `pull-zone list` and `pull-zone count` call `GET /pullzone` / `/pullzone/count`. `search` is server-side global (already finds MC zones). Statistics/purge/logs/shield commands take an explicit id; there are no pickers, name-resolvers or dynamic shell completions. `container app delete --cascade` already discovers zones via endpoints.
