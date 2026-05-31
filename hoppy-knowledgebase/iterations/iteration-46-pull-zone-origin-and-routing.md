---
title: "Iter-46 — pull-zone origin + routing toggles"
type: iteration
date: 2026-05-31
tags:
  - iteration
  - pull-zone
  - origin
  - routing
  - openapi-coverage
status: planned
branch: iter-46/pull-zone-origin-and-routing
---

# Iter-46 — pull-zone origin + routing toggles

## Why

Third pull-zone bucket. The 🟢 **routing / origin** group from
[[../research/spec-coverage/pull-zone-buckets]] — denser than
iter-44/45 (~25 fields), more numbers and enums than booleans, so
help-text + value validation work dominates. Adds origin host
overrides, retry/timeout knobs, origin-shield config, and sticky
session controls.

## Scope

### 1. Origin host + DNS fields on `PullZone` + `UpdatePullZone`

- [ ] `origin_host_header: Option<String>` (`OriginHostHeader`)
- [ ] `add_host_header: bool` (`AddHostHeader`)
- [ ] `add_canonical_header: bool` (`AddCanonicalHeader`)
- [ ] `dns_origin_port: Option<i32>` (`DnsOriginPort`)
- [ ] `dns_origin_scheme: Option<String>` (`DnsOriginScheme`)
- [ ] `follow_redirects: bool` (`FollowRedirects`)

### 2. Timeout + retry fields

- [ ] `origin_connect_timeout: Option<i32>` (`OriginConnectTimeout`)
- [ ] `origin_response_timeout: Option<i32>` (`OriginResponseTimeout`)
- [ ] `origin_retries: Option<i32>` (`OriginRetries`)
- [ ] `origin_retry_5xx_responses: bool` (`OriginRetry5XXResponses`)
- [ ] `origin_retry_connection_timeout: bool` (`OriginRetryConnectionTimeout`)
- [ ] `origin_retry_delay: Option<i32>` (`OriginRetryDelay`)
- [ ] `origin_retry_response_timeout: bool` (`OriginRetryResponseTimeout`)

### 3. Origin-shield fields

- [ ] `enable_origin_shield: bool` (`EnableOriginShield`)
- [ ] `origin_shield_enable_concurrency_limit: bool` (`OriginShieldEnableConcurrencyLimit`)
- [ ] `origin_shield_max_concurrent_requests: Option<i32>` (`OriginShieldMaxConcurrentRequests`)
- [ ] `origin_shield_max_queued_requests: Option<i32>` (`OriginShieldMaxQueuedRequests`)
- [ ] `origin_shield_queue_max_wait_time: Option<i32>` (`OriginShieldQueueMaxWaitTime`)
- [ ] `origin_shield_zone_code: Option<String>` (`OriginShieldZoneCode`)

### 4. Routing + sticky session fields

- [ ] `enable_request_coalescing: bool` (`EnableRequestCoalescing`)
- [ ] `request_coalescing_timeout: Option<i32>` (`RequestCoalescingTimeout`)
- [ ] `routing_filters: Vec<String>` (`RoutingFilters`)
- [ ] `sticky_session_type: Option<StickySessionType>` (`StickySessionType`) — new enum
- [ ] `sticky_session_cookie_name: Option<String>` (`StickySessionCookieName`)
- [ ] `sticky_session_client_headers: Vec<String>` (`StickySessionClientHeaders`)
- [ ] `pull_zone_tier_type: Option<PullZoneTierType>` (`Type`) — spec field name is `Type`,
      rename to `pull_zone_tier_type` to avoid Rust keyword collision and to be more
      descriptive; the existing `pub zone_type: Option<PullZoneType>` is a different
      field (`Type` here is tier: Standard vs Volume — verify against the spec)

### 5. CLI flags

- [ ] One CLI flag per field on `hoppy pull-zone update`, kebab-cased.
- [ ] Help text on numeric flags states units (seconds, count) and any
      observable upper bound documented in the spec.
- [ ] `--pull-zone-tier-type` accepts the enum variants.

### 6. Tests + snapshots

- [ ] `cargo test --workspace --quiet` clean.
- [ ] Refresh e2e snapshots for `hoppy pull-zone update --help`.
- [ ] Integration test: sparse update setting one timeout + one
      retry flag + sticky-session-cookie-name; verify wire payload.

## Out of scope

- Firewall + rate limiting (iter-47).
- Niche / cosmetic (preloading screen, error pages) — defer to backlog.
- Edge scripting / containers / AI overlap (`EdgeScriptId`,
  `MagicContainersAppId`, `EnableBunnyImageAi`) — flag for iter-21 /
  iter-26 owners rather than bundle here.
- Spec validation for retry-delay ranges — the API enforces server-side.

## Acceptance Criteria

- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [ ] `hoppy pull-zone update --help` lists all ~25 new flags with help text.
- [ ] `cargo run -p xtask -- check-iteration-ready --plan
      hoppy-knowledgebase/iterations/iteration-46-pull-zone-origin-and-routing.md
      --base origin/main` exits 0.

## Related

- [[iteration-44-pull-zone-security-compliance]]
- [[iteration-45-pull-zone-vary-and-caching]]
- [[../research/spec-coverage/pull-zone-buckets]]
