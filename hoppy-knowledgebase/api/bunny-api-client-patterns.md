---
title: "Bunny.net API Client Patterns — Learnings from Iteration 0.5"
date: 2026-03-18
tags:
  - patterns
  - api-client
  - serde
  - reqwest
  - rust
  - reference
status: active
---

# Bunny.net API Client Patterns

Established patterns from the hand-written experiment. Follow these when adding new endpoints or API crates in future iterations.

## 1. JSON Field Casing per API

The bunny.net APIs use **different JSON casing** depending on the service:

| API | JSON Casing | Serde Attribute |
|-----|-------------|----------------|
| Core Platform | PascalCase | `#[serde(rename_all = "PascalCase")]` |
| Stream (Video) | PascalCase | `#[serde(rename_all = "PascalCase")]` |
| Edge Scripting | PascalCase | `#[serde(rename_all = "PascalCase")]` |
| Storage | PascalCase | `#[serde(rename_all = "PascalCase")]` |
| Shield | camelCase | `#[serde(rename_all = "camelCase")]` |

Shield is the odd one out — it uses camelCase while all others use PascalCase. Always verify by reading the spec before adding types for a new API.

## 2. Serde `rename_all` vs Acronym Handling

`rename_all = "PascalCase"` lowercases acronyms in field names. For example, `has_mp4_fallback` becomes `HasMp4Fallback` in the JSON, but the actual wire format is `HasMP4Fallback`.

**Rule:** Any field containing an all-caps acronym (MP4, DRM, URL, SSL, IP, etc.) in the bunny.net JSON needs an explicit `#[serde(rename = "...")]` override:

```rust
#[serde(rename = "HasMP4Fallback")]
pub has_mp4_fallback: bool,
```

This applies to PascalCase APIs only. camelCase APIs don't have this issue.

## 3. Integer Enums via `serde_repr`

Bunny.net uses integer-backed enums extensively. Use `serde_repr` with `#[repr(u8)]` (or `#[repr(u16)]` for larger values) for named variants:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PullZoneType {
    Premium = 0,
    Volume = 1,
}
```

**Always add a `Display` impl** for enums that will appear in CLI table output:

```rust
impl std::fmt::Display for PullZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PullZoneType::Premium => write!(f, "Premium"),
            PullZoneType::Volume => write!(f, "Volume"),
        }
    }
}
```

**Gaps in enum values:** Some bunny.net enums skip integers (e.g. `OriginType` has 0, 2, 3, 4 — no 1). `serde_repr` handles this correctly; just map the values that exist.

## 4. Required vs Optional Fields

The OpenAPI specs mark almost everything as `nullable`, but in practice many fields are always present. Our approach:

**Response types (deserialize):**
- Fields that are always present in practice: use concrete types with `#[serde(default)]`
  ```rust
  pub id: i64,           // always present
  pub name: String,      // always present
  #[serde(default)]
  pub origin_url: String, // always present but serde(default) guards against edge cases
  ```
- Fields that may genuinely be absent: use `Option<T>` with `#[serde(default)]`
  ```rust
  #[serde(default)]
  pub storage_zone_id: Option<i64>,
  ```
- Vec fields: use `Vec<T>` with `#[serde(default)]` (never `Option<Vec<T>>`)
  ```rust
  #[serde(default)]
  pub hostnames: Vec<HostnameInfo>,
  ```
- For `PaginatedList<T>`, use `#[serde(default = "Vec::new")]` on the `items` field to avoid requiring `T: Default`:
  ```rust
  #[serde(default = "Vec::new")]
  pub items: Vec<T>,
  ```

**Request types (serialize):**
- Required fields: plain types, no `Option`
  ```rust
  pub name: String,  // required by API
  ```
- Optional fields: `Option<T>` with `skip_serializing_if`
  ```rust
  #[serde(skip_serializing_if = "Option::is_none")]
  pub origin_type: Option<OriginType>,
  ```

## 5. Request Body Patterns

### Create requests: constructor + builder pattern
```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePullZone {
    pub name: String,
    pub origin_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_type: Option<PullZoneType>,
}

impl CreatePullZone {
    pub fn new(name: impl Into<String>, origin_url: impl Into<String>) -> Self {
        Self { name: name.into(), origin_url: origin_url.into(), zone_type: None }
    }
    pub fn zone_type(mut self, t: PullZoneType) -> Self {
        self.zone_type = Some(t);
        self
    }
}
```

