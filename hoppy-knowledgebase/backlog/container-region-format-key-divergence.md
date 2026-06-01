---
title: "container region optimal uses three different field names across table/text/json"
type: backlog
date: 2026-05-31
status: resolved
priority: low
origin: dogfooding-2026-05-31
tags:
  - cli
  - container
  - dx
  - format
  - consistency
---

# Same field, three different names across formats

`hoppy container region optimal` renders the same upstream payload three
different ways:

| Format  | "Anycast" field             | "Capacity" field  |
|---------|-----------------------------|-------------------|
| `table` | `Anycast`                   | `Capacity`        |
| `text`  | `has_anycast_support`       | `has_capacity`    |
| `json`  | `hasAnycastSupport`         | `hasCapacity`     |

That's snake_case in `text`, camelCase in `json`, and a renamed column in
`table` — for the *same* field. Same divergence on `container region list`.

## Repro

```sh
hoppy container region optimal --format table
hoppy container region optimal --format text
hoppy container region optimal --format json
```

## Why this matters

- `text` is supposed to be "table with no borders, machine-greppable" — but
  the key names don't match `json`, so you can't `grep field; jq .field` and
  expect symmetry.
- The `table` column header drops the `has_` prefix and the `_support`
  suffix — readable, but means a user who knows the column name "Anycast"
  cannot search `--format json` output for it.

## Suggested fix

Pick one casing for non-tabular formats and stick with it. PascalCase
already used elsewhere in hoppy's CLI text output (e.g. `pull-zone get
--format text` prints `EnableGeoZoneUs`). Switch container's `--format text`
to PascalCase so it matches both `--format json` (with a single tongue-twist
of casing) and the existing convention used by every other domain.

## Related

- [[json-output-casing-inconsistency]]
- [[table-label-json-key-case-mismatch]]
