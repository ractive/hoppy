use std::collections::HashMap;

use super::serde_helpers::{deserialize_repr_option, deserialize_string_lossy_option};
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
///
/// Note: bunny.net adds new origin types over time (e.g. `MagicContainerEndpoint = 5`
/// for auto-managed Pull Zones backing Magic Container CDN endpoints). The CLI
/// uses [`super::serde_helpers::deserialize_repr_option`] to map any unrecognised
/// future value to `None` instead of panicking — see `decision-log.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OriginType {
    OriginUrl = 0,
    StorageZone = 2,
    LoadBalancer = 3,
    Script = 4,
    MagicContainerEndpoint = 5,
}

impl std::fmt::Display for OriginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OriginType::OriginUrl => write!(f, "OriginUrl"),
            OriginType::StorageZone => write!(f, "StorageZone"),
            OriginType::LoadBalancer => write!(f, "LoadBalancer"),
            OriginType::Script => write!(f, "Script"),
            OriginType::MagicContainerEndpoint => write!(f, "MagicContainerEndpoint"),
        }
    }
}

/// The action to perform when an edge rule's triggers match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum EdgeRuleActionType {
    ForceSSL = 0,
    Redirect = 1,
    OriginUrl = 2,
    OverrideCacheTime = 3,
    BlockRequest = 4,
    SetResponseHeader = 5,
    SetRequestHeader = 6,
    ForceDownload = 7,
    DisableTokenAuthentication = 8,
    EnableTokenAuthentication = 9,
    OverrideCacheTimePublic = 10,
    IgnoreQueryString = 11,
    DisableOptimizer = 12,
    ForceCompression = 13,
    SetStatusCode = 14,
    BypassPermaCache = 15,
    OverrideBrowserCacheTime = 16,
    OriginStorage = 17,
    SetNetworkRateLimit = 18,
    SetConnectionLimit = 19,
    SetRequestsPerSecondLimit = 20,
    RunEdgeScript = 21,
    OriginMagicContainers = 22,
    DisableWAF = 23,
    RetryOrigin = 24,
    OverrideBrowserCacheResponseHeader = 25,
    RemoveBrowserCacheResponseHeader = 26,
    DisableShieldChallenge = 27,
    DisableShield = 28,
    DisableShieldBotDetection = 29,
    BypassAwsS3Authentication = 30,
    DisableShieldAccessLists = 31,
    DisableShieldRateLimiting = 32,
    EnableRequestCoalescing = 33,
    DisableRequestCoalescing = 34,
}