### Update requests: Default + builder pattern
```rust
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdatePullZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_url: Option<String>,
    // ... all fields Optional
}

impl UpdatePullZone {
    pub fn new() -> Self { Self::default() }
    pub fn origin_url(mut self, url: impl Into<String>) -> Self {
        self.origin_url = Some(url.into());
        self
    }
}
```

### Named constructors for small request types
```rust
impl PurgeCache {
    pub fn all() -> Self { Self::default() }
    pub fn by_tag(tag: impl Into<String>) -> Self {
        Self { cache_tag: Some(tag.into()) }
    }
}
```

## 6. Client Structure

### One client struct per API, holding reqwest::Client + auth
```rust
pub struct BunnyClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}
```

### Constructors
- `new(api_key)` — production base URL
- `with_base_url(api_key, url)` — for testing / staging
- Some crates return `Result` from `new()` (using `Client::builder().build()?`), others use `Client::new()` directly. Prefer `Client::new()` for simplicity unless you need custom builder options.

### Auth
All bunny.net APIs use the `AccessKey` header. The auth helper pattern varies slightly by crate but the intent is the same:

```rust
// Pattern A: returns HeaderMap (Core)
fn auth_headers(&self) -> Result<HeaderMap> { ... }

// Pattern B: decorates a RequestBuilder (Stream, Compute)
fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
    rb.header("AccessKey", &self.api_key)
}
```

Pattern B is more concise. Prefer it for new code.

### Storage API: different auth model
Storage uses per-zone access keys (not the account API key), and the base URL varies by region:
```rust
StorageClient::new("storage", "zone-password")  // region + zone key
```

## 7. Error Handling per API

Each API returns different error shapes. Match the error type to the API:

| API | Error Type | JSON Shape |
|-----|-----------|------------|
| Core Platform | `ApiError` | `{"Message": "...", "ErrorKey": "...", "StatusCode": 401}` (PascalCase) |
| Stream | `StatusMessage` | `{"Success": false, "Message": "...", "StatusCode": 400}` (PascalCase) |
| Shield | `ProblemDetails` | RFC 7807: `{"type": "...", "title": "...", "status": 401, "detail": "..."}` (camelCase) |
| Edge Scripting | `ApiError` | `{"ErrorKey": "...", "Field": "...", "Message": "..."}` (PascalCase) |
| Storage | `StorageError` | `{"HttpCode": 404, "Message": "..."}` (PascalCase) |

Error types should implement `Display` and `std::error::Error` so they work with `anyhow`:

```rust
impl std::fmt::Display for ApiError { ... }
impl std::error::Error for ApiError {}
```

### Error extraction pattern
Always try to parse structured errors first, fall back to status+body text:
```rust
async fn extract_api_error(&self, status: StatusCode, response: Response) -> anyhow::Error {
    let bytes = response.bytes().await.unwrap_or_default();
    match serde_json::from_slice::<ApiError>(&bytes) {
        Ok(api_err) => anyhow::Error::new(api_err),
        Err(_) => anyhow::anyhow!("HTTP {status}: {}", String::from_utf8_lossy(&bytes)),
    }
}
```

## 8. Response Handling

### Two response patterns: JSON body vs empty
```rust
// JSON body response
async fn handle_response<T: DeserializeOwned>(&self, resp: Response) -> Result<T>

// Empty response (DELETE, some POSTs)
async fn handle_empty_response(&self, resp: Response) -> Result<()>
```

### Shield API: wrapper objects
The Shield API often wraps responses in `{ "data": ..., "error": ... }` envelopes. Create specific wrapper types:
```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetShieldZoneResponse {
    data: Option<ShieldZoneResponse>,
}
```
Then unwrap in the client method:
```rust
let wrapper: GetShieldZoneResponse = self.handle_response(resp).await?;
wrapper.data.ok_or_else(|| anyhow!("response contained no data"))
```

### Upsert endpoints: 200 vs 204 split
Some endpoints return 200 with a body on create, 204 with no body on update. Handle both:
```rust
if status == StatusCode::NO_CONTENT {
    Ok(placeholder_value)  // caller should re-fetch if full model needed
} else {
    self.json_or_error(resp).await
}
```

## 9. Query Parameters

