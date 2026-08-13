---
title: db create local slug validation (24) doesn't match upstream limit (~<19)
type: backlog
date: 2026-08-09
tags:
  - backlog
  - database
  - validation
status: resolved
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

## Resolution (iter-84, 2026-08-13)

Live binary-search on the test account: 16-char slugs create fine; 17,
18, and 19 chars all return HTTP 500 "Internal error". `SLUG_MAX_LEN` is
now 16 (`^[a-z][a-z0-9-]{0,15}$`), and the error message / `--slug`
help text reflect the measured boundary. Could not isolate "slug
length" from "`<group-ulid>-<slug>` hostname length" with only one
group available in the test account — the group ULID is always 26
chars, so the two hypotheses are observationally identical from a
single-group probe. Either way, 16 is the effective slug limit to
encode. See [[iterations/iteration-84-backlog-fixes]].
