---
title: Iter-87 - OpenAPI drift fixes (spec refresh, WAF operators, enum forward-compat)
type: iteration
date: 2026-10-23
tags:
  - iteration
  - api-drift
  - openapi
  - shield
  - containers
status: planned
branch: iter-87/openapi-drift-fixes
---

# Iter-87 - OpenAPI drift fixes

## Why

[[research/openapi-drift-2026-10-23]] diffed every published bunny.net spec
against the `specs/` snapshot from [[iterations/iteration-66-spec-refresh-drift-fixes]].
Four closed response enums gained values. Any of them can now fail a whole
list or get response on live data. The WAF operator names are also wrong, so
users who follow the help text send the wrong operator. This iteration is the
correctness pass, following the iter-66 pattern: refresh the specs, fix the
breaking drift, live-check the ambiguous items, and close the cheap gaps on
endpoints that hoppy already models.

**Start point:** branch from `main` **after PR #104 (iter-86) merges**. That
PR changes `core/types.rs` (`OriginType`, `PullZone`) and
`containers/types.rs` (`ContainerEndpoint.id`). Both files are edited again
here.

## Design

### WAF operator mapping (highest priority)

The CLI takes a number (`--operator-type N` on `shield waf add-rule`,
`update-rule`, and the rate-limit equivalents). The wire value is the number
the user types, but the help text and the `WafRuleOperatorType` variant names
label the numbers wrongly. Users who trust the labels get the wrong rule:
"0 = Eq" sends `BEGINSWITH`, "6 = Regex" sends `GE`, and "15 = IpMatch" sends
`STREQ`. Library callers who write `WafRuleOperatorType::IpMatch` also send
`STREQ`. The update-rule fallback config defaults to `::Eq`, which is
`BEGINSWITH`.

Spec mapping: `0 BEGINSWITH, 1 ENDSWITH, 2 CONTAINS, 3 CONTAINSWORD,
4 STRMATCH, 5 EQ, 6 GE, 7 GT, 8 LE, 9 LT, 12 WITHIN, 13 IPMATCH, 14 RX,
15 STREQ, 17 DETECTSQLI, 18 DETECTXSS`.

- Rename the variants to the spec meanings and add `IpMatch = 13`.
- Make `--operator-type` accept a spec name (case-insensitive, such as
  `beginswith`, `rx` or `ipmatch`) as well as the number, through one
  table-driven `FromStr`. Do **not** accept the old wrong labels as aliases.
  `eq` is now a real spec name for `5`, so an alias would silently change
  meaning.
- Unknown operators on responses must not fail the read (see the enum
  forward-compat audit below).

### DDoS sensitivity and bot categorization

`--ddos-sensitivity` takes a number, so there is no CLI spelling to alias. The
rename (`Disabled` to `Off`, `VeryHigh` to `Challenge`, plus
`ChallengeInteractive = 5`) changes the `Display` output in tables and the help
text. It is still a breaking rename for `bunny-net-api` library users, so it
goes in the CHANGELOG under a breaking heading. Bot categorization splits into
`BotCategorizationAction` (0 to 3, per bot) and `BotCategoryAction` (0 to 2,
per category), following the spec.

### Enum forward-compat pattern (decide and record)

The 2026-05-07 decision ("Forward-compat repr-enum deserialisation") covers
only `Option<Enum>` fields that carry `deserialize_repr_option`. All four
breaks this time were outside it: plain `#[serde(default)]`, non-optional repr
fields, or closed string enums. Settle one rule for every response enum, then
apply it everywhere:

- **Repr (integer) enums:** choose between making the field `Option` and
  adding `deserialize_repr_option`, which is lossy, or adding an
  `Unknown(u8)`/`Unknown(i32)` variant with a hand-written
  `Deserialize`/`Serialize`, which round-trips. The read-modify-write paths
  decide this. `shield waf update-rule` and `rate-limit update-rule` fetch the
  current config and PATCH it back with every field required. With the lossy
  helper, an unknown `operatorType` comes back as `None` and the PATCH would
  drop or default it. Prefer the round-trip variant at least for enums that
  are echoed back.
