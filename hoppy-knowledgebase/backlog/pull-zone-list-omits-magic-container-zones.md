---
title: pull-zone list and count omit Magic Containers pull zones
type: backlog
date: 2026-10-21
status: completed
priority: medium
origin: dogfooding-2026-10-21
---

# `hoppy pull-zone list` and `count` miss the Magic Containers zones

Seen with `hoppy 0.7.0 (1eddb33a8515 2026-08-30)` on an account with six pull zones in the dashboard: four
ordinary ones and two that Magic Containers created for app endpoints
(`mc-jm6w1xfghv`, id 6834946, and `mc-ne3kick2p8`, id 6442567; origin type 5,
"Magic Container").

- `hoppy pull-zone list` prints only the four ordinary zones.
- `hoppy pull-zone count` says 4.
- `hoppy pull-zone get --id 6834946` and `--id 6442567` work, and
  `hoppy search ne3kick2p8` finds the zone (Type `cdn`).

The raw API behaves the same: `GET /pullzone?page=0&perPage=100` returns four
items, and `GET /pullzone?search=mc-ne3kick2p8` answers
`{"Items":[],"TotalItems":1,"HasMoreItems":true}` on pages 0 and 1, so the
list endpoint counts the zone but never returns it.

## Why it matters

Finding which zone belongs to which container app needs the zone list: the
owner saw two "Magic Container" zones in the dashboard and asked whether one
was a stray duplicate. hoppy could not answer it from `list`; it took
`search`, then `get --id`, then `container get`.

## Fix sketch

- Find out whether the list endpoint has a parameter that includes Magic
  Containers zones; if so, pass it (or offer `--include-containers`).
- Else complete the list from `container app list` → each endpoint's
  `pullZoneId` → `pull-zone get`, and mark those rows (e.g. a column "App").
- At least say in `list` / `count` output that Magic Containers zones are not
  included, and how to see them.
