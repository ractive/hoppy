---
title: Iter-63 — --all auto-paginate flag sweep across list commands
type: iteration
date: 2026-06-01
tags:
  - iteration
  - cli
  - pagination
  - consistency
status: completed
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

### 1. Audit paginated list commands [2/2]

- [x] Identify every `list` command that paginates. Initial list:
      `pull-zone list`, `storage-zone list`, `dns zone list`,
      `dns record list`, `stream library list`, `stream video list`,
      `stream collection list`, `container app list`,
      `container endpoint list`, `container volume list`,
      `db list`, `db v2 list`, `db group list`, `shield zone list`,
      `shield access-list list`, `shield waf list-rules`. Add any
      missing.
- [x] Note which already auto-paginate (none, ideally — confirm).

### 2. Implement [3/3]

- [x] Add a shared `--all` option helper (or extend the existing
      `Pagination` option struct) so the same flag wires into every
      paginated list.
- [x] Implement the auto-pagination loop: walk pages server-side
      until the API signals no more (`HasMoreItems: false`,
      `continuationToken: null`, or response < `per_page`).
- [x] Decide and document interaction with `--page`/`--per-page`:
      either mutually exclusive (clap arg group) or `--all`
      silently overrides.

### 3. Tests [3/3]

- [x] E2E mock test: one representative list command (e.g.
      `pull-zone list --all`) collects across 3 mock pages.
- [x] E2E test that `--all` works under `--format json`, `--format
      table`, `--format text` — rows from subsequent pages append
      cleanly.
- [x] Regression test: existing `--page`/`--per-page` paths
      unchanged.

## Out of scope

- Cursor-based pagination on endpoints that don't already expose it.
- Combining `--all` with `--filter`/`--query` flags that don't yet
  exist.

## Acceptance Criteria

- [x] Every paginated `list` command exposes `--all` with uniform
      semantics.
- [x] `--all` works across all three output formats.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Status Notes

**Surfaces that got `--all`** (page/per_page offset-based):
- `pull-zone list`
- `storage-zone list`
- `dns zone list`
- `stream library list`
- `stream video list`
- `stream collection list`
- `script list`
- `script release list`
- `db v2 list`

**Surfaces that got `--all`** (cursor-based):
- `container app list` (and `container list` alias)
- `container region list`
- `container node list`

**Skipped — API does not paginate (returns all results in one call):**
- `dns record list` — no page/per_page params in API or CLI
- `db list` (v1) — returns `{"databases": [...]}` with no pagination
- `db group list` — returns `{"groups": [...]}` with no pagination
- `shield zone list` — returns flat list, no pagination
- `shield access-list list` — returns all access lists, no pagination
- `shield waf list-rules` — returns all custom WAF rules, no pagination
- `shield rate-limit list` — returns all rate limit rules, no pagination
- `container endpoint list` — returns all endpoints for an app, no pagination
- `container volume list` — returns all volumes for an app, no pagination

**Decision**: `--all` is mutually exclusive with `--page`/`--per-page`/`--cursor`/`--limit` via `conflicts_with` in clap. This produces a clear error message rather than silently overriding.

## Related

- [[../backlog/pull-zone-list-missing-all-flag]]
- [[../dogfooding/session-2026-06-01-round2]]
