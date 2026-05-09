---
title: Add drill-down hints to CLI output (hyalo iter-107 pattern)
type: backlog
date: 2026-05-09
status: planned
priority: medium
origin: iter-23 inspiration scan of ../hyalo git log
---

# Drill-down hints

Hyalo's iter-107 ("Add drill-down hints for lint/types, sync all documentation surfaces") added next-step suggestions after every command. Example:

```
$ hyalo find --property status=planned
... results ...

tip: hyalo find --task todo            # find files with open tasks
tip: hyalo find --orphan               # 1 orphan file
tip: hyalo find --dead-end             # 8 dead-end files
```

Hoppy doesn't have this. Adopting it would help **both** humans (next-step discoverability) and LLMs (chained tool use without hallucination).

## Concrete proposals

- After `hoppy pull-zone list`: tip `hoppy pull-zone get --id <id>` and `hoppy pull-zone statistics --id <id>`
- After `hoppy pull-zone create`: tip `hoppy pull-zone hostname add` and `hoppy pull-zone edge-rule add`
- After `hoppy stream library list`: tip `hoppy stream video list --library <id>`
- After `hoppy container app create`: tip `hoppy container template add --app-id <id>` and `hoppy container endpoint add --app-id <id>`
- After `hoppy auth login`: tip `hoppy pull-zone list` (read-only smoke test)

Implementation: a `--no-hints` global flag (default off) suppresses them. `--format json` already implies `--no-hints` so JSON output stays parseable.

## Reference

- `../hyalo` iter-107 commit `d28325d`
- [[../cli/help-text-style]]
