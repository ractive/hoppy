---
title: inconsistent flag names across nouns (storage, stream, db)
type: backlog
date: 2026-05-10
status: planned
priority: medium
origin: dogfooding-2026-05-10
---

# Flag-name inconsistencies trip up the user

Patterns found during dogfooding:

## storage commands

| Subcommand | Path flag      | Local file flag |
|------------|----------------|-----------------|
| upload     | `--remote-path`| `--file`        |
| download   | `--remote-path`| `--output`      |
| ls         | `--path`       | —               |
| rm         | `--remote-path`| —               |

`ls` is the odd one out (`--path` vs `--remote-path`). And `upload --file`
vs `download --output` is asymmetric — both refer to a local file, just
input vs output direction.

Suggestion: standardise on `--remote-path` everywhere and use `--local-path`
(or `--src` / `--dst`) consistently.

## stream library

`stream library list/get/update/statistics` use `--library-id`, but
`stream library delete` uses `--id`. Hit during cleanup.

Suggestion: use `--id` everywhere (matches pull-zone, dns zone, db, etc.).

## Why this matters

Each minor difference forces the user to re-read `--help`. The cost of
fixing flag names is one breaking change in a pre-1.0 release; the cost
of leaving them is paid by every future user.
