---
title: ContainerEndpoint lacks the now-required id field
type: backlog
date: 2026-10-23
status: superseded
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - containers
  - endpoints
  - api-drift
resolved-in: "PR #104 (iter-86)"
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

## Resolution

Superseded by PR #104 ([[iterations/iteration-86-dogfood-oct-fixes]]), which
models `ContainerEndpoint.id`. It is required there, following the current
spec, rather than the `Option` proposed above. PR #104 also shows the
endpoint ids in `container get`.
