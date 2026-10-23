---
title: Edge rule action types 35-37 (load balancer, cookie stripping) unsupported
type: backlog
date: 2026-10-23
status: planned
priority: medium
origin: openapi-drift-2026-10-23
tags:
  - pull-zone
  - edge-rules
  - api-drift
---

# `EdgeRuleActionType` gained 35–37

The new values are `35 = UseLoadBalancer`,
`36 = ForceLoadBalancerStickySessionMode` and
`37 = StripCookiesClientToOrigin`. `EdgeRuleActionType`
(`crates/bunny-net-api/src/core/types.rs:61`) stops at 34. Response fields use
`deserialize_repr_option`, so the read does not fail, but `edge-rule list`
shows no action for these rules and the CLI cannot create them.

- [ ] Add the three variants with `Display`/`FromStr`
- [ ] Consider keeping the raw integer for unknown actions so list output is not blank

See [[research/openapi-drift-2026-10-23]].
