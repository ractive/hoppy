---
title: Iter-51 — `--format` parity sweep across all subcommands
type: iteration
date: 2026-06-01
tags: [iteration, cli, format, dx, consistency]
status: planned
branch: iter-51/format-parity-sweep
---

# Iter-51 — `--format` parity sweep

## Why

Three separate dogfooding items all point at the same gap: `--format`
is partially honoured. Bundling them avoids three near-identical PRs.

- `db` v2-style commands ignore `--format` and always print snake_case
  JSON — see [[../backlog/db-active-usage-ignores-format]].
- Mutation subcommands (`create`/`update`/`delete`) always print prose,
  even with `--format json` — see [[../backlog/mutation-commands-ignore-format-json]].
- `container region` renders the same field three ways across
  `table`/`text`/`json` — see [[../backlog/container-region-format-key-divergence]].

Goal: every CLI subcommand honours `--format table|text|json`, and
field names are consistent between `text` and `json`. Convention:
PascalCase for non-tabular formats (matches existing `pull-zone get
--format text`).

## Scope

### 1. Audit existing format coverage

- [ ] Grep every subcommand handler for `--format` handling. Build a
      matrix: command × {table, text, json} × honoured?
- [ ] Note the convention each command currently uses for casing.
- [ ] Paste the matrix into the PR body.

### 2. Fix `db` v2-style commands

- [ ] All `db` subcommands honour `--format json` and emit
      PascalCase keys matching other domains.
- [ ] `--format text` uses the same key names as `--format json`.

### 3. Fix mutation subcommands

- [ ] `create`/`update`/`delete` across all domains honour
      `--format json` — print the API response (or a success
      envelope) as JSON, not the prose confirmation.
- [ ] Default (no `--format`) keeps current prose behaviour.

### 4. Fix `container region` field-name divergence

- [ ] `--format text` uses PascalCase (`HasAnycastSupport`,
      `HasCapacity`) to match `--format json` after the casing fix.
- [ ] `--format table` headers map cleanly (document the mapping in
      help text if abbreviated).

### 5. Tests

- [ ] E2E snapshot per fixed command × format.
- [ ] Add a coverage test that asserts every subcommand returns
      something non-empty for each format (catches future
      regressions).

## Out of scope

- Rewriting prose output for failure cases.
- Adding new output formats (yaml, csv, etc.).

## Acceptance Criteria

- [ ] Every subcommand emits valid JSON when `--format json` is
      passed.
- [ ] Field names match between `--format text` and `--format json`.
- [ ] Snapshot tests cover the previously-broken commands.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/db-active-usage-ignores-format]]
- [[../backlog/mutation-commands-ignore-format-json]]
- [[../backlog/container-region-format-key-divergence]]
- [[../backlog/json-output-casing-inconsistency]]
