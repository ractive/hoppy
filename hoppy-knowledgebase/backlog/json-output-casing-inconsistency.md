---
title: --format json output uses three different casing conventions
type: backlog
date: 2026-05-15
status: open
tags:
  - cli
  - json
  - dx
  - scripting
---

# JSON output casing is inconsistent across domains

Running `hoppy <noun> list --format json` produces three different shapes
depending on the domain:

| Surface                  | Wrapper key | Field casing | Pagination meta |
|--------------------------|-------------|--------------|------------------|
| `pull-zone list`         | `Items`     | PascalCase (`Id`, `Name`) | none |
| `storage-zone list`      | `Items`     | PascalCase (`Id`, `Name`) | none |
| `dns zone list`          | `Items`     | PascalCase (`Id`, `Name`) | none |
| `statistics`             | (object)    | PascalCase | none |
| `shield zone list`       | `Items`     | camelCase (`shieldZoneId`, `wafEnabled`) | none |
| `container app list`     | `items`     | camelCase (`id`, `name`)  | `cursor`, `meta` |

This is pass-through from the underlying bunny.net APIs — each one
serializes its own way. But the user-facing CLI has the chance to
normalize.

## Why it bites

- A jq filter that works for pull-zone (`jq '.Items[].Name'`) fails
  silently against containers (`jq '.items[].name'`).
- Pipelines and scripts need a per-domain casing lookup table.
- The dogfooding session (2026-05-15) needed three different jq
  invocations just to count leaked resources by prefix.

## Options

1. **Normalize at the CLI layer** — always lowercase the wrapper to
   `items`, always snake_case or camelCase field names. Breaking.
2. **Add a `--flat` mode** — emit one JSON object per resource on
   stdout (NDJSON), with a stable field naming convention. Non-breaking.
3. **Document the table above in the README** so users at least know
   what to expect. Cheapest, but doesn't fix scripting pain.
