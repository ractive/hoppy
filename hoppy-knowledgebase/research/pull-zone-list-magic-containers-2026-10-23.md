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

## Addendum (iter-91): second zone on the same endpoint

Endpoint `pullZoneId` names one zone only. Live case: app `p7Q7eyp6R6fRlPK` (study2go), endpoint `...-study2go-JAY6AQJFSW` -> `pullZoneId` 6442567 (`mc-ne3kick2p8`), but zone 6442623 (`study2go`, OriginType 5, carrying the production hostnames) has the same `MagicContainersAppId` and was never listed.

Read-only probes (private account):
- Endpoint objects expose no further ids (spec `EndpointListItem` has a single `pullZoneId`); no API lists zones by app.
- `GET /search` indexes pull zone names and hostnames, not app ids, endpoint ids or endpoint names: `p7Q7eyp6R6fRlPK` and `JAY6AQJFSW` give 0 hits. `study2go`, `study2go.ch` and `www.study2go.ch` return zone 6442623; `mc-ne3kick2p8` returns 6442567.
- Empty or `.`/`*` queries do not enumerate (`Total` is reported but `SearchResults` is empty).
- `GET /pullzone?search=` still omits origin-type-5 zones.

Decision: for each app, search its name (plus its display address when it is not a `*.b-cdn.net` host), take the `cdn` hits, `GET /pullzone/{id}` them and keep only those whose `MagicContainersAppId` equals the app id. Dedupe by id against the endpoint-referenced and already-listed zones. Limitation: a zone whose name and hostnames contain neither the app name nor its display address is not found. `/search` rate-limits (HTTP 429) after a few quick requests, so searches run sequentially with a short retry back-off; a remaining failure is a stderr warning.
