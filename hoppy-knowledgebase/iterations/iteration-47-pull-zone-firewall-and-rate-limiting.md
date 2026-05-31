---
title: "Iter-47 — pull-zone firewall + rate limiting"
type: iteration
date: 2026-05-31
tags:
  - iteration
  - pull-zone
  - firewall
  - rate-limiting
  - openapi-coverage
status: planned
branch: iter-47/pull-zone-firewall-and-rate-limiting
---

# Iter-47 — pull-zone firewall + rate limiting

## Why

Fourth and final structural pull-zone bucket from
[[../research/spec-coverage/pull-zone-buckets]] — 🟠 **firewall /
blocking** (~14 fields). Includes geo blocking, IP/referrer
allow/deny lists, request gating, Shield DDoS toggles, and rate
limiting. Some fields exist on `PullZone` read already
(`blocked_ips`, `allowed_referrers`, `blocked_referrers`) but **not
on `UpdatePullZone`** — users can see them but can't change them.

## Scope

### 1. Audit existing fields first

- [ ] Confirm which of the 14 bucket fields already exist on `PullZone`
      (read) — grep `crates/bunny-net-api/src/core/types.rs`. Don't add
      duplicate read-shape fields.
- [ ] Confirm none exist on `UpdatePullZone` yet (expected: all 14 missing
      from the update payload).

### 2. Add firewall/blocking fields

To `PullZone` (only those not already present) and `UpdatePullZone` (all):

- [ ] `blocked_countries: Vec<String>` (`BlockedCountries`) — ISO-3166-1
      alpha-2 codes
- [ ] `budget_redirected_countries: Vec<String>` (`BudgetRedirectedCountries`)
- [ ] `blocked_ips: Vec<String>` (already on read; add to update)
- [ ] `allowed_referrers: Vec<String>` (already on read; add to update)
- [ ] `blocked_referrers: Vec<String>` (already on read; add to update)
- [ ] `block_none_referrer: bool` (`BlockNoneReferrer`)
- [ ] `block_post_requests: bool` (`BlockPostRequests`)
- [ ] `block_root_path_access: bool` (`BlockRootPathAccess`)
- [ ] `disable_cookies: bool` (`DisableCookies`)

### 3. Shield DDoS + rate limiting fields

- [ ] `shield_ddos_protection_enabled: bool` (`ShieldDDosProtectionEnabled`)
- [ ] `shield_ddos_protection_type: Option<ShieldDDosProtectionType>` —
      new enum, check spec for variants
- [ ] `burst_size: Option<i32>` (`BurstSize`)
- [ ] `request_limit: Option<i32>` (`RequestLimit`)
- [ ] `limit_rate_after: Option<i32>` (`LimitRateAfter`)
- [ ] `limit_rate_per_second: Option<i32>` (`LimitRatePerSecond`)
- [ ] `connection_limit_per_ip_count: Option<i32>` (`ConnectionLimitPerIPCount`)
- [ ] `max_web_socket_connections: Option<i32>` (`MaxWebSocketConnections`)

### 4. CLI flags

- [ ] One flag per field on `hoppy pull-zone update`.
- [ ] `Vec<String>` flags use `value_delimiter = ','` and document the
      format (e.g. country codes, CIDR ranges, domain patterns).
- [ ] `--shield-ddos-protection-type` accepts the enum variants.
- [ ] All numeric flags carry unit labels in help text.

### 5. Tests + snapshots

- [ ] `cargo test --workspace --quiet` clean.
- [ ] Refresh e2e snapshots for `hoppy pull-zone update --help`.
- [ ] Integration test: sparse update with `--blocked-countries CN,RU
      --request-limit 100`, verify wire payload casing + comma-split.

## Out of scope

- WAF rules (separate `shield waf` API, already partially covered).
- Niche / cosmetic pull-zone fields (error pages, preloading screen,
  WebSockets toggle) — file as a backlog cleanup pass instead of an
  iteration.
- Edge scripting / containers / AI fields — defer to iter-21 / iter-26
  owners.
- Reverse-gap cleanup (20 fields in struct that may be stale in the
  spec) — separate audit pass.

## Acceptance Criteria

- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [ ] `hoppy pull-zone update --help` lists all firewall / rate-limit flags
      with help text.
- [ ] Existing fields on `PullZone` (e.g. `blocked_ips`) are not duplicated.
- [ ] `cargo run -p xtask -- check-iteration-ready --plan
      hoppy-knowledgebase/iterations/iteration-47-pull-zone-firewall-and-rate-limiting.md
      --base origin/main` exits 0.

## Related

- [[iteration-44-pull-zone-security-compliance]]
- [[iteration-45-pull-zone-vary-and-caching]]
- [[iteration-46-pull-zone-origin-and-routing]]
- [[../research/spec-coverage/pull-zone-buckets]]
- [[iteration-43-openapi-gap-analysis]]
