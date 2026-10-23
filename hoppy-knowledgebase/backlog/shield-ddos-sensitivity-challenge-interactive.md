---
title: shield zone get/list fails on DDoS sensitivity 5 (ChallengeInteractive)
type: backlog
date: 2026-10-23
status: completed
priority: high
origin: openapi-drift-2026-10-23
tags:
  - shield
  - api-drift
  - deserialization
resolved-in: iter-87
---

# Shield `DDoSShieldSensitivity` gained `5 = ChallengeInteractive`

**Impact: breaking.** The fresh shield spec widens `DDoSShieldSensitivity` to
`0..=5`. `ShieldZoneResponse.d_do_s_shield_sensitivity`
(`crates/bunny-net-api/src/shield/types.rs:673`) is
`Option<DdosShieldSensitivity>` with plain `#[serde(default)]`, not
`deserialize_repr_option`, so a zone set to the new level fails the entire
`shield zone get` / `shield zone list` response.

The variant names are also wrong against the spec, and have been since the
July snapshot: hoppy has `0 = Disabled … 4 = VeryHigh`, while the spec has
`0 = Off, 1 = Low, 2 = Medium, 3 = High, 4 = Challenge, 5 = ChallengeInteractive`.

- [x] Add `ChallengeInteractive = 5` and rename `Disabled`→`Off` and `VeryHigh`→`Challenge`, keeping the old names as CLI aliases
- [x] Use `deserialize_repr_option` on the response field
- [x] Add a fixture or unit test that round-trips an unknown sensitivity value

See [[research/openapi-drift-2026-10-23]].

## Resolution (iter-87, 2026-10-23)

`DdosShieldSensitivity` is now `Off/Low/Medium/High/Challenge/ChallengeInteractive` (0-5), Display and `--ddos-sensitivity` help updated, unknown values round-trip. No hidden aliases: the flag is numeric.

See [[iterations/iteration-87-openapi-drift-fixes]].
