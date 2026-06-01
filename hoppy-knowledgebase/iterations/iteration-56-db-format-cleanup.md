---
title: Iter-56 — db config and db v2 list table rendering
type: iteration
date: 2026-06-01
tags:
  - iteration
  - db
  - format
  - table
  - consistency
status: completed
branch: iter-56/db-format-cleanup
---

# Iter-56 — db config and db v2 list table rendering

## Why

Two related friction points on the `db` surface that the
[[iteration-40-dogfooding-2026-05-27-fixes]] / `db active-usage`
fix missed:

1. `db config show` and `db config limits` ignore `--format` entirely
   and always emit raw JSON.
2. `db v2 list` renders only envelope placeholders
   (`<empty list>` for `Databases`, `<object: 3 fields>` for
   `PageInfo`) in `--format table`, instead of one row per database.

See [[../backlog/db-config-show-limits-ignore-format]] and
[[../backlog/db-v2-list-table-placeholders]].

## Scope

### 1. `db config show` and `db config limits` [3/3]

- [x] Wire both subcommands through the standard `--format` pipeline
      (the same one `db active-usage` and `pull-zone get` use).
- [x] Implement a meaningful `table` rendering:
      - `show`: one table per region list (storage regions, primary
        regions) or a single combined table with a `Kind` column.
      - `limits`: simple Field/Value table.
- [x] Implement `text` (tab-separated key/value) rendering.

### 2. `db v2 list` table [2/2]

- [x] Render the `Databases` array as the primary table — one row per
      database with useful columns (id, name, region, created, size).
- [x] Print `PageInfo` as a trailing single-row table or a stderr
      footer; suppress it under `--format json` paths (no change to
      JSON envelope).

### 3. Tests [3/3]

- [x] E2E mock test for each of the three `db config` formats.
- [x] E2E mock test for `db v2 list` table with 0, 1, and N
      databases (uses fixtures).
- [x] Snapshot tests that exercise structural assertions (column
      headers present, row count matches) per the
      [[../dogfooding/dogfooding-playbook]] drift guidance.

## Out of scope

- `db config optimal` — separately broken upstream
  ([[../backlog/db-config-optimal-single-broken]]).
- Adding new columns to the v2 list response.

## Acceptance Criteria

- [x] `db config show --format table|text|json` produce three
      distinct, useful outputs.
- [x] `db config limits --format table|text|json` produce three
      distinct, useful outputs.
- [x] `db v2 list` with N databases shows N rows in table mode;
      shows `No results.` when empty (matching every other `* list`).
- [x] `db v2 list --format json` unchanged.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/db-config-show-limits-ignore-format]]
- [[../backlog/db-v2-list-table-placeholders]]
- [[../backlog/db-active-usage-ignores-format]]
- [[../dogfooding/session-2026-06-01]]
