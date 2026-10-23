---
title: Load Balancer API (16 new core operations) not covered
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - load-balancer
  - api-coverage
  - new-service
---

# Load Balancer API

The core spec gained 16 `/loadbalancer` operations:

- profiles: list, get, create, update, delete
- origin groups: add, update, delete (health checks, geo lat/long, weights, priority)
- origins: add, update, delete (standard / edge script / storage zone / Magic Containers)
- statistics (per LB and account roll-up), usage and pricing, attached pull zones

The related schema additions are `LoadBalancerProfileModel`,
`LoadBalancerOriginGroupModel`, `LoadBalancerOriginModel`, `OriginType`
(`0 Standard, 1 Storage, 2 EdgeScript, 3 MagicContainer`), the load-balancing
method enums, `PullZone*.LoadBalancerId`, and the new edge-rule actions 35/36.

hoppy has no client module. The only reference is
`core::OriginType::LoadBalancer = 3`.

- [ ] Add a feature-gated `crates/bunny-net-api/src/load_balancer/` module, following the per-service module rule
- [ ] Add a `hoppy load-balancer` command tree with e2e tests

See [[research/openapi-drift-2026-10-23]].
