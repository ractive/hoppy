//! Types for the bunny.net CDN Logging API (v2 structured responses).
//!
//! Field names mirror `specs/logging.json`. The v2 endpoint returns a
//! [`LogQueryResponse`] envelope carrying the matched [`LogEntry`] rows plus
//! [`PaginationInfo`] and a [`QuerySummary`]. The v1 (legacy) endpoint returns
//! raw pipe-delimited text and is streamed rather than deserialized.

use serde::{Deserialize, Serialize};

/// A single CDN access log entry returned by the v2 logging endpoint.
///
/// Nullability mirrors the underlying data: fields sourced from optional HTTP
/// headers (`referer`, `user_agent`, `content_range`, `authorization_header`)
/// are `None` when the header was absent. Extended fields (`body_bytes_sent`,
/// `content_range`, `authorization_header`) are only populated when extended
/// logging is enabled for the pull zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// Time the request was received at the edge (UTC, millisecond precision).
    pub timestamp: String,
    /// Pull zone identifier the request was served from.
    pub pull_zone_id: i64,
    /// Unique identifier for the request (32-char hex).
    #[serde(default)]
    pub request_id: Option<String>,
    /// Cache status reported by the edge (e.g. `HIT`, `MISS`, `EXPIRED`, `STALE`).
    #[serde(default)]
    pub cache_status: Option<String>,
    /// HTTP response status code.
    pub status_code: i32,
    /// Total bytes sent in the response (headers + body).
    pub bytes_sent: i64,
    /// Client IP address. May be anonymized when IP anonymization is enabled.
    #[serde(default)]
    pub remote_ip: Option<String>,
    /// ISO 3166 alpha-2 country code derived from the client IP.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Edge location / server zone that handled the request.
    #[serde(default)]
    pub edge_location: Option<String>,
    /// Request scheme (`http` or `https`).
    #[serde(default)]
    pub scheme: Option<String>,
    /// Request `Host` header.
    #[serde(default)]
    pub host: Option<String>,
    /// Request URI path with query string.
    #[serde(default)]
    pub path: Option<String>,
    /// Fully composed URL (`{scheme}://{host}{path}`).
    #[serde(default)]
    pub url: Option<String>,
    /// HTTP `User-Agent` header. `None` when absent.
    #[serde(default)]
    pub user_agent: Option<String>,
    /// HTTP `Referer` header. `None` when absent.
    #[serde(default)]
    pub referer: Option<String>,
    /// Body-only bytes sent (extended logging only).
    #[serde(default)]
    pub body_bytes_sent: Option<i64>,
    /// HTTP `Content-Range` header (extended logging only).
    #[serde(default)]
    pub content_range: Option<String>,
    /// Decrypted HTTP `Authorization` header (extended logging only).
    #[serde(default)]
    pub authorization_header: Option<String>,
    /// JA4 TLS client fingerprint. `None` when absent.
    #[serde(default)]
    pub ja4_fingerprint: Option<String>,
    /// Autonomous System Number derived from the client IP. `None` if unknown.
    #[serde(default)]
    pub asn: Option<i32>,
    /// Name of the organization that owns the AS. `None` if unknown.
    #[serde(default)]
    pub asn_organization: Option<String>,
}

/// Pagination metadata returned with a [`LogQueryResponse`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    /// Offset that was applied to this query.
    pub offset: i64,
    /// Limit that was applied to this query.
    pub limit: i32,
    /// Number of entries actually returned (`<= limit`).
    pub returned: i32,
    /// True if more results are available beyond this page.
    pub has_more: bool,
}

/// Echo of the effective query parameters the server applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuerySummary {
    pub pull_zone_id: i64,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub order: Option<String>,
}

/// Paginated response wrapper for a v2 log query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQueryResponse {
    /// Log entries matching the query, in the requested sort order.
    #[serde(default)]
    pub data: Vec<LogEntry>,
    pub pagination: PaginationInfo,
    pub query: QuerySummary,
}

/// Structured error body returned by v2 logging endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    /// Machine-readable error code (snake_case).
    #[serde(default)]
    pub code: Option<String>,
    /// Human-readable message describing the error.
    #[serde(default)]
    pub message: Option<String>,
    /// Optional per-field validation messages.
    #[serde(default)]
    pub details: Option<Vec<String>>,
}

/// Structured error envelope returned by v2 logging endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

/// Filters for a v2 CDN access-log query.
///
/// Every field maps to a query parameter documented in `specs/logging.json`.
/// All are optional; unset fields are simply not sent. Constructed via
/// [`LogQueryParams::default`] and populated with the builder-style setters, or
/// with struct-update syntax.
#[derive(Debug, Clone, Default)]
pub struct LogQueryParams {
    /// Inclusive start of the time range (UTC, RFC 3339). Defaults server-side to `to - 24h`.
    pub from: Option<String>,
    /// Exclusive end of the time range (UTC, RFC 3339). Defaults server-side to `now`.
    pub to: Option<String>,
    /// Comma-separated HTTP status filters (exact codes or classes like `2xx`).
    pub status: Option<String>,
    /// Comma-separated cache statuses to match exactly (e.g. `HIT,MISS`).
    pub cache_status: Option<String>,
    /// ISO 3166 alpha-2 country code(s), comma-separated.
    pub country: Option<String>,
    /// Edge location / server zone (exact match).
    pub edge_location: Option<String>,
    /// Client IP address filter (IPv4 or IPv6).
    pub remote_ip: Option<String>,
    /// Case-insensitive substring match against the request URL.
    pub url_contains: Option<String>,
    /// Case-insensitive substring match against the `User-Agent` header.
    pub user_agent_contains: Option<String>,
    /// Case-insensitive substring match against the `Referer` header.
    pub referer_contains: Option<String>,
    /// Free-text, case-insensitive token search.
    pub search: Option<String>,
    /// Exact request ID (UUID) to look up a single log entry.
    pub request_id: Option<String>,
    /// Include origin-shield (edge → shield) requests. Defaults to `false`.
    pub include_origin_shield: Option<bool>,
    /// Maximum entries to return (default 100, capped at 10000).
    pub limit: Option<i32>,
    /// Number of entries to skip (default 0).
    pub offset: Option<i64>,
    /// Sort order by timestamp: `asc` or `desc` (default).
    pub order: Option<String>,
}

/// Filters for the v1 (legacy) raw-log query.
///
/// The v1 endpoint returns raw pipe-delimited text and is streamed rather than
/// parsed. `date` (path) is passed separately to the client method.
#[derive(Debug, Clone, Default)]
pub struct LegacyLogParams {
    /// Unix millisecond timestamp — inclusive start of the range.
    pub start: Option<i64>,
    /// Unix millisecond timestamp — exclusive end of the range.
    pub end: Option<i64>,
    /// Sort order (`asc` / `desc`).
    pub sort: Option<String>,
    /// HTTP status filter.
    pub status: Option<String>,
    /// Free-text search.
    pub search: Option<String>,
    /// Request the response as a downloadable attachment.
    pub download: Option<bool>,
}