- **String enums:** use `#[serde(other)] Unknown`, which is already used by
  `database::…` status. Note the limit: `#[serde(other)]` does not keep the
  raw string.
- Record the outcome as a new decision-log row that supersedes or extends the
  2026-05-07 entry, and update the module docs in `core/serde_helpers.rs`.

## Tasks

### 0. Setup

- [ ] Branch `iter-87/openapi-drift-fixes` from `main` after PR #104 merges; set status `in-progress`

### 1. Refresh `specs/`

Backlog: [[backlog/specs-snapshot-refresh-2026-10]]

- [ ] Re-download from the canonical URLs in [[research/openapi-drift-2026-10-23]] §1 into a fresh scratch directory. The 2026-10-23 copies from the research pass can be reused if the content is unchanged. Do not commit the `*-mintlify.json` mirrors or `llms.txt`
- [ ] Replace `specs/core-platform.json`, `shield.json`, `stream.json`, `magic-containers.json`, `database.json` and `edge-scripting.json`. `storage`, `logging` and `origin-errors` should be byte-identical: verify this and leave them untouched
- [ ] Add `specs/partner-portal.json` as a **snapshot only**, with no client code (see [[backlog/partner-portal-api-missing]])
- [ ] Update the spec-version references in module docs (for example `database/types.rs` cites 0.0.130, which becomes 0.0.144; stream 1.5.6 becomes 1.6.12)
- [ ] Commit the specs as a separate first commit, so the type fixes diff cleanly against them

**AC:** `git diff --stat main -- specs/` shows 6 modified files and 1 added file. No code comment cites an outdated spec version.

### 2. WAF operator mapping (highest priority)

Backlog: [[backlog/shield-waf-operator-ipmatch]]

- [ ] Rename the `WafRuleOperatorType` variants to the spec names and add `IpMatch = 13`. Fix the doc comment
- [ ] Rewrite the `--operator-type` help text on all four WAF and rate-limit add/update commands from the spec table
- [ ] Have `--operator-type` accept spec names as well as numbers, through a single table shared by `FromStr` and `Display`
- [ ] Re-check the update-rule fallback default (`WafRuleOperatorType::Eq` in `commands/shield.rs`), which is now `BEGINSWITH`. Keep the wire value `0` or choose deliberately, and comment why
- [ ] Unit test: one table-driven test that pins **every** CLI name and number to its serialized wire integer (`beginswith`→0 … `detectxss`→18, `ipmatch`→13, `streq`→15, `rx`→14), plus deserializing `13`
- [ ] e2e (wiremock): `add-rule --operator-type ipmatch` and `--operator-type 13` both send `"operatorType":13`. `waf list` with a rule using an unknown operator (for example `99`) succeeds
- [ ] CHANGELOG: breaking or migration note. Numbers are unchanged on the wire, the labels were wrong, and library variant names changed
- [ ] Model `isNegated` / `isRegexVariable` on the three condition schemas if it is cheap (response read only, `Option<bool>`). Otherwise leave them in the backlog item

**AC:** Each spec operator name has a test asserting its wire value. `IPMATCH` rules list without error. The help text matches the spec exactly. An unknown operator value does not fail `waf list` or `rate-limit list`.

### 3. Breaking enum additions

Backlog: [[backlog/shield-ddos-sensitivity-challenge-interactive]], [[backlog/shield-bot-categorization-ignore-action]], [[backlog/container-volume-status-maintenance]]

- [ ] `DdosShieldSensitivity`: rename `Disabled` to `Off` and `VeryHigh` to `Challenge`, add `ChallengeInteractive = 5`, and update `Display` and the `--ddos-sensitivity` help text. The flag is numeric, so no CLI alias is needed. If a named value parser is added for symmetry with WAF, accept `disabled` and `veryhigh` as hidden aliases
- [ ] `ShieldZoneResponse.d_do_s_shield_sensitivity`: apply the forward-compat pattern (it currently uses plain `#[serde(default)]`)
- [ ] Bot categorization: add `Ignore = 3` to the per-bot `BotCategorizationAction` and a new `BotCategoryAction` (0 to 2) for `BotCategorizationGroup.categoryAction`, `BotCategoryDetails.action` and the `PUT …/categories/{category}` body. `set-bot --action` accepts 3 and `set-category --action` rejects it locally. Update `bot_action_name`
- [ ] `VolumeInstanceStatus`: add `Maintenance`, plus the string-enum fallback
- [ ] Unit tests: the new value deserializes for each enum, and an unknown value does not fail the containing response struct
- [ ] e2e: `shield zone get` with sensitivity 5, `bot-categorization list` with action 3, and `container volume list` with `maintenance` all render