impl std::fmt::Display for EdgeRuleActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ForceSSL => "force-ssl",
            Self::Redirect => "redirect",
            Self::OriginUrl => "origin-url",
            Self::OverrideCacheTime => "override-cache-time",
            Self::BlockRequest => "block-request",
            Self::SetResponseHeader => "set-response-header",
            Self::SetRequestHeader => "set-request-header",
            Self::ForceDownload => "force-download",
            Self::DisableTokenAuthentication => "disable-token-auth",
            Self::EnableTokenAuthentication => "enable-token-auth",
            Self::OverrideCacheTimePublic => "override-cache-time-public",
            Self::IgnoreQueryString => "ignore-query-string",
            Self::DisableOptimizer => "disable-optimizer",
            Self::ForceCompression => "force-compression",
            Self::SetStatusCode => "set-status-code",
            Self::BypassPermaCache => "bypass-perma-cache",
            Self::OverrideBrowserCacheTime => "override-browser-cache-time",
            Self::OriginStorage => "origin-storage",
            Self::SetNetworkRateLimit => "set-network-rate-limit",
            Self::SetConnectionLimit => "set-connection-limit",
            Self::SetRequestsPerSecondLimit => "set-requests-per-second-limit",
            Self::RunEdgeScript => "run-edge-script",
            Self::OriginMagicContainers => "origin-magic-containers",
            Self::DisableWAF => "disable-waf",
            Self::RetryOrigin => "retry-origin",
            Self::OverrideBrowserCacheResponseHeader => "override-browser-cache-response-header",
            Self::RemoveBrowserCacheResponseHeader => "remove-browser-cache-response-header",
            Self::DisableShieldChallenge => "disable-shield-challenge",
            Self::DisableShield => "disable-shield",
            Self::DisableShieldBotDetection => "disable-shield-bot-detection",
            Self::BypassAwsS3Authentication => "bypass-aws-s3-auth",
            Self::DisableShieldAccessLists => "disable-shield-access-lists",
            Self::DisableShieldRateLimiting => "disable-shield-rate-limiting",
            Self::EnableRequestCoalescing => "enable-request-coalescing",
            Self::DisableRequestCoalescing => "disable-request-coalescing",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for EdgeRuleActionType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "force-ssl" => Ok(Self::ForceSSL),
            "redirect" => Ok(Self::Redirect),
            "origin-url" => Ok(Self::OriginUrl),
            "override-cache-time" => Ok(Self::OverrideCacheTime),
            "block-request" => Ok(Self::BlockRequest),
            "set-response-header" => Ok(Self::SetResponseHeader),
            "set-request-header" => Ok(Self::SetRequestHeader),
            "force-download" => Ok(Self::ForceDownload),
            "disable-token-auth" => Ok(Self::DisableTokenAuthentication),
            "enable-token-auth" => Ok(Self::EnableTokenAuthentication),
            "override-cache-time-public" => Ok(Self::OverrideCacheTimePublic),
            "ignore-query-string" => Ok(Self::IgnoreQueryString),
            "disable-optimizer" => Ok(Self::DisableOptimizer),
            "force-compression" => Ok(Self::ForceCompression),
            "set-status-code" => Ok(Self::SetStatusCode),
            "bypass-perma-cache" => Ok(Self::BypassPermaCache),
            "override-browser-cache-time" => Ok(Self::OverrideBrowserCacheTime),
            "origin-storage" => Ok(Self::OriginStorage),
            "set-network-rate-limit" => Ok(Self::SetNetworkRateLimit),
            "set-connection-limit" => Ok(Self::SetConnectionLimit),
            "set-requests-per-second-limit" => Ok(Self::SetRequestsPerSecondLimit),
            "run-edge-script" => Ok(Self::RunEdgeScript),
            "origin-magic-containers" => Ok(Self::OriginMagicContainers),
            "disable-waf" => Ok(Self::DisableWAF),
            "retry-origin" => Ok(Self::RetryOrigin),
            "override-browser-cache-response-header" => {
                Ok(Self::OverrideBrowserCacheResponseHeader)
            }
            "remove-browser-cache-response-header" => Ok(Self::RemoveBrowserCacheResponseHeader),
            "disable-shield-challenge" => Ok(Self::DisableShieldChallenge),
            "disable-shield" => Ok(Self::DisableShield),
            "disable-shield-bot-detection" => Ok(Self::DisableShieldBotDetection),
            "bypass-aws-s3-auth" => Ok(Self::BypassAwsS3Authentication),
            "disable-shield-access-lists" => Ok(Self::DisableShieldAccessLists),
            "disable-shield-rate-limiting" => Ok(Self::DisableShieldRateLimiting),
            "enable-request-coalescing" => Ok(Self::EnableRequestCoalescing),
            "disable-request-coalescing" => Ok(Self::DisableRequestCoalescing),
            _ => Err(format!("unknown edge rule action type: {s}")),
        }
    }
}

/// The type of condition that triggers an edge rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum TriggerType {
    Url = 0,
    RequestHeader = 1,
    ResponseHeader = 2,
    UrlExtension = 3,
    CountryCode = 4,
    RemoteIP = 5,
    UrlQueryString = 6,
    RandomChance = 7,
    StatusCode = 8,
    RequestMethod = 9,
    CookieValue = 10,
    CountryStateCode = 11,
    OriginRetryAttemptCount = 12,
    OriginConnectionError = 13,
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Url => "url",
            Self::RequestHeader => "request-header",
            Self::ResponseHeader => "response-header",
            Self::UrlExtension => "url-extension",
            Self::CountryCode => "country-code",
            Self::RemoteIP => "remote-ip",
            Self::UrlQueryString => "url-query-string",
            Self::RandomChance => "random-chance",
            Self::StatusCode => "status-code",
            Self::RequestMethod => "request-method",
            Self::CookieValue => "cookie-value",
            Self::CountryStateCode => "country-state-code",
            Self::OriginRetryAttemptCount => "origin-retry-attempt-count",
            Self::OriginConnectionError => "origin-connection-error",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for TriggerType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "url" => Ok(Self::Url),
            "request-header" => Ok(Self::RequestHeader),
            "response-header" => Ok(Self::ResponseHeader),
            "url-extension" => Ok(Self::UrlExtension),
            "country-code" => Ok(Self::CountryCode),
            "remote-ip" => Ok(Self::RemoteIP),
            "url-query-string" => Ok(Self::UrlQueryString),
            "random-chance" => Ok(Self::RandomChance),
            "status-code" => Ok(Self::StatusCode),
            "request-method" => Ok(Self::RequestMethod),
            "cookie-value" => Ok(Self::CookieValue),
            "country-state-code" => Ok(Self::CountryStateCode),
            "origin-retry-attempt-count" => Ok(Self::OriginRetryAttemptCount),
            "origin-connection-error" => Ok(Self::OriginConnectionError),
            _ => Err(format!("unknown trigger type: {s}")),
        }
    }
}

