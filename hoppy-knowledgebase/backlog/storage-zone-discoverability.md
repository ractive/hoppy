---
title: "`storage zone list` returns 'unrecognized subcommand' — should suggest `storage-zone`"
type: backlog
date: 2026-05-26
status: planned
priority: low
origin: dogfooding-2026-05-26 (post-iter-38)
---

# `storage zone list` → no useful suggestion

Storage zone management lives under the top-level `storage-zone` (hyphenated)
command, while `storage` is the file-ops surface (`ls`, `rm`, `upload`,
`download`). A user familiar with the rest of the CLI (`hoppy pull-zone
list`, `hoppy dns zone list`, `hoppy shield zone list`) will naturally
type:

```
hoppy storage zone list
```

…and get:

```
error: unrecognized subcommand 'zone'
Usage: hoppy storage [OPTIONS] <COMMAND>
```

No tip. By contrast, `hoppy storage list` (a different typo) *does* offer:

```
  tip: a similar subcommand exists: 'ls'
```

So clap's typo-suggestion is wired in for one shape and not the other.

## Suggested fix

Either:

1. Wire a custom "did you mean" handler at the `storage` subcommand level
   that recognises the `zone <verb>` pattern and points to
   `hoppy storage-zone <verb>`, **or**
2. Alias `storage zone` → `storage-zone` (clap subcommand alias) so the
   space form just works.

Option 2 is the least surprising — it matches the `pull-zone` /
`dns zone` / `shield zone` cadence and keeps the existing top-level
`storage-zone` command intact for backward compat.

## Related

- iter-35 added drill-down hints (`tip: <next>`) for the success path —
  this is the same UX problem on the error path.