Build query params conditionally using reqwest's `.query()`:
```rust
let mut req = self.auth(self.http.get(&url));
if let Some(p) = page {
    req = req.query(&[("page", p.to_string())]);
}
if let Some(s) = search {
    req = req.query(&[("search", s)]);
}
```

Or for the Core API pattern, manually build the query string:
```rust
let mut params: Vec<(&str, String)> = Vec::new();
if let Some(p) = page { params.push(("page", p.to_string())); }
// ... then join and append to URL
```

Prefer the reqwest `.query()` approach — it handles URL encoding automatically.

## 10. Binary Upload

For endpoints accepting `application/octet-stream` (Stream video upload, Storage file upload):
```rust
pub async fn upload_video(
    &self,
    library_id: i64,
    video_id: &str,
    body: impl Into<reqwest::Body>,
) -> Result<StatusMessage> {
    // ...
    .header("Content-Type", "application/octet-stream")
    .body(body)
    // ...
}
```

`impl Into<reqwest::Body>` accepts `Vec<u8>`, `bytes::Bytes`, `String`, and streaming via `Body::wrap_stream`.

## 11. HTTP Method Quirks

- **Core API uses POST for updates**, not PATCH/PUT:
  `POST /pullzone/{id}` with a body = update
- **Shield API uses PATCH for updates**: standard REST
- **Edge Scripting uses POST for updates**: same as Core
- **Storage uses PUT for uploads**: standard
- **Stream uses PUT for binary upload, POST for metadata updates**

Always check the spec for the correct method per endpoint.

## 12. Workspace & Crate Layout

```
hoppy/
  Cargo.toml          (workspace root, resolver = "2")
  src/                (hoppy CLI binary)
  crates/
    bunny-api-core/   (Core Platform: pull zones, DNS, storage zones, etc.)
    bunny-api-stream/ (Stream/Video API)
    bunny-api-shield/ (Shield/WAF API)
    bunny-api-compute/(Edge Scripting API)
    bunny-api-storage/(Storage file operations API)
  specs/              (original OpenAPI JSON specs for reference)
```

Crate naming: `bunny-api-*` (not `hoppy-api-*`) because these are bunny.net API clients, reusable independently of the hoppy CLI.

Each crate has:
- `src/types.rs` — all types (enums, response models, request bodies)
- `src/client.rs` — the client struct and all async methods
- `src/lib.rs` — re-exports for ergonomic `use bunny_api_core::BunnyClient` imports

### Dependencies (shared across all API crates)
```toml
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_repr = "0.1"
anyhow = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

Storage additionally needs `bytes = "1"`.

## 13. Testing Patterns

### Unit tests for serde round-trips
```rust
#[test]
fn video_status_roundtrip() {
    let status: VideoStatus = serde_json::from_str("4").unwrap();
    assert_eq!(status, VideoStatus::Finished);
    let serialised = serde_json::to_string(&VideoStatus::Error).unwrap();
    assert_eq!(serialised, "5");
}
```

### Unit tests for request body serialization
```rust
#[test]
fn update_skips_none_fields() {
    let body = UpdatePullZone::new().origin_url("https://new.example.com");
    let json = serde_json::to_value(&body).unwrap();
    assert_eq!(json["OriginUrl"], "https://new.example.com");
    assert!(json.get("StorageZoneId").is_none());  // not set, must be absent
}
```

### Unit tests for client construction
```rust
#[test]
fn new_client_accepts_string_api_key() {
    let client = BunnyClient::new("test-key").unwrap();
    assert_eq!(client.api_key, "test-key");
}
```

### No live API tests yet
Integration tests against the live API will come in iteration 1. For now, all tests are pure unit tests that don't hit the network.

## 14. What NOT to Include in API Crates

- **No CLI logic** — the API crates are pure HTTP clients. CLI output formatting, table rendering, progress bars, and user interaction belong in the `hoppy` binary crate.
- **No env var reading** — API key comes from the caller, not `std::env`.
- **No retry logic** — keep the client simple. Retry/backoff can be added as middleware later if needed.
- **No rate limiting** — same as retry.
- **Don't model every field** — only include fields the CLI will actually use or display. More can be added incrementally. This keeps the types manageable and avoids over-engineering.

## Related
- [[api/bunny-api-quirks]] — quirks these patterns address
- [[api/bunny-api-overview]] — API overview
- [[iterations/iteration-1-code-review]] — code review that refined these patterns
- [[decision-log]] — architectural decisions
- [[adding-a-feature]] — feature checklist using these patterns
