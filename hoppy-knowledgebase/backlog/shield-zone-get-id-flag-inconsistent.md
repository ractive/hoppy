---
title: "`shield zone get` uses `--shield-zone-id` instead of `--id` like every other surface"
type: backlog
date: 2026-05-27
status: planned
priority: low
origin: dogfooding-2026-05-27 (post-iter-39)
---

# `shield zone get` flag name diverges from convention

Every other `get` command in hoppy takes the resource identifier as
`--id`:

```
hoppy pull-zone     get --id <id>
hoppy storage-zone  get --id <id>
hoppy container app get --id <id>
hoppy dns zone      get --id <id>
hoppy stream library get --id <id>
```

But:

```
hoppy shield zone get --id 123
error: unexpected argument '--id' found
  tip: a similar argument exists: '--shield-zone-id'
```

Clap's "did you mean" suggestion catches it, so this isn't a hard
blocker — just unnecessary friction for muscle memory.

## Fix

Rename the argument to `--id` in `crates/hoppy-cli/src/cli/shield.rs` (or
wherever the subcommand is defined). Keep `--shield-zone-id` as a clap
alias for back-compat for a release or two, then drop.

## Audit

Grep the CLI for other resource-id args that use the long
`--<noun>-id` form when `--id` would do:

```sh
grep -rn '#\[arg(long' crates/hoppy-cli/src/cli/ | grep -E '\-\-\w+\-id'
```
