use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// WAF execution mode for a Shield Zone.
///
/// Values from the spec: 0 = Disabled, 1 = Enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WafExecutionMode {
    Disabled = 0,
    Enabled = 1,
}

impl std::fmt::Display for WafExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WafExecutionMode::Disabled => write!(f, "Disabled"),
            WafExecutionMode::Enabled => write!(f, "Enabled"),
        }
    }
}

/// Action taken when WAF payload size limit is exceeded.
///
/// Values from the spec: 0 = Allow, 1 = Block, 2 = Ignore.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WafPayloadLimitAction {
    Allow = 0,
    Block = 1,
    Ignore = 2,
}

impl std::fmt::Display for WafPayloadLimitAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WafPayloadLimitAction::Allow => write!(f, "Allow"),
            WafPayloadLimitAction::Block => write!(f, "Block"),
            WafPayloadLimitAction::Ignore => write!(f, "Ignore"),
        }
    }
}

/// Shield plan tier.
///
/// Values from the spec: 0 = Basic, 1 = Standard, 2 = Advanced, 3 = Enterprise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ShieldPlanType {
    Basic = 0,
    Standard = 1,
    Advanced = 2,
    Enterprise = 3,
}

impl std::fmt::Display for ShieldPlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShieldPlanType::Basic => write!(f, "Basic"),
            ShieldPlanType::Standard => write!(f, "Standard"),
            ShieldPlanType::Advanced => write!(f, "Advanced"),
            ShieldPlanType::Enterprise => write!(f, "Enterprise"),
        }
    }
}

/// DDoS protection execution mode.
///
/// Values from the spec: 0 = Disabled, 1 = Enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DdosExecutionMode {
    Disabled = 0,
    Enabled = 1,
}

/// DDoS shield detection sensitivity level.
///
/// Values from the spec: 0 = Disabled, 1 = Low, 2 = Medium, 3 = High, 4 = VeryHigh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DdosShieldSensitivity {
    Disabled = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    VeryHigh = 4,
}

/// Action to take when an access list entry matches a request.
///
/// Values from the spec: 0 = NoAction, 1 = Block, 2 = Allow, 3 = LogOnly,
/// 4 = Challenge, 5 = ChallengeInterstitial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum AccessListAction {
    NoAction = 0,
    Block = 1,
    Allow = 2,
    LogOnly = 3,
    Challenge = 4,
    ChallengeInterstitial = 5,
}

impl std::fmt::Display for AccessListAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessListAction::NoAction => write!(f, "NoAction"),
            AccessListAction::Block => write!(f, "Block"),
            AccessListAction::Allow => write!(f, "Allow"),
            AccessListAction::LogOnly => write!(f, "LogOnly"),
            AccessListAction::Challenge => write!(f, "Challenge"),
            AccessListAction::ChallengeInterstitial => write!(f, "ChallengeInterstitial"),
        }
    }
}

/// Type of access list (IP, Country, ASN, etc.).
///
/// Values from the spec: 0 = Ip, 1 = Country, 2 = Asn, 3 = Hostname, 4 = UserAgent, 5 = Custom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum AccessListType {
    Ip = 0,
    Country = 1,
    Asn = 2,
    Hostname = 3,
    UserAgent = 4,
    Custom = 5,
}

impl std::fmt::Display for AccessListType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessListType::Ip => write!(f, "Ip"),
            AccessListType::Country => write!(f, "Country"),
            AccessListType::Asn => write!(f, "Asn"),
            AccessListType::Hostname => write!(f, "Hostname"),
            AccessListType::UserAgent => write!(f, "UserAgent"),
            AccessListType::Custom => write!(f, "Custom"),
        }
    }
}

/// Action taken when a rate limit rule is breached.
///
/// Values from the spec: 1 = Block, 2 = LogOnly, 3 = Challenge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RateLimitActionType {
    Block = 1,
    LogOnly = 2,
    Challenge = 3,
}

