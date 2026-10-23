---
title: container get --format json drops the endpoints' id
type: backlog
date: 2026-10-21
status: completed
priority: medium
origin: dogfooding-2026-10-21
---

# `hoppy container get --format json` has no endpoint `id`

`hoppy 0.7.0 (1eddb33a8515 2026-08-30)`: `hoppy container get --id dzEp04xZWTD7F3A --format json` lists each
container template's `endpoints` with `displayName`, `publicHost`, `type`,
`pullZoneId`, `portMappings` …, but without `id`. The raw
`GET /mc/apps/{id}` has it (`"id": "dzEp04xZWTD7F3A-relay-cdn-oZwIq0m7CF"`).

The endpoint id is what pull zones reference (`MagicContainersEndpointId` in
`pull-zone get`) and what OpenTofu's `bunnynet_compute_container_app` tracks, so
checking that an app change kept its endpoint (and with it the custom hostname)
needs it. Here it took a raw API call.

## Fix sketch

Keep `id` in the endpoint model (probably missing from the serde struct, so
it is dropped on the way through), in JSON and as a column in the table.
