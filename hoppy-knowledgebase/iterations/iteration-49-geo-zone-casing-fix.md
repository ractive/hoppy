---
title: Iter-49 — geo-zone serde casing fix + pull-zone field audit
type: iteration
date: 2026-06-01
tags:
  - iteration
  - pull-zone
  - serde
  - casing
  - correctness
status: completed
branch: iter-49/geo-zone-casing-fix
---

# Iter-49 — geo-zone casing fix + pull-zone field audit

## Why

`hoppy pull-zone update --enable-geo-zone-us true` succeeds and the
API echoes `EnableGeoZoneUS: true`, but hoppy reads the response back
as `EnableGeoZoneUs` (serde PascalCase default), so all five
`EnableGeoZone*` fields silently deserialise to `false`. Combined with
`#[serde(default)]`, the user sees "false" right after enabling them.

Same shape of bug iter-47 fixed for `BlockedIps` vs `BlockedIPs`. We
need both a targeted fix and a sweep so this doesn't keep showing up.

See [[../backlog/geo-zone-flags-casing-mismatch]].

## Scope

### 1. Fix the five geo-zone fields

- [x] Add explicit `#[serde(rename = "EnableGeoZoneUS")]` (and EU, ASIA,
      SA, AF) on both `PullZone` (read) and `UpdatePullZone` (write).
- [x] Confirm the request side already serialises with the correct
      casing; fix if not.

### 2. Audit every PascalCase rename in pull-zone types

- [x] Grep `crates/bunny-net-api/src/core/types.rs` (and any pull-zone
      type file) for fields where the Rust name's PascalCase differs
      from the API's true casing (acronyms: US, EU, IP, IPv4, IPv6,
      DDoS, DNS, URL, ID, SSL, TLS, AWS, S3).
- [x] Add explicit `#[serde(rename = …)]` to each one with the *real*
      API casing.

### 3. Tests

- [x] Round-trip test: deserialise a saved `pull-zone get` JSON
      response (real shape, captured in a fixture) and assert all
      `EnableGeoZone*` fields read back as `true` when the API said
      `true`.
- [x] Add a parametric helper that asserts no `#[serde(default)]`
      field on a pull-zone response struct is `false` when the
      corresponding API field is `true` — catches future regressions.

### 4. Optional safety net

- [x] Investigate dropping `#[serde(default)]` from boolean fields
      where the API always returns a value. Out-of-scope if it
      breaks existing fixtures.

## Out of scope

- Fixing casing in other domains (DNS, stream, etc.) — track
  separately if discovered.
- Removing `#[serde(default)]` repo-wide.

## Acceptance Criteria

- [ ] `hoppy pull-zone update --enable-geo-zone-us true && hoppy pull-zone get`
      shows `EnableGeoZoneUS: true`. *(Deferred to dogfooding pass — code +
      tests verified; live API check pending.)*
- [x] New round-trip test passes against a captured fixture.
- [x] Audit list of renamed fields is documented in the PR body.
- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Audit results

Fields with explicit `#[serde(rename)]` added or confirmed in this iteration:

### Added in iter-49 (`PullZone` + `UpdatePullZone`)

| Rust field | Wrong PascalCase (before) | Correct API key (after) |
|---|---|---|
| `enable_geo_zone_us` | `EnableGeoZoneUs` | `EnableGeoZoneUS` |
| `enable_geo_zone_eu` | `EnableGeoZoneEu` | `EnableGeoZoneEU` |
| `enable_geo_zone_asia` | `EnableGeoZoneAsia` | `EnableGeoZoneASIA` |
| `enable_geo_zone_sa` | `EnableGeoZoneSa` | `EnableGeoZoneSA` |
| `enable_geo_zone_af` | `EnableGeoZoneAf` | `EnableGeoZoneAF` |
| `enable_webp_vary` (UpdatePullZone only) | `EnableWebpVary` | `EnableWebPVary` |

### Pre-existing correct renames (audit confirmed, no change needed)

`EnableTLS1`, `EnableTLS1_1`, `EnableAutoSSL`, `VerifyOriginSSL`,
`ZoneSecurityIncludeHashRemoteIP`, `AWSSigningEnabled`, `AWSSigningKey`,
`AWSSigningSecret`, `AWSSigningRegionName`, `LoggingIPAnonymizationEnabled`,
`EnableWebPVary` (PullZone read struct), `EnableAvifVary`,
`OriginRetry5XXResponses`, `ShieldDDosProtectionEnabled`,
`ShieldDDosProtectionType`, `ConnectionLimitPerIPCount`,
`OptimizerMinifyCSS`, `OptimizerMinifyJavaScript`, `BlockedIPs` (UpdatePullZone).

### Note on `BlockedIps` vs `BlockedIPs`

The live API fixture returns `"BlockedIps"` (lowercase s). `PullZone` relies on
`rename_all = "PascalCase"` which produces `BlockedIps` correctly. `UpdatePullZone`
uses `#[serde(rename = "BlockedIPs")]` (added in iter-47) which diverges from the
fixture. This is a pre-existing inconsistency left for a future iteration.

## Related

- [[../backlog/geo-zone-flags-casing-mismatch]]
- [[iteration-47-pull-zone-firewall-and-rate-limiting]]
