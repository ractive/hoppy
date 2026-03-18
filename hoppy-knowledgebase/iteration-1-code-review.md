---
title: "Iteration 1 Code Review — Findings & Action Items"
date: 2026-03-18
tags:
  - code-review
  - iteration-1
  - api-client
  - rust
  - action-items
status: active
---

# Iteration 1 Code Review

Review of the 5 hand-written bunny-api-* crates (`core`, `shield`, `stream`, `compute`, `storage`) after the iter-0.5 experiment. These findings should be addressed before building CLI integration on top.

## High Priority

### 1. `tokio` should be a dev-dependency, not a dependency

All 5 crates declare `tokio = { version = "1", features = ["rt-multi-thread", "macros"] }` in `[dependencies]`. The API crates never spawn tasks or use `#[tokio::main]` — they only `.await` futures. This forces downstream users to compile the multi-threaded tokio runtime even if they use a single-threaded executor.

**Fix:** Move tokio to `[dev-dependencies]` in all 5 crates. The crates are runtime-agnostic.

### 2. `compute::ApiError` missing `std::error::Error` impl

Only `Display` is implemented, not `std::error::Error`. This means compute errors can't be used in `anyhow::Error::new(api_err)` — they're silently wrapped as display strings rather than downcast-able errors.

**Fix:** Add `impl std::error::Error for ApiError {}` in compute's types.rs.

### 3. Compute has zero tests

Compute has the most complex business logic (upsert 200/204 split, placeholder construction) but no tests at all. Minimum needed:
- `script_type_roundtrip` (enum serde)
- `create_edge_script_serializes`
- `paginated_list_handles_null_items`
- `upsert_variable_placeholder_construction`

### 4. Storage has zero tests

Storage has unique constructor and URL construction logic. Minimum needed:
- `listing_url_empty_path` / `listing_url_with_path`
- `file_url_empty_path` / `file_url_with_path`
- These are pure unit tests with no network dependency.

## Medium Priority

### 5. Rename `BunnyClient` → `CoreClient`

All other crates use API-specific names (`ShieldClient`, `StreamClient`, `ComputeClient`, `StorageClient`). Core is the outlier with `BunnyClient`. Rename for consistency.

### 6. Standardize constructors

Three different patterns exist:
- core/shield: `new(key) -> Result<Self>` — uses `Client::builder().build()` which doesn't meaningfully fail
- stream/compute: `new(key) -> Self` — uses `Client::new()` directly (simpler, more honest)
- storage: `new(region, key) -> Self` — different shape due to per-zone auth

**Fix:** Use `new(key) -> Self` with `Client::new()` for core and shield. Storage keeps its unique constructor (different auth model). All crates should have `with_base_url(key, url) -> Self` for testability.

### 7. Standardize auth pattern

Three different approaches:
- core: `auth_headers() -> Result<HeaderMap>` (Pattern A — allocates HeaderMap per request)
- shield: inline `.header("AccessKey", &self.api_key)` on every method (24 copies, no helper)
- stream/compute: `fn auth(&self, rb: RequestBuilder) -> RequestBuilder` (Pattern B — clean decorator)

**Fix:** Adopt Pattern B everywhere:
```rust
fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
    rb.header("AccessKey", &self.api_key)
}
```

### 8. Use reqwest `.query()` in core

Core manually builds query strings with `Vec<(&str, String)>` and string joining. All other crates use reqwest's `.query()` which handles URL encoding automatically.

**Fix:** Replace manual query building with `.query()` calls.

### 9. Add `with_base_url` to compute

Compute is the only crate without a URL override, making it impossible to test against a mock server.

### 10. Fix `shield_zone_id` type: `i32` → `i64`

Shield mixes `i32` (for `shield_zone_id` in most methods) and `i64` (for `pull_zone_id`). `CreateCustomWafRule::shield_zone_id` is `i32` but `ShieldZoneResponse::shield_zone_id` is `i64`. Silent truncation risk if IDs exceed `i32::MAX`.

**Fix:** Use `i64` everywhere in shield.

### 11. Fix `update_bot_detection` body field overwrite

The method takes `mut body` and silently overwrites `body.shield_zone_id` from the path parameter. The body type should not include `shield_zone_id` for the update path — it belongs only in the URL.

### 12. Fix `compute::PaginatedList::items`

Currently `Option<Vec<T>>` — should be `Vec<T>` with `#[serde(default = "Vec::new")]` to match stream/core pattern.

### 13. Fix `compute::create_script` duplicated error handling

Re-implements `json_or_error` inline instead of calling it. The comment says "API returns 201" but `json_or_error` already handles any 2xx via `status.is_success()`.

### 14. Fix stale doc comments

`lib.rs` in core, shield, and stream still reference `hoppy_api_*` module paths.

## Low Priority / Future Iteration

### Error handling pattern

Adopt stream's approach across all crates — read response text first, then attempt parse. This gives the best diagnostics when the API returns unexpected responses (HTML 503 from proxy, empty body, etc.).

Current stream pattern:
```rust
let text = resp.text().await?;
serde_json::from_str::<T>(&text).context("failed to parse response")
```

### Filter structs for list operations

`list_videos` has 6 flat `Option` parameters — painful to call as `client.list_videos(id, None, None, Some("foo"), None, None)`. A builder-style filter struct would be cleaner:
```rust
let videos = client.list_videos(lib_id, &ListVideos::default().search("foo")).await?;
```

Same applies to `list_collections`, `list_pull_zones`.

### Bake context IDs into clients

- **Stream:** `library_id` appears on every method. Could be `StreamClient::for_library(key, library_id)`.
- **Storage:** zone name passed on every method. Already constructed per-zone, so store it on the client.

### No shared crate needed (for now)

Response/error handling is ~25 lines per crate. Error types genuinely differ (ApiError, ProblemDetails, StatusMessage, StorageError). Not worth a shared crate at 5 crates. Revisit if more APIs are added.

## Decisions Made

- **Client naming:** `{Api}Client` — `CoreClient`, `ShieldClient`, `StreamClient`, `ComputeClient`, `StorageClient`
- **Constructor:** `new(key) -> Self` (infallible), `with_base_url(key, url) -> Self`
- **Auth:** Pattern B decorator on `RequestBuilder`
- **No shared crate** for response/error handling
