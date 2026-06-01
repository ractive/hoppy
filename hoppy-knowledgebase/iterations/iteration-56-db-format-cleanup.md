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
status: planned
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

### 1. `db config show` and `db config limits` [0/3]

- [ ] Wire both subcommands through the standard `--format` pipeline
      (the same one `db active-usage` and `pull-zone get` use).
- [ ] Implement a meaningful `table` rendering:
      - `show`: one table per region list (storage regions, primary
        regions) or a single combined table with a `Kind` column.
      - `limits`: simple Field/Value table.
- [ ] Implement `text` (tab-separated key/value) rendering.

### 2. `db v2 list` table [0/2]

- [ ] Render the `Databases` array as the primary table — one row per
      database with useful columns (id, name, region, created, size).
- [ ] Print `PageInfo` as a trailing single-row table or a stderr
      footer; suppress it under `--format json` paths (no change to
      JSON envelope).

### 3. Tests [0/3]

- [ ] E2E mock test for each of the three `db config` formats.
- [ ] E2E mock test for `db v2 list` table with 0, 1, and N
      databases (uses fixtures).
- [ ] Snapshot tests that exercise structural assertions (column
      headers present, row count matches) per the
      [[../dogfooding/dogfooding-playbook]] drift guidance.

## Out of scope

- `db config optimal` — separately broken upstream
  ([[../backlog/db-config-optimal-single-broken]]).
- Adding new columns to the v2 list response.

## Acceptance Criteria

- [ ] `db config show --format table|text|json` produce three
      distinct, useful outputs.
- [ ] `db config limits --format table|text|json` produce three
      distinct, useful outputs.
- [ ] `db v2 list` with N databases shows N rows in table mode;
      shows `No results.` when empty (matching every other `* list`).
- [ ] `db v2 list --format json` unchanged.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/db-config-show-limits-ignore-format]]
- [[../backlog/db-v2-list-table-placeholders]]
- [[../backlog/db-active-usage-ignores-format]]
- [[../dogfooding/session-2026-06-01]]
