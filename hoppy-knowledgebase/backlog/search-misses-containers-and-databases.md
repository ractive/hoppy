---
title: search misses container apps, databases and container pull zones
type: backlog
date: 2026-10-27
status: resolved
priority: medium
origin: demo-prep-2026-10-27
resolved-in: iter-91
---

# `hoppy search` misses container apps, databases and container pull zones

`hoppy search` passes the query straight to `GET /search` (`handle_search` in
`crates/hoppy-cli/src/commands/account.rs`). That endpoint matches names only and indexes only
DNS zones, pull zones and storage zones.

`hoppy search study2go` returns 3 hits (DNS `857052`, CDN `6442623`, storage `1888383`). It misses:

- Container app `study2go` (`p7Q7eyp6R6fRlPK`). `/search` has no container type.
- Databases `study2go` and `study2go-dev`. `/search` has no database type.
- Pull zone `6442567` (`mc-ne3kick2p8`), which `pull-zone list` shows with `App = study2go`.
  Its name doesn't contain the query, and the `App` column comes from hoppy's own join.

## Fix sketch

- [x] Besides `/search`, match the query case-insensitively against `container app list` names and
  `db list` names. Add rows with types `container` and `db`.
- [x] For each matched container app, add the pull zones of its endpoints that aren't listed yet
  (type `cdn`, the same join as `pull-zone list`).
- [x] `--server-only` keeps the old single-request behaviour. If the client-side lookups fail,
  print a warning to stderr instead of failing the command.
