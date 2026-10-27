---
title: pull-zone list still misses a container zone that no endpoint points to
type: backlog
date: 2026-10-27
status: resolved
priority: medium
origin: demo-prep-2026-10-27
resolved-in: iter-91
---

# `pull-zone list` still misses a container zone that no endpoint points to

Follow-up to [[backlog/pull-zone-list-omits-magic-container-zones]] (fixed in
[[iterations/iteration-86-dogfood-oct-fixes]]). The iter-86 fallback adds the zones it reaches through
`GET /apps` → `GET /apps/{id}/endpoints` → `pullZoneId`
(see [[research/pull-zone-list-magic-containers-2026-10-23]]). A zone that no endpoint's
`pullZoneId` names is still left out.

Seen with `target/release/hoppy` (main, 2026-10-23) on the private account:

- `hoppy pull-zone list` prints 8 zones and `pull-zone count` says 8.
- Pull zone `6442623` (`study2go`, `OriginType` 5) serves `study2go.ch` and
  `www.study2go.ch`. It has live traffic (`hoppy logs pull-zone 6442623`), and the
  `study2go.ch` DNS records point at it. It is missing from both commands.
- `hoppy search study2go` finds it (type `cdn`), and `hoppy pull-zone get 6442623` works.
- `GET /pullzone?page=1&perPage=1000` itself returns only 8 items. hoppy does no
  client-side filtering.

Why the fallback misses it: app `p7Q7eyp6R6fRlPK` has one endpoint,
`p7Q7eyp6R6fRlPK-study2go-JAY6AQJFSW` (host `mc-ne3kick2p8.bunny.run`). Its
`pullZoneId` leads to `6442567` (`mc-ne3kick2p8`), which is listed. Zone `6442623`
names the same endpoint in its `MagicContainersEndpointId`, so two pull zones
share one endpoint, and the endpoint links back to only one of them.

## Why it matters

The zone that carries the production domain is the one missing from the list. The
listed zone, `mc-ne3kick2p8`, has no traffic. Anyone looking for "the study2go
CDN zone" in `list` finds the wrong one.

## Fix sketch

- [x] Also merge `cdn` hits from global `GET /search` (it already finds the zone) for ids
  not seen yet, then `GET /pullzone/{id}` them. Check whether `/search` can be
  scoped by type without a query string.
- [ ] Or, for each container app, search by the app id / endpoint id and merge zones
  whose `MagicContainersAppId` matches.
- [x] Add a fixture with two zones on one endpoint and a test that both are listed.