impl std::fmt::Display for RateLimitActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitActionType::Block => write!(f, "Block"),
            RateLimitActionType::LogOnly => write!(f, "LogOnly"),
            RateLimitActionType::Challenge => write!(f, "Challenge"),
        }
    }
}

/// Time window (in seconds) for rate limit counting.
///
/// Values from the spec: 1, 10, 60, 300, 900, 3600.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum RateLimitTimeframe {
    Sec1 = 1,
    Sec10 = 10,
    Sec60 = 60,
    Sec300 = 300,
    Sec900 = 900,
    Sec3600 = 3600,
}

/// Duration (in seconds) to block after a rate limit breach.
///
/// Values from the spec: 30, 60, 300, 900, 1800, 3600.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u16)]
pub enum RateLimitBlockDuration {
    Sec30 = 30,
    Sec60 = 60,
    Sec300 = 300,
    Sec900 = 900,
    Sec1800 = 1800,
    Sec3600 = 3600,
}

/// What key to count requests by for rate limiting.
///
/// Values from the spec: 0 = Global, 1 = PerIp, 2 = PerCountry, 3 = PerAsn,
/// 4 = PerHeader, 5 = PerCookie, 6 = PerQuery, 7 = PerFingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RateLimitCounterKey {
    Global = 0,
    PerIp = 1,
    PerCountry = 2,
    PerAsn = 3,
    PerHeader = 4,
    PerCookie = 5,
    PerQuery = 6,
    PerFingerprint = 7,
}

/// Action taken by a custom WAF rule.
///
/// Values from the spec: 1 = Block, 2 = LogOnly, 3 = Challenge,
/// 4 = ChallengeInterstitial, 5 = Allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WafRuleActionType {
    Block = 1,
    LogOnly = 2,
    Challenge = 3,
    ChallengeInterstitial = 4,
    Allow = 5,
}

impl std::fmt::Display for WafRuleActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WafRuleActionType::Block => write!(f, "Block"),
            WafRuleActionType::LogOnly => write!(f, "LogOnly"),
            WafRuleActionType::Challenge => write!(f, "Challenge"),
            WafRuleActionType::ChallengeInterstitial => write!(f, "ChallengeInterstitial"),
            WafRuleActionType::Allow => write!(f, "Allow"),
        }
    }
}

/// Operator used to match a WAF rule variable against a value.
///
/// Values from the spec: 0 = Eq, 1 = NotEq, 2 = Contains, 3 = NotContains,
/// 4 = Begins, 5 = Ends, 6 = Regex, 7 = NotRegex, 8 = Lt, 9 = Gt,
/// 12 = Pm, 14 = PmFromFile, 15 = IpMatch, 17 = GeoLookup, 18 = ValidateUrlEncoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WafRuleOperatorType {
    Eq = 0,
    NotEq = 1,
    Contains = 2,
    NotContains = 3,
    Begins = 4,
    Ends = 5,
    Regex = 6,
    NotRegex = 7,
    Lt = 8,
    Gt = 9,
    Pm = 12,
    PmFromFile = 14,
    IpMatch = 15,
    GeoLookup = 17,
    ValidateUrlEncoding = 18,
}

/// Severity level of a custom WAF rule.
///
/// Values from the spec: 0 = Low, 1 = Medium, 2 = High.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WafRuleSeverityType {
    Low = 0,
    Medium = 1,
    High = 2,
}

/// Transformation applied to the variable value before matching.
///
/// Values from the spec: 1–21.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WafRuleTransformationType {
    Lowercase = 1,
    Uppercase = 2,
    UrlDecode = 3,
    UrlDecodeUni = 4,
    HtmlEntityDecode = 5,
    Base64Decode = 6,
    Base64DecodeExt = 7,
    Base64Encode = 8,
    JsDecode = 9,
    CssDecode = 10,
    Trim = 11,
    TrimLeft = 12,
    TrimRight = 13,
    NormalisePath = 14,
    NormalisePathWin = 15,
    RemoveComments = 16,
    RemoveNulls = 17,
    RemoveWhitespace = 18,
    ReplaceComments = 19,
    CompressWhitespace = 20,
    None = 21,
}

