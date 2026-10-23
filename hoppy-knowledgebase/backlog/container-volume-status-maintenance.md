---
title: container volume list fails when a volume is in "maintenance" status
type: backlog
date: 2026-10-23
status: planned
priority: high
origin: openapi-drift-2026-10-23
tags:
  - containers
  - volumes
  - api-drift
  - deserialization
---

# Magic Containers `VolumeStatus` gained `maintenance`

**Impact: breaking.** `VolumeInstance.status: VolumeInstanceStatus`
(`crates/bunny-net-api/src/containers/types.rs:981`, enum at :218) is a closed
camelCase string enum with no `#[serde(other)]` variant. Any volume that bunny
puts into maintenance fails the whole volume list or get response. The user
cannot avoid this, because bunny controls the state.

- [ ] Add `Maintenance`
- [ ] Add `#[serde(other)] Unknown` handling for the string status enums on MC response types (`ApplicationStatus`, `DeploymentStatus`, `PodStatus`, `ContainerStatus` and others), as a forward-compat audit
- [ ] Add a unit test for an unknown status string

See [[research/openapi-drift-2026-10-23]].
