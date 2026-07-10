//! Types for the bunny.net Origin Errors API.
//!
//! Field names mirror `specs/origin-errors.json`. The single endpoint returns a
//! [`LogResponse`] wrapping a list of [`OriginErrorEntry`] rows.

use serde::{Deserialize, Serialize};

/// Structured labels attached to an origin error log entry.
///
/// All fields are optional — the API only populates the labels it has.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OriginErrorLabels {
    /// Machine-readable error code (e.g. `dns_lookup`).
    #[serde(default)]
    pub error_code: Option<String>,
    /// HTTP status code the edge returned (as a string, e.g. `502`).
    #[serde(default)]
    pub status_code: Option<String>,
    /// Server zone / edge location that recorded the error (e.g. `CA`).
    #[serde(default)]
    pub server_zone: Option<String>,
}

/// A single origin error log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginErrorEntry {
    /// Unique identifier for the log entry (UUID).
    #[serde(default)]
    pub log_id: Option<String>,
    /// Unix millisecond timestamp of the error.
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// Raw JSON-encoded log line (`RequestUrl`, `Message`, `ErrorCode`, ...).
    #[serde(default)]
    pub log: Option<String>,
    /// Structured labels extracted from the log line.
    #[serde(default)]
    pub labels: Option<OriginErrorLabels>,
}

/// Response wrapper for an origin-error query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResponse {
    /// Origin error entries for the requested pull zone and date.
    #[serde(default)]
    pub logs: Vec<OriginErrorEntry>,
}