/// Bot detection execution mode.
///
/// Values from the spec: 0 = Disabled, 1 = Enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BotDetectionExecutionMode {
    Disabled = 0,
    Enabled = 1,
}

/// Sensitivity level for bot detection signals.
///
/// Values from the spec: 0 = Disabled, 1 = Low, 2 = Medium, 3 = High.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BotDetectionSensitivity {
    Disabled = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Aggressiveness of browser fingerprint challenge injection.
///
/// Values from the spec: 0 = Disabled, 1 = VeryLow, 2 = Low, 3 = Medium, 4 = High.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BrowserFingerprintAggression {
    Disabled = 0,
    VeryLow = 1,
    Low = 2,
    Medium = 3,
    High = 4,
}

// ---------------------------------------------------------------------------
// Error / problem types
// ---------------------------------------------------------------------------

/// RFC 7807 problem details returned on 4xx/5xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type", default)]
    pub problem_type: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub instance: Option<String>,
}

impl std::fmt::Display for ProblemDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Shield API error {}: {}",
            self.status.unwrap_or(0),
            self.title.as_deref().unwrap_or("unknown"),
        )?;
        if let Some(detail) = &self.detail {
            write!(f, " — {detail}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ProblemDetails {}

/// Generic API operation result embedded in many Shield responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericRequestResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error_key: Option<String>,
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

/// Pagination metadata returned alongside list responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationResponse {
    pub total_count: i32,
    pub total_pages: i32,
    pub current_page: i32,
    #[serde(default)]
    pub next_page: Option<i32>,
    pub page_size: i32,
}

// ---------------------------------------------------------------------------
// Shield Zone
// ---------------------------------------------------------------------------

/// The full Shield Zone configuration object returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldZoneResponse {
    pub shield_zone_id: i64,
    #[serde(default)]
    pub pull_zone_id: Option<i64>,
    #[serde(default)]
    pub learning_mode: Option<bool>,
    #[serde(default)]
    pub learning_mode_until: Option<String>,
    #[serde(default)]
    pub waf_enabled: Option<bool>,
    #[serde(default)]
    pub waf_execution_mode: Option<WafExecutionMode>,
    #[serde(default)]
    pub waf_disabled_rules: Option<Vec<String>>,
    #[serde(default)]
    pub waf_log_only_rules: Option<Vec<String>>,
    #[serde(default)]
    pub waf_request_header_logging_enabled: Option<bool>,
    #[serde(default)]
    pub waf_request_ignored_headers: Option<Vec<String>>,
    #[serde(default)]
    pub waf_realtime_threat_intelligence_enabled: Option<bool>,
    #[serde(default)]
    pub waf_profile_id: Option<i32>,
    #[serde(default)]
    pub waf_request_body_limit_action: Option<WafPayloadLimitAction>,
    #[serde(default)]
    pub waf_response_body_limit_action: Option<WafPayloadLimitAction>,
    #[serde(default)]
    pub rate_limit_rules_limit: i32,
    #[serde(default)]
    pub custom_waf_rules_limit: i32,
    #[serde(default)]
    pub plan_type: Option<ShieldPlanType>,
    #[serde(default)]
    pub d_do_s_shield_sensitivity: Option<DdosShieldSensitivity>,
    #[serde(default)]
    pub d_do_s_execution_mode: Option<DdosExecutionMode>,
    #[serde(default)]
    pub d_do_s_challenge_window: Option<i32>,
    #[serde(default)]
    pub block_vpn: Option<bool>,
    #[serde(default)]
    pub block_tor: Option<bool>,
    #[serde(default)]
    pub block_datacentre: Option<bool>,
    #[serde(default)]
    pub whitelabel_response_pages: Option<bool>,
}

/// Request body for creating a new Shield Zone.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateShieldZoneRequest {
    pub pull_zone_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield_zone: Option<ShieldZoneRequest>,
}

