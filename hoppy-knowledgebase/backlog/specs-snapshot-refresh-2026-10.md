---
title: Refresh specs/ snapshot to the 2026-10-23 bunny.net specs
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - openapi
  - specs
  - api-drift
---

# Refresh `specs/`

`specs/` dates from iter-66 (2026-07-10, commit `339adce`). Six of the nine
specs have changed since then: core-platform, shield, stream (1.6.12),
magic-containers, database (0.0.144) and edge-scripting. A tenth spec,
Partner Portal, is new. Storage, logging and origin-errors are unchanged.

Per the iter-66 precedent, commit the refresh as step 1 of the drift-fix
iteration that also lands the type fixes, so that the code comments
referencing `specs/*.json` versions stay truthful (for example,
`database/types.rs:3` cites 0.0.130).

- [ ] Download from the canonical URLs listed in [[research/openapi-drift-2026-10-23]]
- [ ] Add `specs/partner-portal.json`
- [ ] Update the version references in module docs

See [[research/openapi-drift-2026-10-23]].
