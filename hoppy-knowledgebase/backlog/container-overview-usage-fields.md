---
title: MC overview limits and per-pod usage fields not modeled
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - containers
  - api-coverage
---

# Magic Containers overview and limits additions

- `Overview.limits` (`OverviewLimits { cpuCores, ramBytes, outboundBandwidthBytesPerSecond }`)
- `OverviewPod`: `cpuUsageCores`, `ramUsageBytes`, `outboundBandwidthBytesPerSecond`, `volumes[]` (`OverviewPodVolume`)
- `UserLimits`: `maxCpuMilliCoresPerPod`, `maxMemoryBytesPerPod`
- `POST /apps/{appId}/restart` now documents 409

None of these are in `crates/bunny-net-api/src/containers/types.rs`
(`OverviewPod` :1360, `UserLimits` :1470).

- [ ] Model the fields and show the usage in `container app overview`

See [[research/openapi-drift-2026-10-23]].