/// Request body for updating an existing Shield Zone.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShieldZoneRequest {
    pub shield_zone_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield_zone: Option<ShieldZoneRequest>,
}

/// Nested Shield Zone configuration fields shared between create/update.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldZoneRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<ShieldPlanType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_mode_until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_execution_mode: Option<WafExecutionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_disabled_rules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_log_only_rules: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_request_header_logging_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_profile_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_realtime_threat_intelligence_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_request_body_limit_action: Option<WafPayloadLimitAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waf_response_body_limit_action: Option<WafPayloadLimitAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d_do_s_shield_sensitivity: Option<DdosShieldSensitivity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d_do_s_execution_mode: Option<DdosExecutionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d_do_s_challenge_window: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub whitelabel_response_pages: Option<bool>,
}

/// Wrapper returned from GET /shield/shield-zone/{shieldZoneId}.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetShieldZoneResponse {
    #[serde(default)]
    pub data: Option<ShieldZoneResponse>,
}

/// Wrapper returned from GET /shield/shield-zones.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetShieldZonesResponse {
    #[serde(default)]
    pub data: Option<Vec<ShieldZoneResponse>>,
    #[serde(default)]
    pub page: Option<PaginationResponse>,
}

// ---------------------------------------------------------------------------
// Custom WAF rules
// ---------------------------------------------------------------------------

/// WAF rule variable matching targets (request/response parts to inspect).
///
/// Values are the variable names as strings; the key is the variable name
/// and the value is an optional argument (e.g. a header name for REQUEST_HEADERS).
pub type WafRuleVariableTypes = HashMap<String, String>;

/// A single chained condition appended to a WAF rule via AND logic.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafChainedRuleCondition {
    #[serde(default)]
    pub variable_types: Option<WafRuleVariableTypes>,
    pub operator_type: WafRuleOperatorType,
    #[serde(default)]
    pub value: Option<String>,
}

/// Configuration payload shared by create and update WAF rule requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRuleConfiguration {
    pub action_type: WafRuleActionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variable_types: Option<WafRuleVariableTypes>,
    pub operator_type: WafRuleOperatorType,
    pub severity_type: WafRuleSeverityType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transformation_types: Option<Vec<WafRuleTransformationType>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chained_rule_conditions: Option<Vec<WafChainedRuleCondition>>,
}

/// A custom WAF rule as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomWafRule {
    pub id: i64,
    pub shield_zone_id: i64,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub rule_name: Option<String>,
    #[serde(default)]
    pub rule_description: Option<String>,
    #[serde(default)]
    pub rule_json: Option<String>,
    #[serde(default)]
    pub rule_configuration: Option<WafRuleConfiguration>,
    #[serde(default)]
    pub error_response: Option<GenericRequestResponse>,
}

/// Request body for creating a new custom WAF rule.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomWafRule {
    pub shield_zone_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_description: Option<String>,
    pub rule_configuration: WafRuleConfiguration,
}

/// Request body for updating an existing custom WAF rule.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomWafRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_configuration: Option<WafRuleConfiguration>,
}

/// Paginated list of custom WAF rules.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCustomWafRulesResponse {
    #[serde(default)]
    pub data: Option<Vec<CustomWafRule>>,
    #[serde(default)]
    pub page: Option<PaginationResponse>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// Rate limit rules
// ---------------------------------------------------------------------------

/// Rate limit rule configuration payload (shared by create/update).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitRuleConfiguration {
    pub action_type: RateLimitActionType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variable_types: Option<WafRuleVariableTypes>,
    pub operator_type: WafRuleOperatorType,
    pub severity_type: WafRuleSeverityType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transformation_types: Option<Vec<WafRuleTransformationType>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub request_count: i32,
    pub counter_key_type: RateLimitCounterKey,
    pub timeframe: RateLimitTimeframe,
    pub block_time: RateLimitBlockDuration,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chained_rule_conditions: Option<Vec<WafChainedRuleCondition>>,
}

