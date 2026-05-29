---
title: "`pull-zone edge-rule list --id <ID>` and several sub-resource args have no help text"
type: backlog
date: 2026-05-28
status: resolved
priority: low
origin: dogfooding-2026-05-28 (post-iter-40)
---

# Sub-resource `--id` args lack `--help` description

`hoppy pull-zone edge-rule list --help` shows:

```
      --id <ID>
```

…with no description line. A user has to guess whether `--id` refers to
the pull zone or the edge rule.

In sub-resource commands the `--id` convention is "ID of the parent
resource"; the rule itself is identified by other args (`--rule-id` for
update/delete). Document this in the arg's `help` attribute.

## Fix

Audit every clap `#[arg(long)]` that is just `--id <ID>` with no `help =`:

```sh
grep -rn -B1 -A3 'long.*help\s*=' crates/hoppy-cli/src/cli/ \
  | grep -B3 '^\s*--id <ID>'
```

For each, add a `help = "<context>"` attribute. Examples:

- `pull-zone edge-rule list`: `help = "Pull zone ID"`
- `pull-zone hostname add`: `help = "Pull zone ID"`
- `container endpoint list`: `help = "Container app ID"`
- `dns record list`: `help = "DNS zone ID"`

## Out of scope

- The `--id` vs `--<noun>-id` decision itself (settled by iter-40 #2).
  This is purely a help-text gap.