**AC:** All three commands succeed on responses that carry the new values. `set-category --action 3` fails with exit 2 before any request is sent.

### 4. Response-enum forward-compat audit

- [ ] Inventory every enum that appears in a **response** type in `crates/bunny-net-api/src/` (45 `Deserialize_repr` enums across core, shield, stream and compute, plus every string enum in containers, database and stream). Record the field, whether it is optional, whether it has a fallback today, and whether a read-modify-write path echoes it back
- [ ] Decide the pattern (see Design) and record it in `decision-log.md` as a dated row that names the 2026-05-07 entry it extends. Update the `core/serde_helpers.rs` module docs and [[api/bunny-api-client-patterns]]
- [ ] Apply the pattern to every response enum field found without a fallback. Request-only enums are exempt
- [ ] Add a guard against regressions: a unit test per module that feeds an out-of-range value to each response enum, or a documented checklist item in [[api/bunny-api-client-patterns]] if a generic test is impractical

**AC:** No response enum in `bunny-net-api` can fail deserialization on an unknown value. The decision row exists. Read-modify-write commands (WAF and rate-limit `update-rule`) do not silently rewrite an unknown value.

### 5. Live checks before changing payloads

Read-only probes with `TEST_BUNNY_API_KEY` first, following [[dogfooding/dogfooding-playbook]]. The only write is the DDoS mode toggle on a throwaway `hoppy-test-…` shield zone, and it is restored right after.

Backlog: [[backlog/shield-ddos-execution-mode-request-removed]], [[backlog/storage-zone-regions-shape-change]], [[backlog/pull-zone-execution-phase-naming]]

- [ ] Shield `dDoSExecutionMode`: on a test zone, run `shield zone update --ddos-execution-mode <other>` and re-read it. If the value is ignored or rejected, deprecate the flag (hidden, with a warning) and drop the field from `ShieldZoneRequest`, keeping it on the response. If it is honoured, keep it and note in [[api/bunny-api-quirks]] that the spec lags
- [ ] Storage regions: `GET /storagezone/regions` (read-only). If the new shape is confirmed, model `Code`, `Endpoint`, `S3Endpoint`, `Continent`, `Tier`, `IsMainRegionSupported` and `IsS3Supported`, and update `fixtures/core/storagezone_regions.json`, the type and the test together (decision 2026-07-10). Show Code, Endpoint and S3Endpoint in the table
- [ ] Pull-zone execution phase: read a pull zone with an edge script attached and check the bunny docs for the meaning of each phase. Rename to `Cache`/`LoadBalancer` and add `PreCache = 2`. Keep `on-request`/`on-response` as hidden CLI aliases mapped to the same wire values, and fix the help text
- [ ] Record each probe outcome (command, observed result) in the backlog item's resolution note

**AC:** Each of the three items has a recorded live result and a code change that matches it, or an explicit "no change, because…" note.

### 6. Cheap gaps on endpoints already modelled

Backlog: [[backlog/edge-rule-action-types-35-37]], [[backlog/stream-resolutions-cleanup-report-dropped]], [[backlog/container-registry-bunny-registry]] (casing only)

- [ ] `EdgeRuleActionType`: add `UseLoadBalancer = 35`, `ForceLoadBalancerStickySessionMode = 36` and `StripCookiesClientToOrigin = 37` with `Display`/`FromStr`. Use the unknown-value fallback from task 4, so that `edge-rule list` shows the raw number rather than a blank for future values
- [ ] Stream `cleanup_video_resolutions`: return `ResolutionsCleanupModel` (wrapped in the status model). The CLI renders the plan (resolutions to delete, storage objects, playlist changes) and makes it the main `--dry-run` output. JSON output passes it through
- [ ] Container registry `RegistryType` casing: run a read-only live `registry list` to see which casing the API returns, and set `rename_all` to match. Adding `bunnyRegistry` as a **read** variant is fine if it falls out of task 4. The new endpoints stay in the backlog
- [ ] Unit and e2e tests for each item (wiremock fixtures that use the new spec shapes)

