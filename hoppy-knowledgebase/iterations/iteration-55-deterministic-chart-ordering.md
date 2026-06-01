---
title: Iter-55 — Deterministic chart key ordering in JSON
type: iteration
date: 2026-06-01
tags:
  - iteration
  - statistics
  - stream
  - json
  - determinism
status: in-progress
branch: iter-55/deterministic-chart-ordering
---

# Iter-55 — Deterministic chart key ordering in JSON

## Why

`hoppy statistics` and `hoppy stream library statistics` return
chart fields as date-keyed maps that come out in HashMap insertion
order. This breaks deterministic fixture diffs, makes the JSON
hard to scan by eye, and forces every downstream consumer to
re-sort.

See [[../backlog/statistics-chart-keys-unordered-json]].

## Scope

### 1. Audit chart-shaped fields [0/2]

- [x] Grep the workspace for serde models with map-of-date fields
      across statistics, stream, and any nascent db/container
      metrics surfaces.
- [x] List each affected field in the PR description.

### 2. Implement [0/2]

- [x] Switch the affected map types from `HashMap` (or
      `IndexMap` if used) to `BTreeMap<String, T>` so serialised
      output is key-sorted automatically.
- [x] Verify no callers rely on insertion order.

### 3. Tests [0/2]

- [x] Add a regression test that calls a statistics surface twice
      against the same mock and asserts byte-identical JSON output.
- [x] Refresh any affected fixtures (`fixture-refresh` two-step) and
      review the diff — should be purely reordering.

## Out of scope

- Changing the response shape (no added/removed fields).
- Implementing the same fix for non-chart fields where ordering is
  semantically irrelevant.

## Acceptance Criteria

- [x] Re-running `hoppy statistics --format json` (and the equivalent
      stream stats command) produces byte-identical output between
      two runs against the same backend state.
- [x] Chart keys appear in ascending date order.
- [x] Existing offline tests pass.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/statistics-chart-keys-unordered-json]]
- [[../dogfooding/session-2026-06-01]]
