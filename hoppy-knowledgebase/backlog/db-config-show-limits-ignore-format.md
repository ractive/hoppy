---
title: db config show and db config limits ignore --format
type: backlog
date: 2026-06-01
status: resolved
priority: medium
origin: dogfooding-2026-06-01
tags:
  - cli
  - db
  - format
  - consistency
---

# `db config show` and `db config limits` always emit raw JSON

```sh
$ hoppy db config show --format table
{
  "storage_region_available": [
    ...
  ]
}

$ hoppy db config limits --format table
{
  "current_databases": 0,
  "max_databases": 50
}
```

The `--format` flag is silently ignored: passing `--format table` (the
documented default) still produces raw JSON. Same with `--format text`.

This is a cousin of [[db-active-usage-ignores-format]] (resolved for
`db active-usage` and the `db v2` family) but the `db config` subtree
was missed.

## Affected

- `hoppy db config show`
- `hoppy db config limits`

`hoppy db config optimal` is separately broken (HTTP 400 missing
`cdn_server_token`) — see [[db-config-optimal-single-broken]].

## Expected behaviour

- `--format table` renders a table:
  - `db config show` → one row per region in each list, or two tables
    (storage regions, primary regions).
  - `db config limits` → simple Field/Value table with
    `current_databases` and `max_databases`.
- `--format text` renders a flat tab-separated key/value form.
- `--format json` continues to emit the current raw JSON (snake_case
  is fine for v2-style endpoints).

## Acceptance

- All three formats produce non-identical output.
- Default (`--format table`) renders a table for both subcommands.
