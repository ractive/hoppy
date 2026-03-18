use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// The billing / performance tier of a Pull Zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PullZoneType {
    Premium = 0,
    Volume = 1,
}

impl std::fmt::Display for PullZoneType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PullZoneType::Premium => write!(f, "Premium"),
            PullZoneType::Volume => write!(f, "Volume"),
        }
    }
}

/// Where the Pull Zone fetches its origin content from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OriginType {
    OriginUrl = 0,
    StorageZone = 2,
    LoadBalancer = 3,
    Script = 4,
}

impl std::fmt::Display for OriginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginType::OriginUrl => write!(f, "OriginUrl"),
            OriginType::StorageZone => write!(f, "StorageZone"),
            OriginType::LoadBalancer => write!(f, "LoadBalancer"),
            OriginType::Script => write!(f, "Script"),
        }
    }
}

// ---------------------------------------------------------------------------
// Response models
// ---------------------------------------------------------------------------

/// A hostname attached to a Pull Zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HostnameInfo {
    pub id: i64,
    pub value: String,
    #[serde(default)]
    pub force_ssl: bool,
    #[serde(default)]
    pub has_certificate: bool,
    #[serde(default)]
    pub is_system_hostname: bool,
}

/// The ~20 most important fields of a bunny.net Pull Zone.
///
/// Fields that bunny.net may omit on older zones are annotated with
/// `#[serde(default)]` so they deserialise to their `Default` value
/// instead of failing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PullZone {
    pub id: i64,
    pub name: String,

    // Origin
    #[serde(default)]
    pub origin_url: String,
    #[serde(default)]
    pub origin_type: Option<OriginType>,
    #[serde(default)]
    pub storage_zone_id: Option<i64>,

    // Network / routing
    #[serde(default)]
    pub cname_domain: String,
    #[serde(default)]
    pub hostnames: Vec<HostnameInfo>,
    #[serde(default)]
    pub zone_type: Option<PullZoneType>,

    // Status
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub suspended: bool,

    // Bandwidth
    #[serde(default)]
    pub monthly_bandwidth_used: i64,
    #[serde(default)]
    pub monthly_bandwidth_limit: i64,

    // Cache
    #[serde(default)]
    pub cache_version: i64,
    #[serde(default)]
    pub cache_expiration_time: i64,

    // Security
    #[serde(default)]
    pub zone_security_enabled: bool,
    #[serde(default, skip_serializing)]
    pub zone_security_key: String,

    // Geo zones
    #[serde(default)]
    pub enable_geo_zone_us: bool,
    #[serde(default)]
    pub enable_geo_zone_eu: bool,
    #[serde(default)]
    pub enable_geo_zone_asia: bool,
    #[serde(default)]
    pub enable_geo_zone_sa: bool,
    #[serde(default)]
    pub enable_geo_zone_af: bool,
}

/// Generic paginated list response returned by the bunny.net API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaginatedList<T> {
    pub items: Vec<T>,
    pub current_page: i64,
    pub total_items: i64,
    pub has_more_items: bool,
}

/// Structured API error returned by bunny.net in 4xx/5xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiError {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub error_key: String,
    #[serde(default)]
    pub status_code: u16,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "bunny.net API error {} ({}): {}",
            self.status_code, self.error_key, self.message
        )
    }
}

impl std::error::Error for ApiError {}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

/// Request body for `POST /pullzone` — create a new Pull Zone.
///
/// Only `name` and `origin_url` are required by the API; all other fields
/// are optional and default to the API's own defaults when absent.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePullZone {
    pub name: String,
    pub origin_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_type: Option<OriginType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_zone_id: Option<i64>,
    #[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
    pub zone_type: Option<PullZoneType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_bandwidth_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_security_enabled: Option<bool>,
}

impl CreatePullZone {
    /// Create the minimum-viable request: a name and an origin URL.
    pub fn new(name: impl Into<String>, origin_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            origin_url: origin_url.into(),
            origin_type: None,
            storage_zone_id: None,
            zone_type: None,
            monthly_bandwidth_limit: None,
            zone_security_enabled: None,
        }
    }

    #[must_use]
    pub fn origin_type(mut self, t: OriginType) -> Self {
        self.origin_type = Some(t);
        self
    }

    #[must_use]
    pub fn storage_zone_id(mut self, id: i64) -> Self {
        self.storage_zone_id = Some(id);
        self
    }

    #[must_use]
    pub fn zone_type(mut self, t: PullZoneType) -> Self {
        self.zone_type = Some(t);
        self
    }

    #[must_use]
    pub fn monthly_bandwidth_limit(mut self, limit: i64) -> Self {
        self.monthly_bandwidth_limit = Some(limit);
        self
    }

    #[must_use]
    pub fn zone_security_enabled(mut self, enabled: bool) -> Self {
        self.zone_security_enabled = Some(enabled);
        self
    }
}

