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

// ---------------------------------------------------------------------------
// DNS Zone types
// ---------------------------------------------------------------------------

/// DNS record type values used by the bunny.net API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DnsRecordType {
    A = 0,
    AAAA = 1,
    CNAME = 2,
    TXT = 3,
    MX = 4,
    Redirect = 5,
    Flatten = 6,
    PullZone = 7,
    SRV = 8,
    CAA = 9,
    PTR = 10,
    Script = 11,
    NS = 12,
    SVCB = 13,
    HTTPS = 14,
    TLSA = 15,
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsRecordType::A => write!(f, "A"),
            DnsRecordType::AAAA => write!(f, "AAAA"),
            DnsRecordType::CNAME => write!(f, "CNAME"),
            DnsRecordType::TXT => write!(f, "TXT"),
            DnsRecordType::MX => write!(f, "MX"),
            DnsRecordType::Redirect => write!(f, "Redirect"),
            DnsRecordType::Flatten => write!(f, "Flatten"),
            DnsRecordType::PullZone => write!(f, "PullZone"),
            DnsRecordType::SRV => write!(f, "SRV"),
            DnsRecordType::CAA => write!(f, "CAA"),
            DnsRecordType::PTR => write!(f, "PTR"),
            DnsRecordType::Script => write!(f, "Script"),
            DnsRecordType::NS => write!(f, "NS"),
            DnsRecordType::SVCB => write!(f, "SVCB"),
            DnsRecordType::HTTPS => write!(f, "HTTPS"),
            DnsRecordType::TLSA => write!(f, "TLSA"),
        }
    }
}

impl std::str::FromStr for DnsRecordType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(DnsRecordType::A),
            "AAAA" => Ok(DnsRecordType::AAAA),
            "CNAME" => Ok(DnsRecordType::CNAME),
            "TXT" => Ok(DnsRecordType::TXT),
            "MX" => Ok(DnsRecordType::MX),
            "REDIRECT" => Ok(DnsRecordType::Redirect),
            "FLATTEN" => Ok(DnsRecordType::Flatten),
            "PULLZONE" => Ok(DnsRecordType::PullZone),
            "SRV" => Ok(DnsRecordType::SRV),
            "CAA" => Ok(DnsRecordType::CAA),
            "PTR" => Ok(DnsRecordType::PTR),
            "SCRIPT" => Ok(DnsRecordType::Script),
            "NS" => Ok(DnsRecordType::NS),
            "SVCB" => Ok(DnsRecordType::SVCB),
            "HTTPS" => Ok(DnsRecordType::HTTPS),
            "TLSA" => Ok(DnsRecordType::TLSA),
            _ => anyhow::bail!("unknown DNS record type: {s}"),
        }
    }
}

/// A DNS record within a zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsRecord {
    pub id: i64,
    #[serde(rename = "Type")]
    pub record_type: DnsRecordType,
    #[serde(default)]
    pub ttl: i32,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub weight: i32,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub port: i32,
    #[serde(default)]
    pub flags: u8,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub accelerated: bool,
    #[serde(default)]
    pub accelerated_pull_zone_id: i64,
    #[serde(default)]
    pub link_name: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub comment: Option<String>,
}

/// A bunny.net DNS zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsZone {
    pub id: i64,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub records: Vec<DnsRecord>,
    #[serde(default)]
    pub date_modified: String,
    #[serde(default)]
    pub date_created: String,
    #[serde(default)]
    pub nameservers_detected: bool,
    #[serde(default)]
    pub custom_nameservers_enabled: bool,
    #[serde(default)]
    pub nameserver1: Option<String>,
    #[serde(default)]
    pub nameserver2: Option<String>,
    #[serde(default)]
    pub soa_email: Option<String>,
    #[serde(default)]
    pub nameservers_next_check: String,
    #[serde(default)]
    pub logging_enabled: bool,
    #[serde(default)]
    pub logging_ip_anonymization_enabled: bool,
    #[serde(default)]
    pub dns_sec_enabled: bool,
}

/// Request body for `POST /dnszone` — create a new DNS zone.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateDnsZone {
    pub domain: String,
}

impl CreateDnsZone {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
        }
    }
}

/// Request body for `POST /dnszone/{id}` — update a DNS zone.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateDnsZone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_nameservers_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameserver1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nameserver2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soa_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging_ip_anonymization_enabled: Option<bool>,
}

impl UpdateDnsZone {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn custom_nameservers_enabled(mut self, enabled: bool) -> Self {
        self.custom_nameservers_enabled = Some(enabled);
        self
    }

    #[must_use]
    pub fn nameserver1(mut self, ns: impl Into<String>) -> Self {
        self.nameserver1 = Some(ns.into());
        self
    }

    #[must_use]
    pub fn nameserver2(mut self, ns: impl Into<String>) -> Self {
        self.nameserver2 = Some(ns.into());
        self
    }

    #[must_use]
    pub fn soa_email(mut self, email: impl Into<String>) -> Self {
        self.soa_email = Some(email.into());
        self
    }

    #[must_use]
    pub fn logging_enabled(mut self, enabled: bool) -> Self {
        self.logging_enabled = Some(enabled);
        self
    }

    #[must_use]
    pub fn logging_ip_anonymization_enabled(mut self, enabled: bool) -> Self {
        self.logging_ip_anonymization_enabled = Some(enabled);
        self
    }
}

// ---------------------------------------------------------------------------
// Video Library types
// ---------------------------------------------------------------------------

