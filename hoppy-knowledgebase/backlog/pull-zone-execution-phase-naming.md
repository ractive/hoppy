---
title: EdgeScriptExecutionPhase is misnamed (spec Cache/LoadBalancer/PreCache) and lacks value 2
type: backlog
date: 2026-10-23
status: completed
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - pull-zone
  - edge-script
  - api-drift
resolved-in: iter-87
---

# Pull-zone `EdgeScriptExecutionPhase` drift

The spec field `EdgeScriptExecutionPhase` references the schema
`ExecutionPhase`: `0 = Cache`, `1 = LoadBalancer`, and now `2 = PreCache`.
hoppy models it as `OnRequest = 0` / `OnResponse = 1`
(`crates/bunny-net-api/src/core/types.rs:534`, iter-74), so the CLI names
(`on-request`/`on-response`) do not describe what the API does. Reads of `2`
fall back to `None`, because the field uses `deserialize_repr_option`.

- [x] Rename the variants to `Cache`/`LoadBalancer` and add `PreCache = 2`, keeping the old CLI names as hidden aliases
- [x] Check the bunny docs for the semantics of each phase and update the help text

See [[research/openapi-drift-2026-10-23]].

## Resolution (iter-87, 2026-10-23)

Renamed to `Cache = 0` / `LoadBalancer = 1` / `PreCache = 2`; `on-request` / `on-response` remain hidden CLI aliases for 0 / 1. No pull zone with an attached script existed on the test account and creating one was out of the sanctioned write scope, so semantics come from the bunny docs (scripts run after the cache by default; "before cache" = PreCache). `LoadBalancer` is undocumented. See [[api/bunny-api-quirks]].

See [[iterations/iteration-87-openapi-drift-fixes]].