/// A rate limit rule as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitRule {
    pub id: i64,
    pub shield_zone_id: i64,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub rule_name: Option<String>,
    #[serde(default)]
    pub rule_description: Option<String>,
    #[serde(default)]
    pub rule_json: Option<String>,
    #[serde(default)]
    pub rule_configuration: Option<RateLimitRuleConfiguration>,
    #[serde(default)]
    pub error_response: Option<GenericRequestResponse>,
}

/// Request body for creating a new rate limit rule.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRateLimitRule {
    pub shield_zone_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_description: Option<String>,
    pub rule_configuration: RateLimitRuleConfiguration,
}

/// Request body for updating an existing rate limit rule.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRateLimitRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_configuration: Option<RateLimitRuleConfiguration>,
}

/// List response for rate limit rules.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRateLimitRulesResponse {
    #[serde(default)]
    pub data: Option<Vec<RateLimitRule>>,
    #[serde(default)]
    pub page: Option<PaginationResponse>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// Access Lists
// ---------------------------------------------------------------------------

/// Detailed information about a single access list and its configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListDetails {
    pub list_id: i64,
    pub configuration_id: i64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub is_enabled: bool,
    #[serde(rename = "type")]
    pub list_type: AccessListType,
    pub action: AccessListAction,
    pub entry_count: i64,
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Full access list response for a Shield Zone (managed + custom).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListsDetailsResponse {
    #[serde(default)]
    pub managed_lists: Option<Vec<AccessListDetails>>,
    #[serde(default)]
    pub custom_lists: Option<Vec<AccessListDetails>>,
    #[serde(default)]
    pub custom_entry_count: Option<i32>,
    #[serde(default)]
    pub custom_entry_limit: Option<i32>,
    #[serde(default)]
    pub custom_list_count: Option<i32>,
    #[serde(default)]
    pub custom_list_limit: Option<i32>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

/// A custom access list as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAccessList {
    pub id: i64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub list_type: AccessListType,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub entry_count: Option<i64>,
    #[serde(default)]
    pub last_modified: Option<String>,
}

/// Response wrapper for a single custom access list.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAccessListResponse {
    #[serde(default)]
    pub data: Option<CustomAccessList>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

/// Request body for creating a new custom access list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCustomAccessList {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub list_type: AccessListType,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Request body for updating an existing custom access list.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomAccessList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

/// Request body for updating an access list configuration (enable/disable, action).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccessListConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<AccessListAction>,
}

// ---------------------------------------------------------------------------
// Bot Detection
// ---------------------------------------------------------------------------

/// Request integrity detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestIntegrityConfiguration {
    #[serde(default)]
    pub sensitivity: Option<BotDetectionSensitivity>,
}

/// IP address reputation detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpAddressConfiguration {
    #[serde(default)]
    pub sensitivity: Option<BotDetectionSensitivity>,
}

/// Browser fingerprinting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserFingerprintConfiguration {
    #[serde(default)]
    pub sensitivity: Option<BotDetectionSensitivity>,
    #[serde(default)]
    pub aggression: Option<BrowserFingerprintAggression>,
    #[serde(default)]
    pub complex_enabled: Option<bool>,
}

/// Full bot detection configuration state for a Shield Zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotDetectionConfigurationState {
    #[serde(default)]
    pub shield_zone_id: i64,
    pub request_integrity: RequestIntegrityConfiguration,
    pub ip_address: IpAddressConfiguration,
    pub browser_fingerprint: BrowserFingerprintConfiguration,
    #[serde(default)]
    pub execution_mode: Option<BotDetectionExecutionMode>,
}

/// Response wrapper for bot detection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotDetectionConfigurationResponse {
    #[serde(default)]
    pub data: Option<BotDetectionConfigurationState>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

/// Request body for updating bot detection settings.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotDetection {
    pub shield_zone_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<BotDetectionExecutionMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_integrity: Option<RequestIntegrityConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<IpAddressConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_fingerprint: Option<BrowserFingerprintConfiguration>,
}

