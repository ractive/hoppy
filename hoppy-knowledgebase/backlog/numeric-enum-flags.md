---
title: several flags expose raw numeric enum values
type: backlog
date: 2026-05-10
status: completed
priority: medium
origin: dogfooding-2026-05-10
---

# Raw numeric enum values in user-facing flags

`pull-zone create --zone-tier` got the named-enum treatment in iter-26
(`premium` / `volume`). Other commands still expose the raw bunny.net API
integer values:

| Command                         | Flag             | Raw values shown to user        |
|---------------------------------|------------------|---------------------------------|
| `script create`                 | `--script-type`  | `0 = Dns, 1 = Cdn, 2 = Middleware` |
| `storage-zone create`           | `--zone-tier`    | `0 = Standard, 1 = Edge`        |

Forces the user to read help, look up the integer, and remember it. Mirror
the `pull-zone create --zone-tier` pattern: ValueEnum with the wire-level
ints behind the scenes, named values in the CLI.

This is iter-28 cleanup (or later follow-up) territory.
