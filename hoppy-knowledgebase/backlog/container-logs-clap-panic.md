---
title: container logs panics with clap downcast mismatch
type: backlog
date: 2026-05-10
status: planned
priority: critical
origin: dogfooding-2026-05-10
---

# `hoppy container logs` always panics

Every invocation of `hoppy container logs --app-id <id>` (with or without
`--tunnel none`, `--local-port`, etc.) panics in clap before doing any work:

```
thread 'main' panicked at clap_builder-4.6.0/src/parser/error.rs:32:
Mismatch between definition and access of `format`. Could not downcast
to TypeId(0xeb5f...), need to downcast to TypeId(0xb9f4...).
```

`--help` works. The crash happens during arg parsing.

## Likely cause

`src/cli.rs:2236` declares the per-subcommand `format: String` field with
`value_parser = ["text", "json"]`, while the global `--format` is parsed as
the `OutputFormat` enum elsewhere. clap stores both under the same id but
the runtime tries to downcast through one type and the value was inserted
as the other → panic on every parse.

The sibling `event-logs`/stream/db commands that override `--format` semantics
do so without colliding because they avoid the global flag; the `Logs`
variant inherits the global and re-declares it.

## Fix sketch

- Rename the local field (e.g. `tail_format`) and read it from a different
  CLI flag, or
- Make `--format` use a single shared enum across the binary, or
- Drop the per-command `--format` override on `container logs` and validate
  the global value at the start of the handler instead.

## Repro

```sh
export BUNNY_API_KEY=<test key>
./target/release/hoppy container logs --app-id <some-app-id> --tunnel none
# panics immediately
```

This is iter-24 territory — `container logs` is its headline feature and
is currently entirely unusable.
