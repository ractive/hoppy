---
title: Iter-46 — pull-zone origin + routing toggles
type: iteration
date: 2026-05-31
tags:
  - iteration
  - pull-zone
  - origin
  - routing
  - openapi-coverage
status: completed
branch: iter-46/pull-zone-origin-and-routing
---

# Iter-46 — pull-zone origin + routing toggles

## Why

Third pull-zone bucket. The 🟢 **routing / origin** group from
[[research/spec-coverage/pull-zone-buckets]] — denser than
iter-44/45 (~25 fields), more numbers and enums than booleans, so
help-text + value validation work dominates. Adds origin host
overrides, retry/timeout knobs, origin-shield config, and sticky
session controls.

## Scope

### 1. Origin host + DNS fields on `PullZone` + `UpdatePullZone`

- [x] `origin_host_header: Option<String>` (`OriginHostHeader`)
- [x] `add_host_header: bool` (`AddHostHeader`)
- [x] `add_canonical_header: bool` (`AddCanonicalHeader`)
- [x] `dns_origin_port: Option<i32>` (`DnsOriginPort`)
- [x] `dns_origin_scheme: Option<String>` (`DnsOriginScheme`)
- [x] `follow_redirects: bool` (`FollowRedirects`)

### 2. Timeout + retry fields

- [x] `origin_connect_timeout: Option<i32>` (`OriginConnectTimeout`)
- [x] `origin_response_timeout: Option<i32>` (`OriginResponseTimeout`)
- [x] `origin_retries: Option<i32>` (`OriginRetries`)
- [x] `origin_retry_5xx_responses: bool` (`OriginRetry5XXResponses`)
- [x] `origin_retry_connection_timeout: bool` (`OriginRetryConnectionTimeout`)
- [x] `origin_retry_delay: Option<i32>` (`OriginRetryDelay`)
- [x] `origin_retry_response_timeout: bool` (`OriginRetryResponseTimeout`)

### 3. Origin-shield fields

- [x] `enable_origin_shield: bool` (`EnableOriginShield`)
- [x] `origin_shield_enable_concurrency_limit: bool` (`OriginShieldEnableConcurrencyLimit`)
- [x] `origin_shield_max_concurrent_requests: Option<i32>` (`OriginShieldMaxConcurrentRequests`)
- [x] `origin_shield_max_queued_requests: Option<i32>` (`OriginShieldMaxQueuedRequests`)
- [x] `origin_shield_queue_max_wait_time: Option<i32>` (`OriginShieldQueueMaxWaitTime`)
- [x] `origin_shield_zone_code: Option<String>` (`OriginShieldZoneCode`)

### 4. Routing + sticky session fields

- [x] `enable_request_coalescing: bool` (`EnableRequestCoalescing`)
- [x] `request_coalescing_timeout: Option<i32>` (`RequestCoalescingTimeout`)
- [x] `routing_filters: Vec<String>` (`RoutingFilters`)
- [x] `sticky_session_type: Option<StickySessionType>` (`StickySessionType`) — new enum
- [x] `sticky_session_cookie_name: Option<String>` (`StickySessionCookieName`)
- [x] `sticky_session_client_headers: Vec<String>` (`StickySessionClientHeaders`)
- [x] `pull_zone_tier_type: Option<PullZoneTierType>` (`Type`) — spec field name is `Type`,
      rename to `pull_zone_tier_type` to avoid Rust keyword collision and to be more
      descriptive; the existing `pub zone_type: Option<PullZoneType>` is a different
      field (`Type` here is tier: Standard vs Volume — verify against the spec)

### 5. CLI flags

- [x] One CLI flag per field on `hoppy pull-zone update`, kebab-cased.
- [x] Help text on numeric flags states units (seconds, count) and any
      observable upper bound documented in the spec.
- [x] `--pull-zone-tier-type` accepts the enum variants.

### 6. Tests + snapshots

- [x] `cargo test --workspace --quiet` clean.
- [x] Refresh e2e snapshots for `hoppy pull-zone update --help`.
- [x] Integration test: sparse update setting one timeout + one
      retry flag + sticky-session-cookie-name; verify wire payload.

## Out of scope

- Firewall + rate limiting (iter-47).
- Niche / cosmetic (preloading screen, error pages) — defer to backlog.
- Edge scripting / containers / AI overlap (`EdgeScriptId`,
  `MagicContainersAppId`, `EnableBunnyImageAi`) — flag for iter-21 /
  iter-26 owners rather than bundle here.
- Spec validation for retry-delay ranges — the API enforces server-side.

## Acceptance Criteria

- [x] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [x] `hoppy pull-zone update --help` lists all ~25 new flags with help text.
- [x] `cargo run -p xtask -- check-iteration-ready --plan
      hoppy-knowledgebase/iterations/iteration-46-pull-zone-origin-and-routing.md
      --base origin/main` exits 0.

## Related

- [[iteration-44-pull-zone-security-compliance]]
- [[iteration-45-pull-zone-vary-and-caching]]
- [[research/spec-coverage/pull-zone-buckets]]
