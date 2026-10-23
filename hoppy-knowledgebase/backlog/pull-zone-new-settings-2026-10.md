---
title: Pull zone IP family, TLS level, load balancer and optimizer AI settings not modeled
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - pull-zone
  - api-drift
  - cli-coverage
---

# New pull-zone settings (core spec, Oct 2026)

These fields are new on `PullZoneModel`, `PullZoneAddModel` and
`PullZoneSettingsModel`:

- `IpFamilyPolicy`: `0 IPv4Only, 1 DualStack, 2 DualStackPreferIPv6, 3 IPv6Only`
- `TlsSecurityLevel`: `0 Legacy, 1 Compatible, 2 ModernOnly`
- `LoadBalancerId` (int64, nullable)
- `OptimizerHtmlToMarkdown`: `0 Off, 1 Simple, 2 Prerendered`
- `OptimizerLlmsTxt` (bool)
- `OptimizerTunnelActiveProbingEnabled` (bool)

None are in `core::PullZone` or `UpdatePullZone`
(`crates/bunny-net-api/src/core/types.rs`).

- [ ] Model the fields with forward-compatible enums
- [ ] Add `pull-zone update` flags and show the values in `pull-zone get`

See [[research/openapi-drift-2026-10-23]].
