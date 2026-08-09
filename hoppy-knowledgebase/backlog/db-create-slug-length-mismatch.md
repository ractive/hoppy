---
title: db create local slug validation (24) doesn't match upstream limit (~<19)
type: backlog
date: 2026-08-09
tags: [backlog, database, validation]
status: open
priority: low
origin: dogfooding-2026-08-09
---

# `db create` local slug validation doesn't match the real upstream limit

hoppy validates slugs against `^[a-z][a-z0-9-]{0,23}$` (24 chars) and the
help text says this local check prevents the upstream "Internal error"
500 on long slugs. It doesn't: during the iter-81 dogfood,
`--slug hoppy-test-fork-src` (19 chars) passed local validation and got
`HTTP 500 {"error":"Internal error"}`, while `hoppy-test-db` (13 chars)
worked. The real upstream limit is somewhere in 14–18 chars (or depends
on the combined `<group-ulid>-<slug>` hostname length —
`libsql://<26-char-ulid>-<slug>.lite.bunnydb.net` suggests a total
hostname cap is the actual constraint).

## Want

- Binary-search the real limit (or find it in bunny docs), encode it in
  the local validator, and fix the help text.
- Check whether the limit is on `len(group_ulid) + 1 + len(slug)` rather
  than the slug alone.
