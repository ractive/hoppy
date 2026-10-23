---
title: Core (pull zone) enum labels disagree with the spec
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: iter-90
tags:
  - core
  - api-drift
  - dx
---

# Core enum labels disagree with the spec

Found by the iter-90 scan of every `forward_compat_repr_enum!` against
`specs/*.json` (`x-enum-varnames` / description). Shield was fixed in
[[iterations/iteration-90-shield-enum-labels]]. These are not fixed because
there is no live ground truth yet (the API does not publish enum tables for
them) and they live in pull-zone code.

Each row was checked against the schema that the pull-zone / DNS models
actually reference (`PullZoneModel`, `PullZoneSettingsModel`,
`PullZoneAddModel`, `DnsRecordModel` in `specs/core-platform.json`).

| Enum (`core/types.rs`) | Spec schema (`core-platform.json`) | hoppy |
|---|---|---|
| `OptimizerWatermarkPosition` | `OptimizerWatermarkPosition`: 0 BottomLeft, 1 BottomRight, 2 TopLeft, 3 TopRight, 4 Center, 5 CenterStretch | 0 TopLeft, 1 TopRight, 2 BottomLeft, 3 BottomRight, 4 Center (no 5) |
| `PreloadingScreenTheme` | `PreloadingScreenTheme`: 0 Light, 1 Dark (also the field description; same since the first spec commit) | 0 Dark, 1 Light; its doc comment wrongly says the spec agrees |
| `StickySessionType` | `StickySessionType` (the pull-zone one; the load balancer has a separate `LoadBalancerStickySessionType`): 0 Off, 1 On | 0 None, 1 Cookie |
| `DnsMonitoringStatus` | `DnsMonitoringStatus` (`DnsRecordModel.MonitorStatus`): 0 Unknown, 1 Online, 2 Offline | 0 Undetermined (iter-87 renamed it from `Unknown`) |

**`OriginType` is not a mismatch.** The spec schema named `OriginType`
(0 Standard, 1 Storage, 2 EdgeScript, 3 MagicContainer) belongs to the *load
balancer* origin models (`LoadBalancerOriginModel`). Pull zones reference
`PullZoneOriginType`: 0 OriginUrl, 1 DnsAccelerate, 2 StorageZone,
3 LoadBalancer, 4 EdgeScript, 5 MagicContainers, 6 PushZone, which hoppy's
`OriginType` matches by number (confirmed live: storage-zone pull zones are 2,
Magic Containers are 5). Only two spellings differ: hoppy `Script` vs spec
`EdgeScript`, and hoppy `MagicContainerEndpoint` vs spec `MagicContainers`.
**Do not apply the load balancer `OriginType` schema to pull zones.**

A new pull zone returns `OptimizerWatermarkPosition: 0` and
`PreloadingScreenTheme: 0` by default; compare with the dashboard defaults to
decide which table is right before changing the three label tables above.

- [ ] Establish ground truth (dashboard / API behaviour) per enum
- [ ] Apply the `repr_enum_names!` pattern from iter-90 to the confirmed ones (and optionally the two `OriginType` spellings)
- [ ] Extend the spec-sync unit test to `core-platform.json`
