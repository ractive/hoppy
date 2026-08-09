---
title: db fork — non-spec `group` field drift
type: backlog
date: 2026-07-10
status: resolved
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

## Dogfood check attempt (2026-08-09, iter-81)

Not testable this round: the test account has no database groups and no
databases (`db group list` and `db list` both return `[]`), and a
point-in-time fork needs PITR history that a freshly created database
would not have accumulated within a dogfooding session. Remains open.
Practical path to resolution: seed a database on the test account, let it
age past the PITR window granularity, then fork with and without
`--group` and compare destination groups. Alternatively close as
overtaken if a future spec bump removes `group` for good (the flag is
already documented as ignored by the current API).

## Live verification (2026-08-09, iter-81 — conclusive)

The account limit "one database per namespace" (403, `max_namespace: 1`)
made the question testable after all:

1. Created group A + db in it; fork into a fresh empty group B via
   `--group` → 403 namespace-full.
2. Control: `db create` directly into the same empty group B → succeeds.

So the fork honored the source namespace and **ignored the `group`
field** — consistent with the spec's `{slug, date}`-only payload.
`group` is dead weight and the `--group` flag actively misleads (users
would expect the fork to land in the target group).

**Action: drop `group` from `ForkDatabasePayload` and remove the
`db fork --group` flag.**

## Resolution (2026-08-09, iter-81)

`group` dropped from `ForkDatabasePayload` and the `db fork --group` flag
removed (clap now rejects it; pinned by the `db_fork_rejects_group` e2e
test). Help text states that forks always land in the source database's
group.
