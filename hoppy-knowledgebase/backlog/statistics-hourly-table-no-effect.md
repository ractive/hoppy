---
title: "`statistics --hourly` produces identical table output to non-hourly"
type: backlog
date: 2026-05-31
status: planned
priority: medium
origin: dogfooding-2026-05-31
tags:
  - cli
  - statistics
  - dx
  - flag-without-effect
---

# `statistics --hourly` has no visible effect in the default table view

`hoppy statistics --hourly` and `hoppy statistics` (no flag) print byte-for-byte
identical tables — only the five top-level summary metrics. The JSON output
*does* contain `*Chart` maps keyed by ISO timestamps (hourly buckets when
`--hourly` is set, daily otherwise), but the default table format never shows
them.

## Repro

```sh
BUNNY_API_KEY=$TEST_BUNNY_API_KEY hoppy statistics
BUNNY_API_KEY=$TEST_BUNNY_API_KEY hoppy statistics --hourly
# diff is empty — same table
```

Confirmed with `--date-from 2026-05-30 --date-to 2026-05-31`:
- non-hourly: 24-key chart (one per hour, but bucketed daily on the API)
- `--hourly`: 48-key chart with `2026-05-30T09:00:00Z`, `2026-05-31T09:00:00Z`, …
- both render the same five-row summary table

## Why this is friction

A `--hourly` flag implies the output will change. Either:

1. The flag should affect the table — e.g. add a chart sparkline or a
   per-bucket breakdown column when `--hourly` is set, or
2. The flag should print a hint after the table (something like
   "hourly chart available via `--format json`") so the user isn't left
   wondering whether the flag took effect.

The current behaviour reads as "broken flag" during dogfooding even though the
API call did honour it.

## Related

- [[json-output-casing-inconsistency]]
- [[drill-down-hints]]
