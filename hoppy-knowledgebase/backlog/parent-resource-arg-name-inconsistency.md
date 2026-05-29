---
title: "sub-resource commands use three different arg names for parent ID (`--id`, `--app-id`, `--zone-id`)"
type: backlog
date: 2026-05-29
status: planned
priority: low
origin: dogfooding-2026-05-29 (post-iter-41)
---

# Parent-resource arg naming diverges across surfaces

For the same concept ("identify the parent resource I want to operate
on"), different sub-resource commands use different argument names:

| Sub-resource command | Parent arg |
|---|---|
| `pull-zone edge-rule list` | `--id` |
| `pull-zone hostname add` | `--id` |
| `container endpoint list` | `--app-id` |
| `dns record list` | `--zone-id` |

A user who has just learned `hoppy pull-zone edge-rule list --id <pz>`
will try `hoppy container endpoint list --id <app>` and get "unexpected
argument". They have to read `--help` every time they cross a surface
boundary.

## Suggested fix — pick one convention and add aliases

Two viable conventions:

**Option A — `--id` everywhere** (matches the top-level `get` pattern):
- Rename `--app-id` → `--id` (alias `--app-id`).
- Rename `--zone-id` → `--id` (alias `--zone-id`).
- Pros: muscle memory across surfaces.
- Cons: `--id` is ambiguous in commands that also identify a sub-resource
  by id (`hoppy pull-zone edge-rule update --id <pz?> --rule-id <r>` —
  the second arg has to disambiguate).

**Option B — `--<parent>-id` everywhere**:
- Rename `--id` → `--pullzone-id` on `pull-zone edge-rule list`,
  `pull-zone hostname add`, etc.
- Keep `--id` as alias for back-compat.
- Pros: unambiguous, self-documenting.
- Cons: more typing.

Option A is the path of least change and matches iter-40's `--id`
unification on the top-level `get` commands. Option B is more explicit
but adds friction.

## Related

- [[../iterations/iteration-40-dogfooding-2026-05-27-fixes]] §2 — chose
  `--id` for top-level commands; this is the symmetric sub-resource
  decision.
- [[iter-41-sub-resource-help-incomplete]] — the help-text gap on
  these args is tracked separately.
