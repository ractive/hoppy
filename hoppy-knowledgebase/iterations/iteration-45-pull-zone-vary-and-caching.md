---
title: "Iter-45 — pull-zone vary headers + caching toggles"
type: iteration
date: 2026-05-31
tags:
  - iteration
  - pull-zone
  - caching
  - vary-headers
  - openapi-coverage
status: planned
branch: iter-45/pull-zone-vary-and-caching
---

# Iter-45 — pull-zone vary headers + caching toggles

## Why

Second pull-zone bucket from iter-43's analysis. Combines the
🟡 **vary headers** (7 fields) and 🟡 **performance / caching**
(16 fields) buckets — they're thematically adjacent (both shape
cache key + freshness), almost all booleans or simple ints, and
deliver the biggest user-visible win per PR. See
[[../research/spec-coverage/pull-zone-buckets]].

## Scope

### 1. Vary-header fields on `PullZone` + `UpdatePullZone`

- [ ] `enable_webp_vary: bool` (`EnableWebpVary`)
- [ ] `enable_avif_vary: bool` (`EnableAvifVary`)
- [ ] `enable_cookie_vary: bool` (`EnableCookieVary`)
- [ ] `enable_country_code_vary: bool` (`EnableCountryCodeVary`)
- [ ] `enable_country_state_code_vary: bool` (`EnableCountryStateCodeVary`)
- [ ] `enable_hostname_vary: bool` (`EnableHostnameVary`)
- [ ] `enable_mobile_vary: bool` (`EnableMobileVary`)

### 2. Caching fields on `PullZone` + `UpdatePullZone`

- [ ] `enable_cache_slice: bool` (`EnableCacheSlice`)
- [ ] `enable_smart_cache: bool` (`EnableSmartCache`)
- [ ] `enable_safe_hop: bool` (`EnableSafeHop`)
- [ ] `ignore_query_strings: bool` (`IgnoreQueryStrings`)
- [ ] `enable_query_string_ordering: bool` (`EnableQueryStringOrdering`)
- [ ] `query_string_vary_parameters: Vec<String>` (`QueryStringVaryParameters`)
- [ ] `cookie_vary_parameters: Vec<String>` (`CookieVaryParameters`)
- [ ] `use_stale_while_updating: bool` (`UseStaleWhileUpdating`)
- [ ] `use_stale_while_offline: bool` (`UseStaleWhileOffline`)
- [ ] `use_background_update: bool` (`UseBackgroundUpdate`)
- [ ] `cache_control_max_age_override: Option<i64>` (`CacheControlMaxAgeOverride`)
- [ ] `cache_control_public_max_age_override: Option<i64>` (`CacheControlPublicMaxAgeOverride`)
- [ ] `cache_control_browser_max_age_override: Option<i64>` (`CacheControlBrowserMaxAgeOverride`)
- [ ] `cache_error_responses: bool` (`CacheErrorResponses`)
- [ ] `perma_cache_storage_zone_id: Option<i64>` (`PermaCacheStorageZoneId`)
- [ ] `perma_cache_type: Option<PermaCacheType>` (`PermaCacheType`) — new enum; check spec for variants

### 3. CLI flags

In `crates/hoppy-cli/src/cli.rs`, add one flag per new field on
`hoppy pull-zone update`:

- [ ] 10 boolean toggles for vary + cache enable/use flags.
- [ ] 3 `Option<i64>` override flags for cache-control max-age.
- [ ] 2 `Vec<String>` flags with `value_delimiter = ','` for vary parameters.
- [ ] `--perma-cache-storage-zone-id` and `--perma-cache-type`.
- [ ] All flags carry `help = "..."` text describing the on/off semantics.

### 4. Tests + snapshots

- [ ] `cargo test --workspace --quiet` clean.
- [ ] Refresh e2e snapshots for `hoppy pull-zone update --help`.
- [ ] Integration test: sparse update with one vary flag + one cache
      override, verify wire payload.

## Out of scope

- Firewall + rate limiting (iter-47).
- Origin/routing (iter-46).
- Validation rules for cache-control max-age beyond "non-negative" —
  the API enforces upper bounds; we'd be guessing.

## Acceptance Criteria

- [ ] `cargo fmt && cargo clippy --workspace --all-targets -- -D warnings &&
      cargo test --workspace --quiet` clean.
- [ ] `hoppy pull-zone update --help` lists all 23 new flags with help text.
- [ ] Sparse update with `--use-stale-while-updating true --enable-webp-vary true`
      serialises to exactly those two PascalCase keys.
- [ ] `cargo run -p xtask -- check-iteration-ready --plan
      hoppy-knowledgebase/iterations/iteration-45-pull-zone-vary-and-caching.md
      --base origin/main` exits 0.

## Related

- [[iteration-44-pull-zone-security-compliance]]
- [[../research/spec-coverage/pull-zone-buckets]]
- [[iteration-43-openapi-gap-analysis]]
