---
title: All `db` v2-style commands ignore `--format` and print snake_case JSON
type: backlog
date: 2026-05-31
status: resolved
priority: medium
origin: dogfooding-2026-05-31
tags:
  - cli
  - db
  - dx
  - format
  - consistency
---

# `db` v2-style commands ignore `--format` and use snake_case keys

The pattern repeats across the entire `db v2` family and the v2-shaped
top-level helpers (`db active-usage`, `db v2 list`, `db v2 get`, `db usage`,
`db live`). All emit raw JSON in snake_case regardless of `--format`:

```sh
hoppy db active-usage
hoppy db active-usage --format table
hoppy --format table db active-usage
# All three print:
# {
# "active_db": 0,

# "total_db": 0,

# "total_db_size": "0 B"

# }

hoppy db v2 list --format table
# {
# "databases": [],

# "page_info": { "current_page": 1, "total_items": 0, "has_more_items": false }

# }
```

Two problems:

1. **`--format table` is silently ignored.** The default global format is
   `table`, so a user typing `hoppy db active-usage` without `--format json`
   already sees JSON — surprising and inconsistent with `db list`,
   `db statistics`, `pull-zone get`, etc.
2. **Snake_case keys** (`active_db`, `total_db_size`) — every other JSON
   surface uses either PascalCase (core API) or camelCase (shield / container).
   See [[json-output-casing-inconsistency]] for the broader pattern; this
   command adds a *third* casing in one binary.

## Suggested fix

Render a real 3-row metric table by default. Mirror the casing used in
`db list` (PascalCase or camelCase, whichever the v2 API actually returns)
for JSON output. Drop the snake_case rewrite.

## Related

- [[json-output-casing-inconsistency]]
