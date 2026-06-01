---
title: dns zone export emits literally nothing for empty zones
type: backlog
date: 2026-06-01
status: planned
priority: low
origin: dogfooding-2026-06-01-round2
tags:
  - cli
  - dns
  - polish
---

# Empty zones produce no output at all

```sh
$ hoppy dns zone create --domain hoppy-test.example
$ hoppy dns zone export --id <z>
$ # nothing — no header, no comment, no newline
```

Users running `hoppy dns zone export | grep ...` against an empty
zone get a silent no-op with exit 0. There's no signal that the
command ran successfully on an empty zone vs. the binary just
hanging.

## Fix

For an empty zone, emit at minimum a single header comment:

```
;; zone hoppy-test.example — 0 records
```

…or even just `;; empty zone\n` — anything that lands a non-zero
byte on stdout to confirm execution.

If the [[dns-zone-export-ignores-format]] fix lands first and
introduces a JSON shape, an empty zone in JSON becomes
`{"Bind": ""}` or `{"Records": []}` — both fine.

## Acceptance

- `dns zone export --id <z>` on an empty zone produces non-empty
  stdout.
- The empty-zone output is a no-op for downstream BIND parsers
  (just comments / blank lines).
