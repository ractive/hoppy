---
title: Iter-90 - Shield enum labels follow the spec
type: iteration
date: 2026-10-23
tags:
  - iteration
  - shield
  - api-drift
  - breaking
status: completed
branch: iter-90/shield-enum-labels
---

# Iter-90 - Shield enum labels follow the spec

Backlog: [[backlog/shield-enum-labels-disagree-with-spec]]. Research:
[[research/shield-enum-labels-live-check-2026-10-23]]. Pattern from
[[iterations/iteration-87-openapi-drift-fixes]] (`WafRuleOperatorType`).

## Why

Eighteen Shield integer enums carried labels that never matched
`specs/shield.json`. Most dangerous: `AccessListAction` 1 and 2 were swapped
(a user following the help created the opposite policy). Wire numbers are
unchanged; only labels (variants, `Display`, help, CLI input) change.

## Design

- New `repr_enum_names!` macro (next to `forward_compat_repr_enum!`): one
  `Variant => "SpecName"` table generates `NAMES`, `name()`, `describe_all()`,
  `Display` and `FromStr` (spec name case-insensitive, or declared number).
  `WafRuleOperatorType` moved onto it.
- Variant names are the spec names. Exceptions: `BrowserFingerprintAggression`
  spec `Unknown` (0) is `Unspecified` in Rust (collides with the forward-compat
  `Unknown(n)`), displayed as `Unknown`; type name `RateLimitActionType` kept
  (spec `RatelimitRuleActionType`).
- CLI flags take the enum type (`NAME|N`); no aliases for old labels, because
  `block` or `low` would now mean a different number.
- Tests pin each enum to a hand-written table and to `x-enum-varnames` read
  from `specs/shield.json`, so a spec refresh that moves a label fails.

## Fixture changes

- `fixtures/shield/access_lists_get.json` and `access_list_get.json` were
  **hand-edited** (type 1 to 3, action 1 to 2) so the lists still read as
  "Blocked Countries" with a Block action under the new tables. They are hand
  written test contracts, not recorded responses.
- `fixtures/shield/access_list_enums.json` was **reshaped from the live check**
  (number to name, as the API returns it; the old fixture had name to number
  with the old labels).
- Review follow-ups: JSON list rows (`access-list list`, `shield zone list`,
  `waf list-rules`) carry numeric `*_id` fields next to the labels;
  `BotCategorizationAction` / `BotCategoryAction` moved onto
  `repr_enum_names!` (CLI takes names); macro unit tests; e2e for
  `waf update-rule`, `review-triggered-rule`, `rate-limit update`,
  `upload-scanning update`, `bot-categorization`.

## Tasks

- [x] Branch `iter-90/shield-enum-labels` from `origin/main`; merge `main` after #108
- [x] Live-check `AccessListAction` / `AccessListType` (research note)
- [x] `repr_enum_names!` macro
- [x] Relabel `AccessListAction`, `AccessListType`, `WafRuleActionType`, `RatelimitRuleActionType`, `WafRuleSeverityType`, `ShieldPlanType`, `WafExecutionMode`, `BotDetectionExecutionMode`, `BrowserFingerprintAggression`, `DDoSExecutionMode`
- [x] Also relabel the further mismatches found: `BotDetectionSensitivity` (Disabled to Off), `WafRatelimitCounterKeyType`, `WafRuleTransformationType`, `WAFPayloadLimitAction`, `ReviewActionType`, `UploadScanningScannerMode` (LogOnly to Log)
- [x] CLI flags accept names or numbers; help text from the spec
- [x] Spec-sync unit tests for every Shield enum
- [x] Help-text sync test over all affected flags
- [x] e2e tests: names send the right wire numbers; old labels rejected with exit 2
- [x] Fixtures changed together with types: `access_list_get`, `access_lists_get` (type 3 = Country, action 2 = Block), `access_list_enums` (live shape and values); three snapshots
- [x] CHANGELOG breaking section with migration table; remove "known remaining mismatches" bullet
- [x] Scan non-Shield integer enums; file [[backlog/core-enum-labels-disagree-with-spec]]
- [x] `cargo build --release` and dogfood changed commands with the test key; clean up
- [x] fmt, clippy, test gates

## Acceptance criteria

- [x] `--action allow` on `access-list update-config` sends 1, `block` sends 2
- [x] Every Shield enum's name table equals the spec's `x-enum-varnames`
- [x] Old labels are rejected with exit 2, not aliased
- [x] `access-list list` against the live API shows CIDR for the managed VPN list and Log as the default action
