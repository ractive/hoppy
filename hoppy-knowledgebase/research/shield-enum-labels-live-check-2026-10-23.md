---
title: Shield enum labels - live check against the API (2026-10-23)
type: research
date: 2026-10-23
status: completed
tags:
  - shield
  - api-drift
  - research
---

# Shield enum labels: live check

Question: is `specs/shield.json` right about the integer meaning of
`AccessListAction` and `AccessListType`, or is hoppy's older table? Test
account, shield zone on a fresh `hoppy-test-iter90` pull zone (zone 206411 from
iter-87 no longer exists: `Shield Zone not found`). Pull zone deleted afterwards;
the shield zone is orphaned (no delete API).

## Findings

1. **The API publishes its own tables.** `GET .../access-lists/enums`
   (`hoppy shield access-list enums`) returns number to name maps. They match the
   spec exactly: `AccessListAction` 1 Allow, 2 Block, 3 Challenge, 4 Log, 5 Bypass
   (no 0 in the live map); `AccessListType` 0 IP, 1 CIDR, 2 ASN, 3 Country,
   4 Organization, 5 JA4; `ShieldPlanType` 0 Basic, 1 Advanced, 2 Business,
   3 Enterprise. hoppy's old fixture had the inverse shape (name to number) and
   the old labels; it is replaced with the live shape.
2. **Behaviour agrees.** A custom list created with `--type 1` and content
   `203.0.113.0/24` was accepted and read back as type 1 (old hoppy label:
   "Country"). Content `XX` with `--type 3` was rejected:
   `'XX' is not a correct Country value`. So 3 is Country, 1 is CIDR.
3. **Managed lists read back with the spec meaning.** Raw `action` for every
   managed list is 4 (hoppy printed "Challenge"; spec and live enums: Log).
   "VPN Providers" (10941 entries) and "SpamHaus DROP" are raw type 1, which
   hoppy printed as "Country" but are CIDR lists; "iCloud Private Relay" is raw
   4 (hoppy: "UserAgent", spec: Organization).
4. Setting `action` 2 on a custom list round-trips as raw 2 (Block per the live
   enums).
5. **Not provable live (kept to the spec):** `WafRuleActionType`,
   `RatelimitRuleActionType`, `WafRuleSeverityType`, `WafRatelimitCounterKeyType`,
   `WafRuleTransformationType`, `ReviewActionType`, `WAFPayloadLimitAction`,
   execution modes. The Basic plan cannot run bot detection or WAF custom
   rules. Circumstantial support: a fresh zone in learning mode returns
   `wafExecutionMode: 0`, `wafRequestBodyLimitAction: 1`, which reads sensibly
   as Log / Log under the spec labels and not at all as "Disabled" / "Block".

## Not touched (outside Shield), filed in backlog

Core pull-zone enums whose spec labels differ: see
[[backlog/core-enum-labels-disagree-with-spec]].

Cleanup: pull zone 6853247 deleted via `dogfooding/cleanup.sh`; the shield zone
206524 stays orphaned.
