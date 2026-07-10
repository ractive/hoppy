---
title: db fork — non-spec `group` field drift
type: backlog
date: 2026-07-10
status: open
priority: low
origin: iter-66 spec refresh
resolved-by: iteration-66-spec-refresh-drift-fixes
---

# `db fork` payload drift: `{slug, date}` vs `{slug, group}`

The refreshed `specs/database.json` defines `ForkDatabasePayload` as
`{slug, date}` with **both required** — a point-in-time fork where `date` is an
RFC 3339 `date-time`. Earlier hoppy releases sent `{slug, group}` instead,
which is not in the current spec.

## What iter-66 changed

- `ForkDatabasePayload` is now `{slug, date, group?}`:
  - `date` is required and always serialised.
  - `group` is optional and serialised **only when set** (`skip_serializing_if`).
- `db fork` gained a required `--date <RFC3339>` flag and kept `--group` as an
  optional passthrough.

## Open question (needs a live check)

Whether the API still accepts (or silently ignores) the non-spec `group`
field. It is kept for backward compatibility but is expected to be ignored.
Next dogfooding pass: fork with and without `--group` against a real account
and confirm the destination group. If the field is rejected, drop `group` from
the payload and the CLI flag entirely.
