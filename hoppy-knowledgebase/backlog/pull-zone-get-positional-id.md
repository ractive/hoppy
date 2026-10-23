---
title: pull-zone get takes the id only as --id, not positionally
type: backlog
date: 2026-10-21
status: completed
priority: low
origin: dogfooding-2026-10-21
---

# `hoppy pull-zone get 6442567` is rejected

`hoppy 0.7.0 (1eddb33a8515 2026-08-30)`: `hoppy pull-zone get 6442567` fails with
"error: unexpected argument '6442567' found", usage
`hoppy pull-zone get [OPTIONS] --id <ID>`. The id is the one required value,
and `hoppy search` prints ids to paste, so the positional form is the first
thing one types.

## Fix sketch

Accept the id positionally as well as with `--id` (clap: `#[arg(value_name = "ID",
required_unless_present = "id")]` beside the flag), for `get` and the other
single-resource commands that take only an id.