/// Response wrapper for update bot detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotDetectionResponse {
    #[serde(default)]
    pub data: Option<BotDetectionConfigurationState>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// WAF Profiles
// ---------------------------------------------------------------------------

/// A minimal WAF profile (preset rule configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafProfileMinimal {
    pub id: i32,
    #[serde(default)]
    pub name: Option<String>,
    pub is_premium: bool,
    #[serde(default)]
    pub profile_category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub features: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waf_execution_mode_round_trips() {
        let json = serde_json::to_string(&WafExecutionMode::Enabled).unwrap();
        assert_eq!(json, "1");
        let decoded: WafExecutionMode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, WafExecutionMode::Enabled);
    }

    #[test]
    fn access_list_action_round_trips() {
        let json = serde_json::to_string(&AccessListAction::Block).unwrap();
        assert_eq!(json, "1");
        let decoded: AccessListAction = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, AccessListAction::Block);
    }

    #[test]
    fn rate_limit_timeframe_round_trips() {
        let json = serde_json::to_string(&RateLimitTimeframe::Sec3600).unwrap();
        assert_eq!(json, "3600");
        let decoded: RateLimitTimeframe = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, RateLimitTimeframe::Sec3600);
    }

    #[test]
    fn rate_limit_block_duration_round_trips() {
        let json = serde_json::to_string(&RateLimitBlockDuration::Sec1800).unwrap();
        assert_eq!(json, "1800");
        let decoded: RateLimitBlockDuration = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, RateLimitBlockDuration::Sec1800);
    }

    #[test]
    fn problem_details_display() {
        let p = ProblemDetails {
            problem_type: None,
            title: Some("Unauthorized".to_string()),
            status: Some(401),
            detail: Some("API key missing".to_string()),
            instance: None,
        };
        let s = p.to_string();
        assert!(s.contains("401"));
        assert!(s.contains("Unauthorized"));
        assert!(s.contains("API key missing"));
    }

    #[test]
    fn shield_zone_response_deserializes_partial() {
        let json = r#"{
            "shieldZoneId": 42,
            "pullZoneId": 100,
            "wafEnabled": true,
            "wafExecutionMode": 1
        }"#;
        let zone: ShieldZoneResponse = serde_json::from_str(json).unwrap();
        assert_eq!(zone.shield_zone_id, 42);
        assert_eq!(zone.pull_zone_id, Some(100));
        assert_eq!(zone.waf_execution_mode, Some(WafExecutionMode::Enabled));
    }

    #[test]
    fn create_custom_waf_rule_serializes_correctly() {
        let rule = CreateCustomWafRule {
            shield_zone_id: 1,
            rule_name: Some("Block SQL injection".to_string()),
            rule_description: None,
            rule_configuration: WafRuleConfiguration {
                action_type: WafRuleActionType::Block,
                variable_types: None,
                operator_type: WafRuleOperatorType::Contains,
                severity_type: WafRuleSeverityType::High,
                transformation_types: None,
                value: Some("SELECT".to_string()),
                chained_rule_conditions: None,
            },
        };
        let json = serde_json::to_string(&rule).unwrap();
        // actionType must be 1 (Block)
        assert!(json.contains("\"actionType\":1"));
        // operatorType must be 2 (Contains)
        assert!(json.contains("\"operatorType\":2"));
        // rule_description absent because None + skip_serializing_if
        assert!(!json.contains("ruleDescription"));
    }

    #[test]
    fn waf_rule_operator_type_values() {
        assert_eq!(
            serde_json::to_string(&WafRuleOperatorType::IpMatch).unwrap(),
            "15"
        );
        assert_eq!(
            serde_json::to_string(&WafRuleOperatorType::GeoLookup).unwrap(),
            "17"
        );
        assert_eq!(
            serde_json::to_string(&WafRuleOperatorType::Pm).unwrap(),
            "12"
        );
    }
}
