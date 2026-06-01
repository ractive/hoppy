---
title: Iter-63 — --all auto-paginate flag sweep across list commands
type: iteration
date: 2026-06-01
tags:
  - iteration
  - cli
  - pagination
  - consistency
status: planned
branch: iter-63/list-all-flag-sweep
---

# Iter-63 — `--all` auto-paginate flag sweep

## Why

`shield event-logs --all` auto-paginates server-side. Every other
paginated `list` command requires manual `--page`/`--per-page`
juggling. The inconsistency means scripts have to special-case
each surface.

See [[../backlog/pull-zone-list-missing-all-flag]].

## Scope

### 1. Audit paginated list commands [0/2]

- [ ] Identify every `list` command that paginates. Initial list:
      `pull-zone list`, `storage-zone list`, `dns zone list`,
      `dns record list`, `stream library list`, `stream video list`,
      `stream collection list`, `container app list`,
      `container endpoint list`, `container volume list`,
      `db list`, `db v2 list`, `db group list`, `shield zone list`,
      `shield access-list list`, `shield waf list-rules`. Add any
      missing.
- [ ] Note which already auto-paginate (none, ideally — confirm).

### 2. Implement [0/3]

- [ ] Add a shared `--all` option helper (or extend the existing
      `Pagination` option struct) so the same flag wires into every
      paginated list.
- [ ] Implement the auto-pagination loop: walk pages server-side
      until the API signals no more (`HasMoreItems: false`,
      `continuationToken: null`, or response < `per_page`).
- [ ] Decide and document interaction with `--page`/`--per-page`:
      either mutually exclusive (clap arg group) or `--all`
      silently overrides.

### 3. Tests [0/3]

- [ ] E2E mock test: one representative list command (e.g.
      `pull-zone list --all`) collects across 3 mock pages.
- [ ] E2E test that `--all` works under `--format json`, `--format
      table`, `--format text` — rows from subsequent pages append
      cleanly.
- [ ] Regression test: existing `--page`/`--per-page` paths
      unchanged.

## Out of scope

- Cursor-based pagination on endpoints that don't already expose it.
- Combining `--all` with `--filter`/`--query` flags that don't yet
  exist.

## Acceptance Criteria

- [ ] Every paginated `list` command exposes `--all` with uniform
      semantics.
- [ ] `--all` works across all three output formats.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/pull-zone-list-missing-all-flag]]
- [[../dogfooding/session-2026-06-01-round2]]