/// A bunny.net Video Library (Stream).
///
/// `ApiKey` and `ReadOnlyApiKey` are deserialized but never serialized
/// to prevent accidental exposure in JSON output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VideoLibrary {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub video_count: i64,
    #[serde(default)]
    pub traffic_usage: i64,
    #[serde(default)]
    pub storage_usage: i64,
    #[serde(default)]
    pub date_created: String,
    #[serde(default, skip_serializing)]
    pub api_key: String,
    #[serde(default, skip_serializing)]
    pub read_only_api_key: String,
    #[serde(default)]
    pub has_watermark: bool,
    #[serde(default)]
    pub pull_zone_id: i64,
    #[serde(default)]
    pub storage_zone_id: i64,
    #[serde(default)]
    pub enabled_resolutions: Option<String>,
    #[serde(default)]
    pub replication_regions: Vec<String>,
    #[serde(default)]
    pub allow_direct_play: bool,
    #[serde(rename = "EnableMP4Fallback", default)]
    pub enable_mp4_fallback: bool,
}

/// Request body for `POST /videolibrary` — create a new Video Library.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateVideoLibrary {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication_regions: Option<Vec<String>>,
}

impl CreateVideoLibrary {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            replication_regions: None,
        }
    }

    #[must_use]
    pub fn replication_regions(mut self, regions: Vec<String>) -> Self {
        self.replication_regions = Some(regions);
        self
    }
}

/// Request body for `POST /videolibrary/{id}` — update an existing Video Library.
///
/// All fields are optional; only non-`None` values are serialised.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateVideoLibrary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_direct_play: Option<bool>,
    #[serde(rename = "EnableMP4Fallback", skip_serializing_if = "Option::is_none")]
    pub enable_mp4_fallback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_watermark: Option<bool>,
}

impl UpdateVideoLibrary {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn allow_direct_play(mut self, allow: bool) -> Self {
        self.allow_direct_play = Some(allow);
        self
    }

    #[must_use]
    pub fn enable_mp4_fallback(mut self, enable: bool) -> Self {
        self.enable_mp4_fallback = Some(enable);
        self
    }

    #[must_use]
    pub fn has_watermark(mut self, has: bool) -> Self {
        self.has_watermark = Some(has);
        self
    }
}

// ---------------------------------------------------------------------------
// Billing / account types
// ---------------------------------------------------------------------------

/// Key account and billing details returned by `GET /billing`.
///
/// Only the most actionable fields are modelled here. All are marked
/// `#[serde(default)]` because bunny.net may omit optional fields on some
/// account types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct BillingDetails {
    /// Current account credit balance (USD).
    #[serde(default)]
    pub balance: f64,
    /// Charges accumulated so far this calendar month (USD).
    #[serde(default)]
    pub this_month_charges: f64,
    /// Whether billing is enabled on the account.
    #[serde(default)]
    pub billing_enabled: bool,
    /// Minimum monthly spending commitment (USD), 0 if none.
    #[serde(default)]
    pub minimum_monthly_commit: f64,
    /// Whether automatic recharge is enabled.
    #[serde(default)]
    pub automatic_recharge_enabled: bool,
    /// Balance threshold that triggers an automatic recharge (USD).
    #[serde(default)]
    pub automatic_recharge_treshold: f64,
    /// Amount charged on each automatic recharge (USD).
    #[serde(default)]
    pub automatic_payment_amount: f64,
    /// Card/payment-method type string (e.g. "Visa"), if on file.
    #[serde(default)]
    pub automatic_payment_card_type: Option<String>,
    /// Masked card or account identifier, if on file.
    #[serde(default)]
    pub automatic_payment_identifier: Option<String>,
    /// Total bandwidth used this month (bytes).
    #[serde(default)]
    pub monthly_bandwidth_used: i64,
}

// ---------------------------------------------------------------------------
// Request bodies
// ---------------------------------------------------------------------------

/// Request body for `PUT /dnszone/{zoneId}/records` — add a DNS record.
/// Note: bunny.net uses PUT for record creation, not POST.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddDnsRecord {
    #[serde(rename = "Type")]
    pub record_type: DnsRecordType,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl AddDnsRecord {
    pub fn new(record_type: DnsRecordType, value: impl Into<String>) -> Self {
        Self {
            record_type,
            value: value.into(),
            name: None,
            ttl: None,
            priority: None,
            weight: None,
            port: None,
            flags: None,
            tag: None,
            comment: None,
        }
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn ttl(mut self, ttl: i32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = Some(priority);
        self
    }

    #[must_use]
    pub fn weight(mut self, weight: i32) -> Self {
        self.weight = Some(weight);
        self
    }

    #[must_use]
    pub fn port(mut self, port: i32) -> Self {
        self.port = Some(port);
        self
    }

    #[must_use]
    pub fn flags(mut self, flags: u8) -> Self {
        self.flags = Some(flags);
        self
    }

    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Request body for `POST /dnszone/{zoneId}/records/{id}` — update a DNS record.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateDnsRecord {
    pub id: i64,
    #[serde(rename = "Type")]
    pub record_type: DnsRecordType,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl UpdateDnsRecord {
    pub fn new(id: i64, record_type: DnsRecordType, value: impl Into<String>) -> Self {
        Self {
            id,
            record_type,
            value: value.into(),
            name: None,
            ttl: None,
            priority: None,
            weight: None,
            comment: None,
        }
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn ttl(mut self, ttl: i32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = Some(priority);
        self
    }

    #[must_use]
    pub fn weight(mut self, weight: i32) -> Self {
        self.weight = Some(weight);
        self
    }

    #[must_use]
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}
