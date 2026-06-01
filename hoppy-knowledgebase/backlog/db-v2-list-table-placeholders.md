---
title: db v2 list table renders only envelope placeholders
type: backlog
date: 2026-06-01
status: planned
priority: medium
origin: dogfooding-2026-06-01
tags:
  - cli
  - db
  - table
  - dx
---

# `db v2 list` table is unhelpful

```sh
$ hoppy db v2 list
+-----------+--------------------+
| Field     | Value              |
+-----------+--------------------+
| Databases | <empty list>       |
+-----------+--------------------+
| PageInfo  | <object: 3 fields> |
+-----------+--------------------+
```

The table format treats `db v2 list` as a single-record object and
renders only the envelope: a placeholder for the (likely-non-empty)
`Databases` array and a placeholder for the pagination metadata.

Compare with `pull-zone list`, `storage-zone list`, etc. — those
render one row per item in the list and surface column-meaningful
fields (ID, Name, …).

## Expected behaviour

- Default `--format table` should render the `Databases` list directly,
  with one row per database and useful columns (id, name, region,
  created, size, etc.).
- Pagination metadata can be printed as a stderr footer or a separate
  trailing single-row table.
- `--format json` keeps the current envelope shape (no API drift).

## Reproduction

```sh
hoppy db v2 list                # current: placeholders
hoppy db v2 list --format json  # current: full envelope JSON
```

## Acceptance

- `db v2 list` table format renders a per-database row table.
- `db v2 list` with N databases shows N rows; with 0 shows "No
  results." (matching every other `* list` command).
- `--format json` output unchanged.
