---
title: "list-table commands overflow on long text columns (e.g. `shield waf profiles` = 267 chars)"
type: backlog
date: 2026-05-28
status: planned
priority: low
origin: dogfooding-2026-05-28 (post-iter-40)
---

# Long-text columns in `list` tables blow out width

iter-39/iter-40 addressed `get` table width (vertical pivot + nested-JSON
summary cells). The same width problem exists in `list` tables when one
column has long free-text values:

| Command | Width |
|---|---|
| `shield waf profiles` | 267 chars (Description column has multi-sentence prose) |

Other likely offenders: `shield rate-limit list` (rule descriptions),
`pull-zone edge-rule list` (action-parameters JSON), `dns record list`
(value column for TXT records can be arbitrarily long).

`list` is the wide-by-design shape so the iter-40 pivot doesn't apply,
but unbounded long-text columns still ruin readability.

## Suggested fix

1. **Truncate long text cells** in table mode at e.g. 60 chars with `…`,
   with a stderr `tip: --format json …` redirect.
2. Or **omit verbose columns by default** in table mode and add a
   `--verbose` flag to re-enable (Description is rarely the column you
   scan a list for).
3. Or **respect `$COLUMNS`** and dynamically resize column widths to fit;
   `tabled` has helpers for this.

(2) is the most conservative — the user can scan IDs / names quickly,
then use `get --id <id>` to see full text. (3) is the most polished
but the riskiest in terms of test stability.

## Out of scope

- The iter-40 fix for `get` pivots — that's the right shape for single
  resources. This is the symmetric list-side problem.
