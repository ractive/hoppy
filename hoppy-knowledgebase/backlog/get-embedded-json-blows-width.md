---
title: "single-resource `get` tables still overflow when a field value is JSON-stringified"
type: backlog
date: 2026-05-27
status: planned
priority: medium
origin: dogfooding-2026-05-27 (post-iter-39)
---

# Embedded JSON values still blow out `get` table width

Iter-39 pivoted single-resource `get` commands from a wide row layout to a
vertical Field/Value table. The pivot is correct, but the Value column
inherits any field whose JSON value is an object or array — and `tabled`
renders those as inline single-line JSON. One huge value forces the
whole column to that width.

Measured first-line widths (= row separator length = total table width):

| Command | Width | Cause |
|---|---|---|
| `storage-zone get` | 70 chars | only scalar fields ✓ |
| `pull-zone get` | 176 chars | `Hostnames` rendered as `{"Id":...,"Value":"...","ForceSsl":...,...}` |
| `container app get` | 922 chars | `repositorySettings`, `regionSettings`, `volumes` are nested objects/arrays |

176 chars wraps unreadably on a 120-col terminal; 922 chars is
catastrophic on any terminal.

## Suggested fixes (any one helps)

1. **Render arrays/objects as `<3 items>` / `<object: 5 fields>` summaries**
   in table mode, with a hint:
   `tip: hoppy --format json container app get --id <id> | jq .repositorySettings`
2. **Split nested collections into sub-tables** rendered below the main
   `Field / Value` block (one sub-table per array/object field).
3. **Cap the Value column width** at e.g. `terminal_width - 30` and
   truncate with `…`. Lossy but unblocks readability.

Option 1 is the cheapest and most defensible — table format is for
humans, JSON is for scripts. Don't try to humanise nested structure;
direct the user to `--format json` for that.

## Out of scope

- Re-flowing the Field/Value layout itself; iter-39's choice stands.
- Container app's separate `endpoint list` / `region show` subcommands
  already handle some of these views.
