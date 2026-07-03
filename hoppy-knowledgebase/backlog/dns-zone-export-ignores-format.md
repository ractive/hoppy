---
title: dns zone export ignores --format and always emits raw BIND
type: backlog
date: 2026-06-01
status: resolved
priority: medium
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - dns
  - format
  - consistency
---

# `dns zone export --format json` ignores `--format`

```sh
$ hoppy dns zone export --id <z> --format json
;A records
www.example.com.    IN    5m    A    1.2.3.4

;TXT records
example.com.    IN    5m    TXT    "hello world"
```

The `--format json` flag is silently ignored — the BIND zone file
text comes through verbatim. Same goes for `--format table` (which
arguably *should* render the BIND text as-is, since it's already
human-readable, but the JSON path needs a wrapper).

## Fix

Decide the JSON shape and wire it through the standard `--format`
pipeline. Two reasonable options:

1. **Envelope** — `{"Bind": "<full BIND text>"}` — minimal, lossy
   for downstream consumers but easy to emit.
2. **Structured** — `{"Records": [{"Name": "...", "Type": "A", "Value": "1.2.3.4", "Ttl": 300}, …]}` — pre-parsed for jq/scripts.
   Lossier in the round-trip-fidelity sense but more useful.

Recommended: **(1)** to start, with `--format text` aliased to the
current raw output and `--format table` rendering a per-record table
sourced from `dns record list` so the BIND text is only emitted under
the dedicated raw mode.

## Related

- [[dns-zone-export-empty-zone-silent]] — sibling polish item.
- [[db-config-show-limits-ignore-format]] — same shape, different
  surface.

## Acceptance

- `dns zone export --format json` emits valid JSON.
- `dns zone export --format table` renders a per-record table.
- `dns zone export --format text` (the new default for BIND) keeps
  the existing raw output. Decide which name maps to which mode
  based on least-surprise.