/// How multiple triggers or patterns are combined (AND / OR / NOT).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MatchingType {
    MatchAny = 0,
    MatchAll = 1,
    MatchNone = 2,
}

impl std::fmt::Display for MatchingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MatchAny => write!(f, "match-any"),
            Self::MatchAll => write!(f, "match-all"),
            Self::MatchNone => write!(f, "match-none"),
        }
    }
}

impl std::str::FromStr for MatchingType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "match-any" => Ok(Self::MatchAny),
            "match-all" => Ok(Self::MatchAll),
            "match-none" => Ok(Self::MatchNone),
            _ => Err(format!("unknown matching type: {s}")),
        }
    }
}

/// Where a watermark image is placed on optimised images.
///
/// Bunny.net may add new positions in future API versions. The
/// `optimizer_watermark_position` field on [`PullZone`] is deserialised via
/// [`super::serde_helpers::deserialize_repr_option`] so unknown values become
/// `None` instead of failing the whole response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum OptimizerWatermarkPosition {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
    Center = 4,
}

impl std::fmt::Display for OptimizerWatermarkPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::TopLeft => "top-left",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomRight => "bottom-right",
            Self::Center => "center",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for OptimizerWatermarkPosition {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "top-left" => Ok(Self::TopLeft),
            "top-right" => Ok(Self::TopRight),
            "bottom-left" => Ok(Self::BottomLeft),
            "bottom-right" => Ok(Self::BottomRight),
            "center" => Ok(Self::Center),
            _ => Err(format!(
                "unknown watermark position: {s:?}; \
                 expected one of: top-left, top-right, bottom-left, bottom-right, center"
            )),
        }
    }
}

/// Transport protocol used by CDN log forwarding to a remote syslog endpoint.
///
/// Bunny.net may add new protocols in future API versions. The
/// `log_forwarding_protocol` field on [`PullZone`] is deserialised via
/// [`super::serde_helpers::deserialize_repr_option`] so unknown values become
/// `None` instead of failing the whole response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PullZoneLogForwarderProtocolType {
    Udp = 0,
    Tcp = 1,
    TcpEncrypted = 2,
    DataDog = 3,
}

impl std::fmt::Display for PullZoneLogForwarderProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Udp => "udp",
            Self::Tcp => "tcp",
            Self::TcpEncrypted => "tcp-encrypted",
            Self::DataDog => "datadog",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for PullZoneLogForwarderProtocolType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "udp" | "UDP" => Ok(Self::Udp),
            "tcp" | "TCP" => Ok(Self::Tcp),
            "tcp-encrypted" | "TCPEncrypted" => Ok(Self::TcpEncrypted),
            "datadog" | "DataDog" => Ok(Self::DataDog),
            _ => Err(format!(
                "unknown log forwarding protocol: {s:?}; \
                 expected one of: udp, tcp, tcp-encrypted, datadog \
                 (or PascalCase: UDP, TCP, TCPEncrypted, DataDog)"
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Response models
// ---------------------------------------------------------------------------

/// A single trigger condition within an edge rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeRuleTrigger {
    #[serde(rename = "Type", default, deserialize_with = "deserialize_repr_option")]
    pub trigger_type: Option<TriggerType>,
    #[serde(default)]
    pub pattern_matches: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub pattern_matching_type: Option<MatchingType>,
    #[serde(default)]
    pub parameter1: Option<String>,
}

/// An additional action on an edge rule (beyond the primary action).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeRuleExtraAction {
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub action_type: Option<EdgeRuleActionType>,
    #[serde(default)]
    pub action_parameter1: Option<String>,
    #[serde(default)]
    pub action_parameter2: Option<String>,
    #[serde(default)]
    pub action_parameter3: Option<String>,
}

