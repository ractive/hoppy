---
title: >-
  table column labels are Title-Case while JSON keys are camelCase — mappings
  non-obvious (e.g. `Premium` → `isPremium`)
type: backlog
date: 2026-05-29
status: resolved
priority: medium
origin: dogfooding-2026-05-29 (post-iter-41)
resolved-by: "[[iterations/iteration-42-dogfooding-2026-05-29-fixes]]"
---

# Table-label vs JSON-key naming mismatch

`hoppy shield waf profiles` (and likely other commands sourcing from the
camelCase containers/shield APIs) renders table headers in Title-Case
while the JSON output uses camelCase. The mappings are non-obvious:

| Table column | JSON key |
|---|---|
| `Description` | `description` |
| `Category` | `profileCategory` |
| `Premium` | `isPremium` |
| `Name` | `name` |
| `ID` | `id` |

The iter-41 truncation tip even mentions a column name:
`tip: some Description values were truncated — use --format json for full values`

A user querying `.Description` would get null; they need `.description`.
Worse, querying `.Category` or `.Premium` would also miss — the JSON keys
are `profileCategory` and `isPremium`.

## Suggested fix — pick one option

1. **Tip text references the JSON key** when it differs from the column
   label. E.g.: `tip: ... use --format json (key: .description)`. Cheap
   and immediately useful.
2. **Render table labels in the JSON-key shape** (camelCase or snake-case)
   so the user only ever has to remember one name per field.
3. **Surface a `--field-map` view** (`hoppy schema show shield waf profiles`)
   so the user can see "Table column ↔ JSON key" for any command.

(1) is the cheapest unblock; (2) is the most consistent long-term answer.

## Out of scope

Whether the underlying bunny.net APIs use camelCase or PascalCase is up
to them — this is purely about how hoppy presents them.
