---
title: Iter-41 — dogfooding 2026-05-28 polish
type: iteration
date: 2026-05-28
tags:
  - iteration
  - bugfix
  - cli
  - polish
  - help-text
status: completed
branch: iter-41/dogfooding-2026-05-28-polish
---

# Iter-41 — dogfooding 2026-05-28 polish

## Why

Post-iter-40 dogfooding confirmed all three iter-40 fixes work as
designed (container app get: 922 → 49 chars; shield `--id` accepted with
`--shield-zone-id` alias; storage paths single-slashed). The round
surfaced three small follow-ups — all low priority, all bite-sized.
Bundling them as one polish iteration rather than three separate ones
because each is a sub-day fix.

## Scope

### 1. `<1 items>` plural mismatch

Source: [[backlog/summary-cell-plural-mismatch]]

iter-40's array-summary cell renders `<1 items>` for n=1.

- [x] In the summary-cell renderer (the helper added by iter-40 #1),
      pluralise the label: `n == 1 ? "item" : "items"`. Same treatment
      for any other countable label.
- [x] Unit test: `<1 item>` and `<2 items>` both render correctly.
- [x] e2e snapshot refresh for any zone whose `Hostnames` array length
      is 1 (the dogfooding test zone has exactly this shape).

### 2. List-table columns overflow on long free-text values

Source: [[backlog/list-tables-overflow-on-long-text-columns]]

iter-39/40 fixed `get` width. The symmetric problem on `list` is
unfixed: e.g. `shield waf profiles` renders at 267 chars because the
`Description` column carries multi-sentence prose. Likely also affects
`shield rate-limit list`, `pull-zone edge-rule list`, `dns record list`
(TXT values).

- [x] Audit every `list`-style command for columns that can carry
      arbitrarily long text. Identify the offenders mechanically by
      running each `list` against the live test account and measuring
      first-line width; flag anything > 160 chars.
- [x] Pick a strategy and apply uniformly:
      **Truncate long text cells** at e.g. 60 chars with `…` in table
      mode, leaving JSON mode untouched. Add a stderr `tip:` redirecting
      the user at the JSON view when truncation actually happens
      (don't print a stray tip on every list that just happens to have
      a long-text column whose values are short today).
- [x] Out of scope: dynamic `$COLUMNS`-aware sizing (bigger change, can
      land later if truncation isn't enough).
- [x] e2e snapshot refresh for the affected commands. Keep
      drift-tolerant per the iter-37 playbook.
- [x] Dogfooding pass: re-run `shield waf profiles` and confirm output
      fits a 120-col terminal with truncation indicators visible.

### 3. Sub-resource `--id` args lack help text

Source: [[backlog/edge-rule-list-id-no-doc]]

`hoppy pull-zone edge-rule list --help` shows `--id <ID>` with no
description. The user has to guess whether `--id` refers to the pull
zone or the edge rule. (Convention: parent resource.)

- [x] Audit every clap `#[arg(long)]` for sub-resource commands that
      uses bare `--id` with no `help = "..."`. Suggested grep:
      `grep -rn -B1 -A3 '#\[arg' crates/hoppy-cli/src/cli/ | grep -B3 '"id"'`
      (or equivalent).
- [x] Add a `help = "<Parent> ID"` attribute to each. Examples:
      - `pull-zone edge-rule list` → `help = "Pull zone ID"`
      - `pull-zone hostname add` → `help = "Pull zone ID"`
      - `container endpoint list` → `help = "Container app ID"`
      - `dns record list` → `help = "DNS zone ID"`
- [x] e2e snapshot refresh for changed `--help` output.

## Out of scope

- Restructuring how `list` tables render in general (the dynamic
  $COLUMNS-aware sizing approach). Truncation is the cheap path.
- Adding any new sub-resource commands or changing arg shapes beyond
  adding `help` text and pluralising one label.
- Cleaning up the stale `LogForwardingHostname`/`Token` state on the
  dogfooding test pull zone — iter-39's guard works as designed and
  the state isn't blocking anything.

## Acceptance

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] Dogfooding repro of all three issues against the live test account
      shows the fixed behaviour:
      - `pull-zone get --id <id>` Hostnames row reads `<1 hostname>` or
        `<1 item>` (singular).
      - `shield waf profiles` fits on a 120-col terminal with `…`
        truncation indicators in the Description column.
      - `pull-zone edge-rule list --help` shows
        `--id <ID>  Pull zone ID` (or similar) instead of an undocumented
        flag.
- [x] All three backlog items closed (`status=resolved`) with a link to
      this iteration.

## Related

- [[backlog/summary-cell-plural-mismatch]]
- [[backlog/list-tables-overflow-on-long-text-columns]]
- [[backlog/edge-rule-list-id-no-doc]]
- [[iteration-39-dogfooding-2026-05-26-fixes]] — `get` pivot
- [[iteration-40-dogfooding-2026-05-27-fixes]] — summary cells
- Dogfooding round: 2026-05-28 (post-iter-40).