/// An edge rule attached to a pull zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeRule {
    #[serde(default)]
    pub guid: Option<String>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub action_type: Option<EdgeRuleActionType>,
    #[serde(default)]
    pub action_parameter1: Option<String>,
    #[serde(default)]
    pub action_parameter2: Option<String>,
    #[serde(default)]
    pub action_parameter3: Option<String>,
    #[serde(default)]
    pub triggers: Vec<EdgeRuleTrigger>,
    #[serde(default)]
    pub extra_actions: Vec<EdgeRuleExtraAction>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub trigger_matching_type: Option<MatchingType>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub order_index: i32,
    #[serde(default)]
    pub read_only: bool,
}

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
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub origin_type: Option<OriginType>,
    #[serde(default)]
    pub storage_zone_id: Option<i64>,

    // Network / routing
    #[serde(default)]
    pub cname_domain: String,
    #[serde(default)]
    pub hostnames: Vec<HostnameInfo>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
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

    // Optimizer — master switches
    #[serde(default)]
    pub optimizer_enabled: Option<bool>,
    #[serde(default)]
    pub optimizer_automatic_optimization_enabled: Option<bool>,

    // Optimizer — image dimensions & quality
    #[serde(default)]
    pub optimizer_desktop_max_width: Option<i32>,
    #[serde(default)]
    pub optimizer_mobile_max_width: Option<i32>,
    #[serde(default)]
    pub optimizer_image_quality: Option<i32>,
    #[serde(default)]
    pub optimizer_mobile_image_quality: Option<i32>,

    // Optimizer — format & upscale
    #[serde(default)]
    pub optimizer_enable_web_p: Option<bool>,
    #[serde(default)]
    pub optimizer_enable_upscaling: Option<bool>,

    // Optimizer — minify
    #[serde(rename = "OptimizerMinifyCSS", default)]
    pub optimizer_minify_css: Option<bool>,
    #[serde(rename = "OptimizerMinifyJavaScript", default)]
    pub optimizer_minify_java_script: Option<bool>,

    // Optimizer — manipulation engine
    #[serde(default)]
    pub optimizer_enable_manipulation_engine: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_string_lossy_option")]
    pub optimizer_classes: Option<String>,
    #[serde(default)]
    pub optimizer_force_classes: Option<bool>,

    // Optimizer — watermark
    #[serde(default)]
    pub optimizer_watermark_enabled: Option<bool>,
    #[serde(default)]
    pub optimizer_watermark_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub optimizer_watermark_position: Option<OptimizerWatermarkPosition>,
    #[serde(default)]
    pub optimizer_watermark_offset: Option<f64>,
    #[serde(default)]
    pub optimizer_watermark_min_image_size: Option<i32>,

    // Optimizer — static HTML / WordPress
    #[serde(default)]
    pub optimizer_static_html_enabled: Option<bool>,
    #[serde(default)]
    pub optimizer_static_html_word_press_path: Option<String>,
    #[serde(default)]
    pub optimizer_static_html_word_press_bypass_cookie: Option<String>,

    // Optimizer — prerender & tunnel
    #[serde(default)]
    pub optimizer_prerender_html: Option<bool>,
    #[serde(default)]
    pub optimizer_tunnel_enabled: Option<bool>,

    // Optimizer — read-only pricing tier (server-set; not writable)
    /// Server-set float indicating the Optimizer pricing tier. Not writable —
    /// omit from `UpdatePullZone`. The wire format is a float (e.g. 9.5),
    /// despite the API docs suggesting an integer.
    #[serde(default)]
    pub optimizer_pricing: Option<f64>,

    // Log forwarding
    #[serde(default)]
    pub log_forwarding_enabled: Option<bool>,
    #[serde(default)]
    pub log_forwarding_hostname: Option<String>,
    #[serde(default)]
    pub log_forwarding_port: Option<i32>,
    #[serde(default)]
    pub log_forwarding_token: Option<String>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub log_forwarding_protocol: Option<PullZoneLogForwarderProtocolType>,
    #[serde(default)]
    pub logging_save_to_storage: Option<bool>,
    #[serde(default)]
    pub logging_storage_zone_id: Option<i64>,

    // Edge rules
    #[serde(default)]
    pub edge_rules: Vec<EdgeRule>,

    // Access control
    #[serde(default)]
    pub allowed_referrers: Vec<String>,
    #[serde(default)]
    pub blocked_referrers: Vec<String>,
    #[serde(default)]
    pub blocked_ips: Vec<String>,
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
/// Only `name` is structurally required by this type; the bunny API requires
/// either an `origin_url` or a `storage_zone_id` (CLI enforces "exactly one"
/// at parse time via clap `ArgGroup`). All other fields default to the API's
/// own defaults when absent.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreatePullZone {
    pub name: String,

    /// Origin URL for HTTP/HTTPS-backed Pull Zones. Omit when `storage_zone_id`
    /// is set — bunny then uses the Storage Zone as the origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_url: Option<String>,

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
    /// Create a Pull Zone backed by an HTTP/HTTPS origin URL.
    pub fn new(name: impl Into<String>, origin_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            origin_url: Some(origin_url.into()),
            origin_type: None,
            storage_zone_id: None,
            zone_type: None,
            monthly_bandwidth_limit: None,
            zone_security_enabled: None,
        }
    }

    /// Create a Pull Zone backed by an existing Storage Zone (no origin URL).
    pub fn for_storage_zone(name: impl Into<String>, storage_zone_id: i64) -> Self {
        Self {
            name: name.into(),
            origin_url: None,
            origin_type: None,
            storage_zone_id: Some(storage_zone_id),
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

    // Log forwarding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_forwarding_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_forwarding_hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_forwarding_port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_forwarding_token: Option<String>,
    /// Serialises as an integer via `Serialize_repr`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_forwarding_protocol: Option<PullZoneLogForwarderProtocolType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging_save_to_storage: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging_storage_zone_id: Option<i64>,

    // Optimizer — master switches
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_automatic_optimization_enabled: Option<bool>,

    // Optimizer — image dimensions & quality
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_desktop_max_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_mobile_max_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_image_quality: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_mobile_image_quality: Option<i32>,

    // Optimizer — format & upscale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_enable_web_p: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_enable_upscaling: Option<bool>,

    // Optimizer — minify
    #[serde(rename = "OptimizerMinifyCSS", skip_serializing_if = "Option::is_none")]
    pub optimizer_minify_css: Option<bool>,
    #[serde(
        rename = "OptimizerMinifyJavaScript",
        skip_serializing_if = "Option::is_none"
    )]
    pub optimizer_minify_java_script: Option<bool>,

    // Optimizer — manipulation engine
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_enable_manipulation_engine: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_classes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_force_classes: Option<bool>,

    // Optimizer — watermark
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_watermark_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_watermark_url: Option<String>,
    /// Serialises as an integer via `Serialize_repr`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_watermark_position: Option<OptimizerWatermarkPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_watermark_offset: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_watermark_min_image_size: Option<i32>,

    // Optimizer — static HTML / WordPress
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_static_html_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_static_html_word_press_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_static_html_word_press_bypass_cookie: Option<String>,

    // Optimizer — prerender & tunnel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_prerender_html: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimizer_tunnel_enabled: Option<bool>,
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

    #[must_use]
    pub fn optimizer_enabled(mut self, v: bool) -> Self {
        self.optimizer_enabled = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_automatic_optimization_enabled(mut self, v: bool) -> Self {
        self.optimizer_automatic_optimization_enabled = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_desktop_max_width(mut self, v: i32) -> Self {
        self.optimizer_desktop_max_width = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_mobile_max_width(mut self, v: i32) -> Self {
        self.optimizer_mobile_max_width = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_image_quality(mut self, v: i32) -> Self {
        self.optimizer_image_quality = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_mobile_image_quality(mut self, v: i32) -> Self {
        self.optimizer_mobile_image_quality = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_enable_web_p(mut self, v: bool) -> Self {
        self.optimizer_enable_web_p = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_enable_upscaling(mut self, v: bool) -> Self {
        self.optimizer_enable_upscaling = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_minify_css(mut self, v: bool) -> Self {
        self.optimizer_minify_css = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_minify_java_script(mut self, v: bool) -> Self {
        self.optimizer_minify_java_script = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_enable_manipulation_engine(mut self, v: bool) -> Self {
        self.optimizer_enable_manipulation_engine = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_classes(mut self, v: impl Into<String>) -> Self {
        self.optimizer_classes = Some(v.into());
        self
    }

    #[must_use]
    pub fn optimizer_force_classes(mut self, v: bool) -> Self {
        self.optimizer_force_classes = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_watermark_enabled(mut self, v: bool) -> Self {
        self.optimizer_watermark_enabled = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_watermark_url(mut self, v: impl Into<String>) -> Self {
        self.optimizer_watermark_url = Some(v.into());
        self
    }

    #[must_use]
    pub fn optimizer_watermark_position(mut self, v: OptimizerWatermarkPosition) -> Self {
        self.optimizer_watermark_position = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_watermark_offset(mut self, v: f64) -> Self {
        self.optimizer_watermark_offset = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_watermark_min_image_size(mut self, v: i32) -> Self {
        self.optimizer_watermark_min_image_size = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_static_html_enabled(mut self, v: bool) -> Self {
        self.optimizer_static_html_enabled = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_static_html_word_press_path(mut self, v: impl Into<String>) -> Self {
        self.optimizer_static_html_word_press_path = Some(v.into());
        self
    }

    #[must_use]
    pub fn optimizer_static_html_word_press_bypass_cookie(mut self, v: impl Into<String>) -> Self {
        self.optimizer_static_html_word_press_bypass_cookie = Some(v.into());
        self
    }

    #[must_use]
    pub fn optimizer_prerender_html(mut self, v: bool) -> Self {
        self.optimizer_prerender_html = Some(v);
        self
    }

    #[must_use]
    pub fn optimizer_tunnel_enabled(mut self, v: bool) -> Self {
        self.optimizer_tunnel_enabled = Some(v);
        self
    }

    #[must_use]
    pub fn log_forwarding_enabled(mut self, v: bool) -> Self {
        self.log_forwarding_enabled = Some(v);
        self
    }

    #[must_use]
    pub fn log_forwarding_hostname(mut self, v: impl Into<String>) -> Self {
        self.log_forwarding_hostname = Some(v.into());
        self
    }

    #[must_use]
    pub fn log_forwarding_port(mut self, v: i32) -> Self {
        self.log_forwarding_port = Some(v);
        self
    }

    #[must_use]
    pub fn log_forwarding_token(mut self, v: impl Into<String>) -> Self {
        self.log_forwarding_token = Some(v.into());
        self
    }

    #[must_use]
    pub fn log_forwarding_protocol(mut self, v: PullZoneLogForwarderProtocolType) -> Self {
        self.log_forwarding_protocol = Some(v);
        self
    }

    #[must_use]
    pub fn logging_save_to_storage(mut self, v: bool) -> Self {
        self.logging_save_to_storage = Some(v);
        self
    }

    #[must_use]
    pub fn logging_storage_zone_id(mut self, v: i64) -> Self {
        self.logging_storage_zone_id = Some(v);
        self
    }
}

/// Request body for `POST /pullzone/{id}/edgerules/addOrUpdate`.
///
/// When `guid` is `None`, a new edge rule is created. When set to an existing
/// GUID, the rule is updated (upsert semantics).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddOrUpdateEdgeRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guid: Option<String>,
    pub action_type: EdgeRuleActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_parameter1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_parameter2: Option<String>,
    pub triggers: Vec<EdgeRuleTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_matching_type: Option<MatchingType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

impl AddOrUpdateEdgeRule {
    pub fn new(action_type: EdgeRuleActionType) -> Self {
        Self {
            guid: None,
            action_type,
            action_parameter1: None,
            action_parameter2: None,
            triggers: Vec::new(),
            trigger_matching_type: None,
            description: None,
            enabled: None,
        }
    }

    #[must_use]
    pub fn guid(mut self, guid: impl Into<String>) -> Self {
        self.guid = Some(guid.into());
        self
    }

    #[must_use]
    pub fn action_parameter1(mut self, val: impl Into<String>) -> Self {
        self.action_parameter1 = Some(val.into());
        self
    }

    #[must_use]
    pub fn action_parameter2(mut self, val: impl Into<String>) -> Self {
        self.action_parameter2 = Some(val.into());
        self
    }

    #[must_use]
    pub fn trigger(mut self, trigger: EdgeRuleTrigger) -> Self {
        self.triggers.push(trigger);
        self
    }

    #[must_use]
    pub fn trigger_matching_type(mut self, t: MatchingType) -> Self {
        self.trigger_matching_type = Some(t);
        self
    }

    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
}

// ---------------------------------------------------------------------------
// Storage Zone types
// ---------------------------------------------------------------------------

/// A bunny.net Storage Zone.
///
/// `Password` and `ReadOnlyPassword` are returned by the API and are required
/// for authenticating against the storage endpoint. They are *serialised* but
/// the CLI redacts them by default — opt in with `--reveal` to see raw values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageZone {
    pub id: i64,
    pub user_id: String,
    pub name: String,
    #[serde(default)]
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
    #[serde(default)]
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
///
/// `record_type` is an `Option` because bunny.net may return new record-type
/// integers that pre-date this client's enum — in that case the field
/// deserialises to `None` rather than panicking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsRecord {
    pub id: i64,
    #[serde(rename = "Type", default, deserialize_with = "deserialize_repr_option")]
    pub record_type: Option<DnsRecordType>,
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

/// Result of importing DNS records into a zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsImportResult {
    pub records_successful: i64,
    pub records_failed: i64,
    pub records_skipped: i64,
}

/// DNSSEC DS record information returned when enabling/disabling DNSSEC.
///
/// The DS record fields (`ds_record`, `digest`, `digest_type`, `algorithm`,
/// `key_tag`, `flags`, `public_key`) are needed to configure DNSSEC at the
/// domain registrar. `ds_configured` indicates whether bunny.net has detected
/// matching DS records at the registrar.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsSecDsRecord {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub ds_record: Option<String>,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub digest_type: Option<String>,
    #[serde(default)]
    pub algorithm: i32,
    #[serde(default)]
    pub public_key: Option<String>,
    #[serde(default)]
    pub key_tag: i32,
    #[serde(default)]
    pub flags: i32,
    #[serde(default)]
    pub ds_configured: bool,
}

/// Status of a DNS record scan job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DnsScanJobStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
}

impl std::fmt::Display for DnsScanJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::InProgress => write!(f, "InProgress"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// Discovered DNS record type returned by the record scan endpoint.
///
/// The integer values use a different numbering scheme than [`DnsRecordType`]
/// (e.g. the scan API uses A=0 where the zone API uses A=1), so we keep a
/// dedicated enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DnsDiscoveredRecordType {
    A = 0,
    AAAA = 1,
    CNAME = 2,
    TXT = 3,
    MX = 4,
    SRV = 8,
    CAA = 9,
    PTR = 10,
    NS = 12,
    SVCB = 13,
    HTTPS = 14,
    TLSA = 15,
    SOA = 16,
}

impl std::fmt::Display for DnsDiscoveredRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::A => "A",
            Self::AAAA => "AAAA",
            Self::CNAME => "CNAME",
            Self::TXT => "TXT",
            Self::MX => "MX",
            Self::SRV => "SRV",
            Self::CAA => "CAA",
            Self::PTR => "PTR",
            Self::NS => "NS",
            Self::SVCB => "SVCB",
            Self::HTTPS => "HTTPS",
            Self::TLSA => "TLSA",
            Self::SOA => "SOA",
        };
        f.write_str(s)
    }
}

/// A DNS record discovered during a record scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsDiscoveredRecord {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "Type", default, deserialize_with = "deserialize_repr_option")]
    pub record_type: Option<DnsDiscoveredRecordType>,
    #[serde(default)]
    pub ttl: Option<i32>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub weight: Option<i32>,
    #[serde(default)]
    pub port: Option<i32>,
    #[serde(default)]
    pub is_proxied: bool,
}

/// Response from `POST /dnszone/records/scan` — a scan trigger acknowledgement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsRecordScanTrigger {
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub status: Option<DnsScanJobStatus>,
}

/// Response from `GET /dnszone/{zoneId}/records/scan` — the latest scan job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsRecordScanResult {
    #[serde(default)]
    pub job_id: Option<String>,
    #[serde(default)]
    pub zone_id: Option<i64>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_repr_option")]
    pub status: Option<DnsScanJobStatus>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default, deserialize_with = "super::serde_helpers::null_to_empty_vec")]
    pub records: Vec<DnsDiscoveredRecord>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Request body for `POST /dnszone/records/scan` — provide either `zone_id`
/// (for an existing zone) or `domain` (pre-creation), but not both.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TriggerDnsRecordScan {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl TriggerDnsRecordScan {
    #[must_use]
    pub fn for_zone(zone_id: i64) -> Self {
        Self {
            zone_id: Some(zone_id),
            domain: None,
        }
    }

    #[must_use]
    pub fn for_domain(domain: impl Into<String>) -> Self {
        Self {
            zone_id: None,
            domain: Some(domain.into()),
        }
    }
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
    // bunny.net misspells "threshold" as "Treshold" in the API JSON key.
    #[serde(rename = "AutomaticRechargeTreshold", default)]
    pub automatic_recharge_threshold: f64,
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

// ---------------------------------------------------------------------------
// Statistics types
// ---------------------------------------------------------------------------

/// Account-level statistics returned by `GET /statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccountStatistics {
    #[serde(default)]
    pub total_bandwidth_used: i64,
    #[serde(default)]
    pub total_origin_traffic: i64,
    #[serde(default)]
    pub average_origin_response_time: i64,
    #[serde(default)]
    pub total_requests_served: i64,
    #[serde(default)]
    pub cache_hit_rate: f64,
    pub origin_response_time_chart: Option<HashMap<String, i64>>,
    pub bandwidth_used_chart: Option<HashMap<String, i64>>,
    pub bandwidth_cached_chart: Option<HashMap<String, i64>>,
    pub cache_hit_rate_chart: Option<HashMap<String, f64>>,
    pub requests_served_chart: Option<HashMap<String, i64>>,
    pub pull_requests_pulled_chart: Option<HashMap<String, i64>>,
    pub origin_shield_bandwidth_used_chart: Option<HashMap<String, i64>>,
    pub origin_shield_internal_bandwidth_used_chart: Option<HashMap<String, i64>>,
    pub origin_traffic_chart: Option<HashMap<String, i64>>,
    pub user_balance_history_chart: Option<HashMap<String, f64>>,
    pub geo_traffic_distribution: Option<HashMap<String, i64>>,
    #[serde(rename = "Error3xxChart")]
    pub error3xx_chart: Option<HashMap<String, i64>>,
    #[serde(rename = "Error4xxChart")]
    pub error4xx_chart: Option<HashMap<String, i64>>,
    #[serde(rename = "Error5xxChart")]
    pub error5xx_chart: Option<HashMap<String, i64>>,
}

/// Storage Zone statistics returned by `GET /storagezone/{id}/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageZoneStatistics {
    pub storage_used_chart: Option<HashMap<String, i64>>,
    pub file_count_chart: Option<HashMap<String, i64>>,
}

/// DNS Zone statistics returned by `GET /dnszone/{id}/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DnsZoneStatistics {
    #[serde(default)]
    pub total_queries_served: i64,
    pub queries_served_chart: Option<HashMap<String, i64>>,
    pub normal_queries_served_chart: Option<HashMap<String, i64>>,
    pub smart_queries_served_chart: Option<HashMap<String, i64>>,
    pub queries_by_type_chart: Option<HashMap<String, i64>>,
}

/// Pull Zone optimizer statistics returned by
/// `GET /pullzone/{pullZoneId}/optimizer/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OptimizerStatistics {
    #[serde(default)]
    pub total_requests_optimized: f64,
    #[serde(default)]
    pub total_traffic_saved: f64,
    #[serde(default)]
    pub average_processing_time: f64,
    #[serde(default)]
    pub average_compression_ratio: f64,
    pub requests_optimized_chart: Option<HashMap<String, i64>>,
    pub average_compression_chart: Option<HashMap<String, f64>>,
    pub traffic_saved_chart: Option<HashMap<String, i64>>,
    pub average_processing_time_chart: Option<HashMap<String, f64>>,
}

/// Origin shield queue statistics returned by
/// `GET /pullzone/{pullZoneId}/originshield/queuestatistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OriginShieldQueueStatistics {
    pub concurrent_requests_chart: Option<HashMap<String, i64>>,
    pub queued_requests_chart: Option<HashMap<String, i64>>,
}

/// SafeHop statistics returned by
/// `GET /pullzone/{pullZoneId}/safehop/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SafeHopStatistics {
    #[serde(default)]
    pub total_requests_retried: f64,
    #[serde(default)]
    pub total_requests_saved: f64,
    pub requests_retried_chart: Option<HashMap<String, i64>>,
    pub requests_saved_chart: Option<HashMap<String, i64>>,
}

