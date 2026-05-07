//! Types for the bunny.net Database control-plane API.
//!
//! Captured from `specs/database.json` (API spec version 0.0.130, 2026-05-05).
//! v1 and v2 are exposed as parallel types (`Database` vs `Database2`,
//! `CreateDatabasePayload` vs `CreateDatabaseV2Payload`, etc.) within this
//! single module — unifying them would silently lose fields one side or the
//! other doesn't model.
//!
//! Region taxonomy:
//! - `storage_region` — flat storage region (e.g. `eu-west-1`). Modelled as
//!   `String` for forward-compat (bunny adds them silently).
//! - `primary_regions` / `replicas_regions` — compute region codes
//!   (e.g. `DE`, `AT`, `AMS`). Also modelled as `String` for the same reason.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Authorization (token scope)
// ---------------------------------------------------------------------------

/// Scope of an auth-token. Spec defines two values; serialised as kebab-case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Authorization {
    FullAccess,
    ReadOnly,
}

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------
//
// `DatabaseId` and `DatabaseGroupId` are stringly-typed in the spec
// (`db_<ulid>` / `group_<ulid>`). We use `String` directly rather than
// newtypes to keep the API surface ergonomic for callers.

// ---------------------------------------------------------------------------
// Database (v1)
// ---------------------------------------------------------------------------

/// A bunny.net Database (libSQL).
///
/// Returned by v1 endpoints (`GET /v1/databases`, `GET /v1/databases/{db_id}`,
/// `POST /v1/databases`). The `url` field is the libSQL data-plane URL —
/// preserve casing AND trailing slash exactly when passing it to libSQL
/// clients (some reject normalised URLs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
    pub id: String,
    pub url: String,
    pub block_reads: bool,
    pub block_writes: bool,
    pub allow_attach: bool,
    pub group_id: String,
    pub group_name: String,
    pub is_schema: bool,
    /// Parent database whose schema this database inherits, if any.
    #[serde(default)]
    pub schema: Option<String>,
    pub version: String,
    pub size_max: String,
    pub current_size: String,
}

/// Wrapper for `POST /v1/databases` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseResponse {
    pub database: Database,
}

/// Wrapper for `GET /v1/databases/{db_id}` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDatabaseResponse {
    pub database: Database,
}

/// Wrapper for `GET /v1/databases` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDatabaseResponse {
    pub databases: Vec<Database>,
}

/// Wrapper for `DELETE /v1/databases/{db_id}` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteDatabaseResponse {
    pub database: String,
}

