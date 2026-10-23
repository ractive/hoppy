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

| Enum (`core/types.rs`) | Spec (`core-platform.json`) | hoppy |
|---|---|---|
| `OptimizerWatermarkPosition` | 0 BottomLeft, 1 BottomRight, 2 TopLeft, 3 TopRight, 4 Center, 5 CenterStretch | 0 TopLeft, 1 TopRight, 2 BottomLeft, 3 BottomRight, 4 Center |
| `PreloadingScreenTheme` | 0 Light, 1 Dark (same since the first spec commit) | 0 Dark, 1 Light, doc claims spec agrees |
| `OriginType` | 0 Standard, 1 Storage, 2 EdgeScript, 3 MagicContainer | 0 OriginUrl, 1 DnsAccelerate, 2 StorageZone, 3 LoadBalancer, 4 Script, 5 MagicContainerEndpoint, 6 PushZone |
| `StickySessionType` | 0 Off, 1 On | 0 None, 1 Cookie |
| `DnsMonitoringStatus` | 0 Unknown | 0 Undetermined |

A new pull zone returns `OptimizerWatermarkPosition: 0` and
`PreloadingScreenTheme: 0` by default; compare with the dashboard defaults to
decide which table is right. `OriginType` may be a different schema than the
pull-zone one in practice (hoppy has seen values 4-6 live), so check before
touching it.

- [ ] Establish ground truth (dashboard / API behaviour) per enum
- [ ] Apply the `repr_enum_names!` pattern from iter-90 to the confirmed ones
- [ ] Extend the spec-sync unit test to `core-platform.json`
