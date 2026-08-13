---
title: db group create region flags use two vocabularies with no local validation
type: backlog
date: 2026-08-09
tags:
  - backlog
  - database
  - dx
  - validation
status: resolved
priority: low
origin: dogfooding-2026-08-09
---

# `db group create` region flags mix two vocabularies, no local validation

`--primary-region` takes short uppercase codes (`DE`, `AMS`, ...);
`--storage-region` takes AWS-style ids (`eu-west-1`, `us-east-1`). Help
text documents neither vocabulary, values are case-sensitive, and errors
come raw from the API — lowercase `de` for primary returns a JSON-schema
dump; `DE` for storage returns a 500 "This storage region DE is not
supported." Both vocabularies are available locally via
`db config show`, so the CLI could validate before the call (mirroring
what `db create` does for slugs) or at least name the valid values in
help text/possible-values.

## Resolution (iter-84, 2026-08-13)

`db group create` now pre-flight-validates `--storage-region`,
`--primary-region`, and `--replicas-region` case-sensitively against
the live vocabulary from `GET /v1/config` before the mutating POST. An
unknown value fails locally with the valid-value list; a casing-only
mismatch gets a did-you-mean. No region vocabulary is hardcoded. Help
text for all three flags now names the vocabulary shape with examples
and points at `hoppy db config show`.

`db group update` takes the same `--primary-region`/`--replicas-region`
flags but was deliberately left unvalidated in this pass: the
iteration's Design/Acceptance-criteria sections scope the fix to
`db group create` only, and the existing `db_group_update_regions` e2e
test asserts a region (`NY`) that isn't in the checked-in
`fixtures/database/config.json` vocabulary — adding validation there
would require either changing that test's region values or fixtures
(off-limits per repo policy) or expanding the fixture. Flagged as a
natural follow-up, not done here.
