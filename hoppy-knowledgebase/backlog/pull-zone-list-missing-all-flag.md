---
title: pull-zone list missing --all auto-paginate flag
type: backlog
date: 2026-06-01
status: planned
priority: low
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - pagination
  - consistency
---

# `pull-zone list` (and siblings) lack `--all` while `shield event-logs` has it

`shield event-logs` exposes `--all` to auto-paginate:

```sh
hoppy shield event-logs --id <z> --date <d> --all
```

But `pull-zone list` (and most other `list` commands) only have
`--page`/`--per-page` and require manual paging:

```sh
hoppy pull-zone list --page 1 --per-page 100
hoppy pull-zone list --page 2 --per-page 100
...
```

## Fix

Sweep every paginated `list` command and add a uniform `--all`
flag that auto-paginates server-side until the response indicates
no more pages. Same flag name + semantics as `shield event-logs`
so muscle memory transfers.

Surfaces to audit (non-exhaustive):

- `pull-zone list`
- `storage-zone list`
- `dns zone list`
- `dns record list`
- `stream library list`
- `stream video list`
- `stream collection list`
- `container app list`
- `container endpoint list`
- `container volume list`
- `db list`, `db v2 list`
- `db group list`
- `shield zone list`
- `shield access-list list`
- `shield waf list-rules`

## Acceptance

- Every `list` command that paginates exposes `--all` with the same
  semantics as `shield event-logs --all`.
- `--all` is mutually exclusive with `--page`/`--per-page` (or
  silently overrides them, your call — document either way).
- `--all` works under `--format json`, `--format table`, and
  `--format text` (the rows of subsequent pages append cleanly).
