---
title: Several Shield enum labels in hoppy disagree with the spec
type: backlog
date: 2026-10-23
status: completed
priority: high
origin: iter-87
tags:
  - shield
  - api-drift
  - dx
resolved-in: iter-90
---

# Shield enum labels disagree with the spec

Found while fixing `WafRuleOperatorType` in [[iterations/iteration-87-openapi-drift-fixes]].
The operator bug was not new drift: the spec never matched. The same applies to
other Shield enums, so a diff of old spec against new spec cannot find it. Wire
numbers are what users type, so the problem is wrong names in help text,
`Display` output (tables) and library variant names. Some of these can make a
user pick the wrong action: access-list `1` is **Allow** in the spec but
`Block` in hoppy.

Spec (`specs/shield.json`, `description` / `x-enum-varnames`) versus hoppy
(`crates/bunny-net-api/src/shield/types.rs`):

| Enum | Spec | hoppy |
|------|------|-------|
| `AccessListAction` | 0 None, 1 Allow, 2 Block, 3 Challenge, 4 Log, 5 Bypass | 0 NoAction, 1 Block, 2 Allow, 3 LogOnly, 4 Challenge, 5 ChallengeInterstitial |
| `AccessListType` | 0 IP, 1 CIDR, 2 ASN, 3 Country, 4 Organization, 5 JA4 | 0 Ip, 1 Country, 2 Asn, 3 Hostname, 4 UserAgent, 5 Custom |
| `WafRuleActionType` | 1 Block, 2 Log, 3 Challenge, 4 Allow, 5 Bypass | 1 Block, 2 LogOnly, 3 Challenge, 4 ChallengeInterstitial, 5 Allow |
| `RatelimitRuleActionType` | 1 RateLimit, 2 Log, 3 Challenge | 1 Block, 2 LogOnly, 3 Challenge |
| `WafRuleSeverityType` | 0 NOTICE, 1 WARNING, 2 CRITICAL | 0 Low, 1 Medium, 2 High |
| `ShieldPlanType` | 0 Basic, 1 Advanced, 2 Business, 3 Enterprise | 0 Basic, 1 Standard, 2 Advanced, 3 Enterprise |
| `BotDetectionExecutionMode` | 0 LogOnly, 1 Challenge | 0 Disabled, 1 Enabled |
| `BotDetectionSensitivity` | 0 Off, 1 Low, 2 Medium, 3 High | 0 Disabled, ... |
| `BrowserFingerprintAggression` | 0 Unknown, 1 Low, 2 Medium, 3 High, 4 Custom | 0 Disabled, 1 VeryLow, 2 Low, 3 Medium, 4 High |
| `DDoSExecutionMode` | 0 Log, 1 Block | 0 Disabled, 1 Enabled (flag is now ignored by the API) |

(`WafExecutionMode`, `WafPayloadLimitAction`: the spec gives no names.)

**Highest risk, review-pr-106:** `AccessListAction` 1 and 2 (Allow/Block) appear
swapped versus the spec. A user who picks an action by its label gets the
opposite policy (an intended Block becomes Allow and vice versa). Verify
against the live API first and fix this enum before the other label work.

- [x] Re-derive every Shield enum from `x-enum-varnames` with a spec-table test like `waf_rule_operator_names_and_numbers_match_spec`
- [x] Rewrite the matching `--*-type` / `--action` help texts; keep numbers unchanged
- [x] Decide per enum whether to also accept names on the CLI (as `--operator-type` does)
- [x] CHANGELOG breaking note (library variant renames; table labels change)
- [x] Consider a generic spec-vs-code enum check in the `/api-drift-sweep` skill so this cannot hide again

## Resolution (iter-90, 2026-10-23)

Fixed in [[iterations/iteration-90-shield-enum-labels]]: all Shield enums relabelled to the spec (plus `BotDetectionSensitivity`, counter key, transformation, payload-limit, review-action and scanner-mode enums found later), names accepted on the CLI, spec-sync tests added. Live check: [[research/shield-enum-labels-live-check-2026-10-23]]. The generic spec-vs-code check is now a unit test for Shield (`specs/shield.json` read in `shield/types.rs` tests); extending it to other specs is tracked in [[backlog/core-enum-labels-disagree-with-spec]] (the `/api-drift-sweep` skill item stays open there).
