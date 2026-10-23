---
title: container volume list fails when a volume is in "maintenance" status
type: backlog
date: 2026-10-23
status: completed
priority: high
origin: openapi-drift-2026-10-23
tags:
  - containers
  - volumes
  - api-drift
  - deserialization
resolved-in: iter-87
---

# Magic Containers `VolumeStatus` gained `maintenance`

**Impact: breaking.** `VolumeInstance.status: VolumeInstanceStatus`
(`crates/bunny-net-api/src/containers/types.rs:981`, enum at :218) is a closed
camelCase string enum with no `#[serde(other)]` variant. Any volume that bunny
puts into maintenance fails the whole volume list or get response. The user
cannot avoid this, because bunny controls the state.

- [x] Add `Maintenance`
- [x] Add `#[serde(other)] Unknown` handling for the string status enums on MC response types (`ApplicationStatus`, `DeploymentStatus`, `PodStatus`, `ContainerStatus` and others), as a forward-compat audit
- [x] Add a unit test for an unknown status string

See [[research/openapi-drift-2026-10-23]].

## Resolution (iter-87, 2026-10-23)

`VolumeInstanceStatus::Maintenance` added; all container string enums now fall back to `Unknown` via `#[serde(other)]`.

See [[iterations/iteration-87-openapi-drift-fixes]].
