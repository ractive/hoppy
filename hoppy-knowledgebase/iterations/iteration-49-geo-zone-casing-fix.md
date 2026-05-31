---
title: Iter-49 — geo-zone serde casing fix + pull-zone field audit
type: iteration
date: 2026-06-01
tags: [iteration, pull-zone, serde, casing, correctness]
status: planned
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

- [ ] Add explicit `#[serde(rename = "EnableGeoZoneUS")]` (and EU, ASIA,
      SA, AF) on both `PullZone` (read) and `UpdatePullZone` (write).
- [ ] Confirm the request side already serialises with the correct
      casing; fix if not.

### 2. Audit every PascalCase rename in pull-zone types

- [ ] Grep `crates/bunny-net-api/src/core/types.rs` (and any pull-zone
      type file) for fields where the Rust name's PascalCase differs
      from the API's true casing (acronyms: US, EU, IP, IPv4, IPv6,
      DDoS, DNS, URL, ID, SSL, TLS, AWS, S3).
- [ ] Add explicit `#[serde(rename = …)]` to each one with the *real*
      API casing.

### 3. Tests

- [ ] Round-trip test: deserialise a saved `pull-zone get` JSON
      response (real shape, captured in a fixture) and assert all
      `EnableGeoZone*` fields read back as `true` when the API said
      `true`.
- [ ] Add a parametric helper that asserts no `#[serde(default)]`
      field on a pull-zone response struct is `false` when the
      corresponding API field is `true` — catches future regressions.

### 4. Optional safety net

- [ ] Investigate dropping `#[serde(default)]` from boolean fields
      where the API always returns a value. Out-of-scope if it
      breaks existing fixtures.

## Out of scope

- Fixing casing in other domains (DNS, stream, etc.) — track
  separately if discovered.
- Removing `#[serde(default)]` repo-wide.

## Acceptance Criteria

- [ ] `hoppy pull-zone update --enable-geo-zone-us true && hoppy pull-zone get`
      shows `EnableGeoZoneUs: true` (or whatever rendered name we
      choose).
- [ ] New round-trip test passes against a captured fixture.
- [ ] Audit list of renamed fields is documented in the PR body.
- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --quiet` all clean.

## Related

- [[../backlog/geo-zone-flags-casing-mismatch]]
- [[iteration-47-pull-zone-firewall-and-rate-limiting]]
