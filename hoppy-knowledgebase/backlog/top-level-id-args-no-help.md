---
title: "14 top-level get/delete/statistics commands ship `--id <ID>` with no help text"
type: backlog
date: 2026-05-30
status: planned
priority: low
origin: dogfooding-2026-05-30 (post-iter-42)
---

# Top-level `--id` args lack help description

iter-41 §3 and iter-42 §1 fixed `help = "..."` text on **sub-resource**
commands (where the parent's ID is ambiguous). They never touched the
**top-level** `get`/`delete`/`statistics` commands. Sweeping every such
command turned up 14 with `--id <ID>` rendered as a bare arg with no
description:

```
pull-zone get        pull-zone delete        pull-zone statistics
container app get    container app delete    container app statistics
dns zone get         dns zone delete
stream library get   stream library delete   stream library statistics
stream video list
script get           script delete
```

Sample output (`hoppy pull-zone get --help`):

```
Options:
      --format <FORMAT>   Output format [default: table] [possible values: ...]
      --id <ID>           <-- empty
      --debug             Enable debug output (shows HTTP requests)
```

The meaning is obvious from context — `--id` on `pull-zone get` is the
pull-zone id — so this isn't a hard blocker. But:

1. The empty line *looks* broken next to its peers, which all have
   descriptions.
2. A user comparing `--help` between sibling commands can't tell whether
   the empty description means "intentionally generic" or "TODO".
3. The fix is cheap and mechanical.

## Fix

Add `help = "<Resource> ID"` (singular, the resource itself) to the `--id`
arg on each command. Examples:

```rust
// crates/hoppy-cli/src/cli/pull_zone.rs
#[arg(long, help = "Pull zone ID")]
id: u64,
```

Repeat for `pull-zone delete/statistics`, `container app *`,
`dns zone *`, `stream library *`, `stream video list`, `script *`.

Then re-run the sweep below to confirm nothing else escaped:

```sh
for spec in "pull-zone get" "pull-zone delete" ...; do
  line=$(eval "hoppy $spec --help" 2>/dev/null | grep -E "^\s+--id\s")
  if echo "$line" | grep -qE "<ID>\s*$"; then echo "MISSING: $spec"; fi
done
```

## Out of scope

- Sub-resource `--id` args (already done in iter-41 §3 + iter-42 §1).
- The `--<noun>-id` legacy aliases iter-42 §2 added — leave those.