**AC:** `edge-rule list` shows actions 35 to 37 by name. `stream video resolutions-cleanup --dry-run` prints the plan. `RegistryType` serializes in the casing the live API uses, with a test that pins it.

### 7. Backlog housekeeping

- [ ] Mark each resolved backlog item `status=resolved` with a short resolution note. Keep the out-of-scope parts open, for example `isNegated` (if deferred), `bunnyRegistry` endpoints, and the `isPlatformManaged` field
- [x] [[backlog/container-endpoint-id-missing]] marked `superseded`, because PR #104 ([[iterations/iteration-86-dogfood-oct-fixes]]) models `ContainerEndpoint.id`

### 8. Gates and dogfood

- [ ] CHANGELOG entries: breaking renames (WAF operator, DDoS sensitivity, execution phase, bot category split) and new values
- [ ] Command-tree notes in `cli/` for any changed flag help or value syntax
- [ ] `cargo fmt`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace --quiet`
- [ ] `cargo build --release`, then dogfood with `target/release/hoppy` against the test account: `shield zone get/list`, `shield waf list`, `shield rate-limit list`, `shield bot-categorization list`, `container volume list`, `storage-zone regions`, `edge-rule list`, and `stream … resolutions-cleanup --dry-run` on a test video. Follow the safe-prefix and cleanup rules in [[dogfooding/dogfooding-playbook]]. File any friction in `backlog/`
- [ ] `hyalo lint` clean on the touched KB files
- [ ] Self-review the diff, then open the PR `iter-87/openapi-drift-fixes`

## Acceptance criteria

- [ ] `specs/` matches the 2026-10-23 bunny.net specs, and `specs/partner-portal.json` is present
- [ ] Every WAF operator name and number maps to the spec wire value, pinned by a test. `IPMATCH` (13) is supported
- [ ] `shield zone`, `bot-categorization`, `waf` and `rate-limit` lists, and `container volume` list, succeed on responses that contain the new and unknown enum values
- [ ] The decision-log row covers the forward-compat pattern for all response enums, and the audit shows no remaining closed response enum
- [ ] Each of the three live-check items has a recorded result and a matching change
- [ ] Edge-rule actions 35 to 37, the stream cleanup plan and the registry casing are handled
- [ ] Quality gates pass and the release build has been dogfooded

## Out of scope (stay in backlog)

- Load Balancer API: [[backlog/load-balancer-api-missing]]
- Shield event-log search/export, zone defaults, under-attack create and the new create-zone fields: [[backlog/shield-event-logs-search-and-zone-defaults]]
- Partner Portal client (spec snapshot only here): [[backlog/partner-portal-api-missing]]
- Billing and pricing estimation: [[backlog/billing-pricing-estimation-2026-10]]
- New pull-zone settings (IP family, TLS level, load balancer id, optimizer AI): [[backlog/pull-zone-new-settings-2026-10]]
- Low items: [[backlog/pull-zone-external-http-certificate]], [[backlog/dns-record-list-filters]], [[backlog/container-overview-usage-fields]], [[backlog/stream-encoding-statistics]], [[backlog/core-misc-new-fields-2026-10]], [[backlog/db-list-per-page-zero-rejected]]
- `bunnyRegistry` endpoints and `isPlatformManaged`: the rest of [[backlog/container-registry-bunny-registry]]

## References

- [[research/openapi-drift-2026-10-23]]: source report
- [[iterations/iteration-66-spec-refresh-drift-fixes]]: precedent for the spec refresh
- [[iterations/iteration-19-pullzone-bugfixes]]: origin of `deserialize_repr_option`
- [[iterations/iteration-86-dogfood-oct-fixes]]: prerequisite PR #104
- [[decision-log]]: 2026-05-07 forward-compat entry and 2026-07-10 fixture rule
