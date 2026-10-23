---
title: db list --per-page 0 now rejected by the API (default changed to 50)
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - database
  - api-drift
---

# `GET /v2/databases` `per_page` semantics changed

Database spec 0.0.130 → 0.0.144: `per_page` is now `uint32` with `minimum: 1`
and `default: 50`, and is documented as "`0` is rejected". A 400 response was
added. `db list` auto-pagination passes `AUTO_PER_PAGE`, so it is unaffected.
An explicit `--per-page 0` passes through `list_databases_v2`
(`crates/bunny-net-api/src/database/client.rs:320`) and now fails with a 400.
`GET /v1/groups/{id}` also documents 404 now.

- [ ] Validate `--per-page >= 1` in the CLI (clap range)

See [[research/openapi-drift-2026-10-23]].
