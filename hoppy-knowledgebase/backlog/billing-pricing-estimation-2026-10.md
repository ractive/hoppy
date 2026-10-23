---
title: Billing AI Gateway / Load Balancer charges and pricing estimation endpoint
type: backlog
date: 2026-10-23
status: planned
priority: low
origin: openapi-drift-2026-10-23
tags:
  - billing
  - api-coverage
---

# Billing and pricing additions (core spec)

- `BillingModel`: `MonthlyChargesAiGateway`, `MonthlyChargesLoadBalancer`, `MonthlyAiGateway{Input,Output,Cached}Tokens`, `MonthlyAiGatewayOtherCharges`
- `GET /v1/pricing/{source}/{resourceId}` returns a `PriceEstimationModel` (active price, plus a cost estimate when a usage amount is passed; `PricingEstimationSource` is a large integer enum)

`core::BillingDetails` (`crates/bunny-net-api/src/core/types.rs:3787`) does not
model the new charge lines.

- [ ] Add the charge fields to `billing get` output
- [ ] Evaluate a `pricing estimate` command

See [[research/openapi-drift-2026-10-23]].
