---
title: WAF operator enum lacks 13 = IPMATCH and its names do not match the spec
type: backlog
date: 2026-10-23
status: planned
priority: high
origin: openapi-drift-2026-10-23
tags:
  - shield
  - waf
  - api-drift
  - deserialization
---

# `WafRuleOperatorType` drift

**Impact: breaking.** The fresh spec adds `13 = IPMATCH`. `operator_type` is a
non-optional `WafRuleOperatorType` on `WafChainedRuleCondition`,
`WafRuleConfiguration` and `RateLimitRuleConfiguration`
(`crates/bunny-net-api/src/shield/types.rs:778,790,867`), so listing custom WAF
or rate-limit rules fails as soon as any rule uses IP matching. Dashboard IP
rules probably do.

**Pre-existing correctness bug:** hoppy's variant names do not match the wire
meaning. The spec has `0 BEGINSWITH, 1 ENDSWITH, 2 CONTAINS, 3 CONTAINSWORD,
4 STRMATCH, 5 EQ, 6 GE, 7 GT, 8 LE, 9 LT, 12 WITHIN, 13 IPMATCH, 14 RX,
15 STREQ, 17 DETECTSQLI, 18 DETECTXSS`. hoppy has `0 Eq, 1 NotEq, …, 6 Regex,
15 IpMatch, …`, so `--operator eq` sends BEGINSWITH and `--operator regex`
sends GE.

- [ ] Rename the variants to the spec meanings and add `IpMatch = 13`
- [ ] Add a forward-compatible fallback for unknown operators on the response structs
- [ ] Add a CLI migration note, since the operator names change meaning
- [ ] Model `isNegated` / `isRegexVariable`, which are new on all three condition schemas

See [[research/openapi-drift-2026-10-23]].