/// Request body for `POST /pullzone/{id}` — update an existing Pull Zone.
///
/// Every field is optional; only non-`None` fields are serialised.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdatePullZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_type: Option<OriginType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_zone_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_bandwidth_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_expiration_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_security_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_geo_zone_us: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_geo_zone_eu: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_geo_zone_asia: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_geo_zone_sa: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_geo_zone_af: Option<bool>,
}

impl UpdatePullZone {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn origin_url(mut self, url: impl Into<String>) -> Self {
        self.origin_url = Some(url.into());
        self
    }

    #[must_use]
    pub fn origin_type(mut self, t: OriginType) -> Self {
        self.origin_type = Some(t);
        self
    }

    #[must_use]
    pub fn storage_zone_id(mut self, id: i64) -> Self {
        self.storage_zone_id = Some(id);
        self
    }

    #[must_use]
    pub fn monthly_bandwidth_limit(mut self, limit: i64) -> Self {
        self.monthly_bandwidth_limit = Some(limit);
        self
    }

    #[must_use]
    pub fn cache_expiration_time(mut self, secs: i64) -> Self {
        self.cache_expiration_time = Some(secs);
        self
    }

    #[must_use]
    pub fn zone_security_enabled(mut self, enabled: bool) -> Self {
        self.zone_security_enabled = Some(enabled);
        self
    }
}

// ---------------------------------------------------------------------------
// Storage Zone types
// ---------------------------------------------------------------------------

/// A bunny.net Storage Zone.
///
/// `Password` and `ReadOnlyPassword` are deserialized but never serialized
/// to prevent accidental exposure in JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageZone {
    pub id: i64,
    pub user_id: String,
    pub name: String,
    #[serde(default, skip_serializing)]
    pub password: String,
    #[serde(default)]
    pub date_modified: String,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default)]
    pub storage_used: i64,
    #[serde(default)]
    pub files_stored: i64,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub replication_regions: Vec<String>,
    /// Complex nested object — passed through as raw JSON.
    #[serde(default)]
    pub pull_zones: Option<serde_json::Value>,
    #[serde(default, skip_serializing)]
    pub read_only_password: String,
    #[serde(default)]
    pub rewrite_404_to_200: bool,
    /// Custom 404 file path — nullable in the API response.
    #[serde(default)]
    pub custom_404_file_path: Option<String>,
    #[serde(default)]
    pub storage_hostname: String,
    #[serde(default)]
    pub zone_tier: i64,
    #[serde(default)]
    pub replication_change_in_progress: bool,
    #[serde(default)]
    pub price_override: f64,
    #[serde(default)]
    pub discount: i64,
    #[serde(default)]
    pub storage_zone_type: i64,
}

/// Request body for `POST /storagezone` — create a new Storage Zone.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateStorageZone {
    pub name: String,
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication_regions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_tier: Option<i64>,
}

impl CreateStorageZone {
    pub fn new(name: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            region: region.into(),
            replication_regions: None,
            zone_tier: None,
        }
    }

    #[must_use]
    pub fn replication_regions(mut self, regions: Vec<String>) -> Self {
        self.replication_regions = Some(regions);
        self
    }

    #[must_use]
    pub fn zone_tier(mut self, tier: i64) -> Self {
        self.zone_tier = Some(tier);
        self
    }
}

/// Request body for `POST /storagezone/{id}` — update an existing Storage Zone.
///
/// Every field is optional; only non-`None` fields are serialised.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateStorageZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication_zones: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_404_file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite_404_to_200: Option<bool>,
}

impl UpdateStorageZone {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn replication_zones(mut self, zones: Vec<String>) -> Self {
        self.replication_zones = Some(zones);
        self
    }

    #[must_use]
    pub fn origin_url(mut self, url: impl Into<String>) -> Self {
        self.origin_url = Some(url.into());
        self
    }

    #[must_use]
    pub fn custom_404_file_path(mut self, path: impl Into<String>) -> Self {
        self.custom_404_file_path = Some(path.into());
        self
    }

    #[must_use]
    pub fn rewrite_404_to_200(mut self, rewrite: bool) -> Self {
        self.rewrite_404_to_200 = Some(rewrite);
        self
    }
}

/// Request body for `POST /pullzone/{id}/purgeCache`.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PurgeCache {
    /// Optional cache tag to limit the purge scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_tag: Option<String>,
}

impl PurgeCache {
    /// Purge everything (no tag filter).
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Purge only entries matching this cache tag.
    pub fn by_tag(tag: impl Into<String>) -> Self {
        Self {
            cache_tag: Some(tag.into()),
        }
    }
}