// ---------------------------------------------------------------------------
// Database group
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseGroup {
    pub id: String,
    pub name: String,
    pub storage_region: String,
    pub primary_regions: Vec<String>,
    pub replicas_regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseGroupResponse {
    pub group: DatabaseGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDatabaseGroupResponse {
    pub group: DatabaseGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDatabaseGroupResponse {
    pub groups: Vec<DatabaseGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedGroupResponse {
    pub group: DatabaseGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseGroupResponse {
    pub group: DatabaseGroup,
}

// ---------------------------------------------------------------------------
// Request bodies (v1)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CreateDatabasePayload {
    pub slug: String,
    pub group: String,
}

impl CreateDatabasePayload {
    pub fn new(slug: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            group: group.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDatabaseGroupPayload {
    pub display_name: String,
    pub storage_region: String,
    pub primary_regions: Vec<String>,
    pub replicas_regions: Vec<String>,
}

impl CreateDatabaseGroupPayload {
    pub fn new(
        display_name: impl Into<String>,
        storage_region: impl Into<String>,
        primary_regions: Vec<String>,
        replicas_regions: Vec<String>,
    ) -> Self {
        Self {
            display_name: display_name.into(),
            storage_region: storage_region.into(),
            primary_regions,
            replicas_regions,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateDatabaseGroupPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateTokenDatabasePayload {
    pub authorization: Authorization,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

impl GenerateTokenDatabasePayload {
    pub fn new(authorization: Authorization) -> Self {
        Self {
            authorization,
            expires_at: None,
        }
    }
    pub fn expires_at(mut self, ts: impl Into<String>) -> Self {
        self.expires_at = Some(ts.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateTokenDatabaseGroupPayload {
    pub authorization: Authorization,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

impl GenerateTokenDatabaseGroupPayload {
    pub fn new(authorization: Authorization) -> Self {
        Self {
            authorization,
            expires_at: None,
        }
    }
    pub fn expires_at(mut self, ts: impl Into<String>) -> Self {
        self.expires_at = Some(ts.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateTokenResponse {
    pub token: String,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForkDatabasePayload {
    pub slug: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkDatabaseResponse {
    pub database: Database,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreVersionDatabasePayload {
    pub generation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreDatabaseResponse {
    /// UUID of the restored generation.
    pub generation: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListVersionsDatabasePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub older_than: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newer_than: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Generation {
    pub created_at: String,
    pub generation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVersionsResponse {
    pub generations: Vec<Generation>,
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Database v2
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database2 {
    pub name: String,
    pub id: String,
    pub url: String,
    pub block_reads: bool,
    pub block_writes: bool,
    /// Deprecated; use `size_max_bytes`.
    pub size_max: String,
    /// Deprecated; use `current_size_bytes`.
    pub current_size: String,
    pub size_max_bytes: u64,
    pub current_size_bytes: u64,
    pub storage_region: String,
    pub primary_regions: Vec<String>,
    pub replicas_regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseV2PageInfo {
    pub current_page: i64,
    pub total_items: i64,
    pub has_more_items: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDatabaseV2Response {
    pub databases: Vec<Database2>,
    pub page_info: DatabaseV2PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadDatabaseV2Response {
    pub database: Database2,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDatabaseV2Payload {
    pub name: String,
    pub storage_region: String,
    pub primary_regions: Vec<String>,
    pub replicas_regions: Vec<String>,
}

impl CreateDatabaseV2Payload {
    pub fn new(
        name: impl Into<String>,
        storage_region: impl Into<String>,
        primary_regions: Vec<String>,
        replicas_regions: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            storage_region: storage_region.into(),
            primary_regions,
            replicas_regions,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDatabaseV2Response {
    pub db_id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateDatabaseV2Payload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDatabaseV2Response {
    pub database: Database2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletedDatabaseV2Response {
    pub db_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateTokenDatabaseV2Payload {
    pub authorization: Authorization,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

impl GenerateTokenDatabaseV2Payload {
    pub fn new(authorization: Authorization) -> Self {
        Self {
            authorization,
            expires_at: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Statistics / usage / active
// ---------------------------------------------------------------------------

/// `[timestamp, value]` data point. The spec types it as a heterogeneous tuple.
pub type Datapoint = (String, f64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleRegionChart {
    pub data: Vec<Datapoint>,
    #[serde(default)]
    pub diff: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyChartData {
    pub data: Vec<Datapoint>,
    #[serde(default)]
    pub diff: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySingleRegionChart {
    /// Spec exposes `unit` as a string enum (`NoUnit | milliseconds | megabyte`).
    /// Modelled as `String` for forward-compat.
    pub unit: String,
    pub data: std::collections::BTreeMap<String, LatencyChartData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub row_read_count: SingleRegionChart,
    pub row_write_count: SingleRegionChart,
    pub delegated_write_requests: SingleRegionChart,
    pub storage: SingleRegionChart,
    pub latency: LatencySingleRegionChart,
    pub query_count: SingleRegionChart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub rows_read: u64,
    pub rows_written: u64,
    pub avg_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStatsResponse {
    pub active_db: u64,
    pub total_db: i64,
    pub total_db_size: String,
}

// ---------------------------------------------------------------------------
// Live metrics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum LiveStatus {
    Live {
        metadata: LiveStatusMetadata,
    },
    ReplicaOnly,
    Offline,
    /// Forward-compat: any unknown future state.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStatusMetadata {
    pub main: String,
    pub replicas: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetricsForDBResponse {
    pub live_metrics: std::collections::BTreeMap<String, LiveStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveMetricsForGroupResponse {
    pub live_metrics: std::collections::BTreeMap<String, LiveStatus>,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRegion {
    pub id: String,
    pub name: String,
    /// Region group (e.g. `EU`, `NA`). Modelled as `String` for forward-compat.
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub group: String,
    #[serde(default)]
    pub country: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListConfigResponse {
    pub storage_region_available: Vec<StorageRegion>,
    pub primary_regions: Vec<Region>,
    pub replica_regions: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsResponse {
    pub current_databases: u32,
    pub max_databases: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalConfigResponse {
    pub storage_region: StorageRegion,
    pub primary_regions: Vec<Region>,
    pub replica_regions: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalSingleConfigResponse {
    pub storage_region: StorageRegion,
    #[serde(default)]
    pub region: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    pub error: String,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Hoppy-only: ping result for `db ping`
// ---------------------------------------------------------------------------

/// Result of a libSQL `SELECT 1` against the data-plane endpoint.
///
/// Returned as a typed value (rather than a free-form message) so it's easy
/// to consume in CI gates (the field-report's primary use case).
#[derive(Debug, Clone, Serialize)]
pub struct PingResult {
    pub ok: bool,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_serialises_kebab() {
        assert_eq!(
            serde_json::to_string(&Authorization::FullAccess).unwrap(),
            "\"full-access\""
        );
        assert_eq!(
            serde_json::to_string(&Authorization::ReadOnly).unwrap(),
            "\"read-only\""
        );
    }

    #[test]
    fn database_deserialises_minimal() {
        let json = r#"{
            "name": "my-app",
            "id": "db_01H0",
            "url": "libsql://group_01-my-app.lite.bunnydb.net/",
            "block_reads": false,
            "block_writes": false,
            "allow_attach": false,
            "group_id": "group_01",
            "group_name": "EU",
            "is_schema": false,
            "version": "0.24.30",
            "size_max": "10737418240",
            "current_size": "16384"
        }"#;
        let db: Database = serde_json::from_str(json).unwrap();
        assert_eq!(db.id, "db_01H0");
        assert!(db.url.ends_with('/'));
        assert!(db.schema.is_none());
    }

    #[test]
    fn live_status_unknown_fallback() {
        let json = r#"{"state":"FuturePending"}"#;
        let status: LiveStatus = serde_json::from_str(json).unwrap();
        assert!(matches!(status, LiveStatus::Unknown));
    }

    #[test]
    fn live_status_live_with_metadata() {
        let json = r#"{"state":"Live","metadata":{"main":"DE","replicas":["FR"]}}"#;
        let status: LiveStatus = serde_json::from_str(json).unwrap();
        match status {
            LiveStatus::Live { metadata } => {
                assert_eq!(metadata.main, "DE");
                assert_eq!(metadata.replicas, vec!["FR".to_owned()]);
            }
            _ => panic!("expected Live"),
        }
    }

    #[test]
    fn create_payload_serialises() {
        let body = CreateDatabasePayload::new("my-app", "group_123");
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v["slug"], "my-app");
        assert_eq!(v["group"], "group_123");
    }

    #[test]
    fn token_payload_skips_none_expires() {
        let body = GenerateTokenDatabasePayload::new(Authorization::FullAccess);
        let v = serde_json::to_value(&body).unwrap();
        assert_eq!(v["authorization"], "full-access");
        assert!(v.get("expires_at").is_none());
    }

    #[test]
    fn datapoint_tuple_deserialises() {
        let json = r#"["2026-05-07T00:00:00Z", 1.5]"#;
        let dp: Datapoint = serde_json::from_str(json).unwrap();
        assert_eq!(dp.0, "2026-05-07T00:00:00Z");
        assert!((dp.1 - 1.5).abs() < f64::EPSILON);
    }
}