/// Video Library DRM statistics returned by
/// `GET /videolibrary/{id}/drm/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VideoLibraryDrmStatistics {
    #[serde(default)]
    pub total_licenses_issued: i64,
    pub licenses_issued_chart: Option<HashMap<String, i64>>,
}

/// Video Library transcribing statistics returned by
/// `GET /videolibrary/{id}/transcribing/statistics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VideoLibraryTranscribingStatistics {
    #[serde(default)]
    pub total_transcription_seconds: i64,
    pub transcription_seconds_chart: Option<HashMap<String, i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression: the Bunny API returns `"Records": null` for in-progress
    /// scans. Verify that `DnsRecordScanResult` deserialises to an empty vec
    /// rather than panicking.
    #[test]
    fn dns_record_scan_result_null_records_becomes_empty_vec() {
        let json = r#"{
            "JobId": "abc123",
            "ZoneId": 42,
            "Domain": "example.com",
            "Status": 1,
            "Records": null
        }"#;
        let result: DnsRecordScanResult = serde_json::from_str(json).unwrap();
        assert!(result.records.is_empty());
        assert_eq!(result.domain.as_deref(), Some("example.com"));
    }

    /// Confirm that a populated `Records` array still deserialises correctly.
    #[test]
    fn dns_record_scan_result_with_records() {
        let json = r#"{
            "JobId": "abc123",
            "ZoneId": 42,
            "Domain": "example.com",
            "Status": 2,
            "Records": [
                {"Value": "1.2.3.4", "Ttl": 300, "Type": 0}
            ]
        }"#;
        let result: DnsRecordScanResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.records.len(), 1);
    }

    /// Confirm that an absent `Records` field also yields an empty vec.
    #[test]
    fn dns_record_scan_result_missing_records_becomes_empty_vec() {
        let json = r#"{"JobId": "abc123"}"#;
        let result: DnsRecordScanResult = serde_json::from_str(json).unwrap();
        assert!(result.records.is_empty());
    }
}
