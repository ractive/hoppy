---
title: ContainerEndpoint lacks the now-required id field
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - containers
  - endpoints
  - api-drift
---

# Magic Containers `ContainerEndpoint.id`

The fresh MC spec adds `id` (string) to `ContainerEndpoint` and marks it
**required**. `containers::ContainerEndpoint`
(`crates/bunny-net-api/src/containers/types.rs:781`, used in
`ContainerTemplate.endpoints`) does not model it. Only `EndpointListItem`
has `id`. `container app get` therefore cannot show endpoint ids, and users
need an extra `endpoint list` call to address an endpoint.

- [ ] Add `id: Option<String>` (optional for tolerance of older responses)
- [ ] Show the id in `container app get` endpoint output

See [[research/openapi-drift-2026-10-23]].
