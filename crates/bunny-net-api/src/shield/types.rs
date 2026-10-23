use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// ---------------------------------------------------------------------------
// Enums
//
// Variant names, `Display` output and the CLI-accepted names all come from the
// spec's `x-enum-varnames` (`specs/shield.json`). Wire numbers are what the API
// sends; the labels are only for humans. Iter-90 fixed a batch of enums whose
// labels had never matched the spec (see CHANGELOG, "Breaking").
// ---------------------------------------------------------------------------

forward_compat_repr_enum! {
    /// WAF execution mode for a Shield Zone (spec `WAFExecutionMode`).
    ///
    /// Values: 0 = Log, 1 = Block.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafExecutionMode: u8 {
        Log = 0,
        Block = 1,
    }
}

repr_enum_names!(WafExecutionMode: u8, "WAF execution mode", [
    Log => "Log",
    Block => "Block",
]);

forward_compat_repr_enum! {
    /// Action taken when WAF payload size limit is exceeded (spec
    /// `WAFPayloadLimitAction`).
    ///
    /// Values: 0 = Block, 1 = Log, 2 = Ignore.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafPayloadLimitAction: u8 {
        Block = 0,
        Log = 1,
        Ignore = 2,
    }
}

repr_enum_names!(WafPayloadLimitAction: u8, "payload limit action", [
    Block => "Block",
    Log => "Log",
    Ignore => "Ignore",
]);

forward_compat_repr_enum! {
    /// Shield plan tier (spec `ShieldPlanType`).
    ///
    /// Values: 0 = Basic, 1 = Advanced, 2 = Business, 3 = Enterprise.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ShieldPlanType: u8 {
        Basic = 0,
        Advanced = 1,
        Business = 2,
        Enterprise = 3,
    }
}

repr_enum_names!(ShieldPlanType: u8, "plan type", [
    Basic => "Basic",
    Advanced => "Advanced",
    Business => "Business",
    Enterprise => "Enterprise",
]);

forward_compat_repr_enum! {
    /// DDoS protection execution mode (spec `DDoSExecutionMode`).
    ///
    /// Values: 0 = Log, 1 = Block.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DdosExecutionMode: u8 {
        Log = 0,
        Block = 1,
    }
}

repr_enum_names!(DdosExecutionMode: u8, "DDoS execution mode", [
    Log => "Log",
    Block => "Block",
]);

forward_compat_repr_enum! {
    /// DDoS shield detection sensitivity level.
    ///
    /// Values from the spec: 0 = Off, 1 = Low, 2 = Medium, 3 = High,
    /// 4 = Challenge, 5 = ChallengeInteractive.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DdosShieldSensitivity: u8 {
        Off = 0,
        Low = 1,
        Medium = 2,
        High = 3,
        Challenge = 4,
        ChallengeInteractive = 5,
    }
}

repr_enum_names!(DdosShieldSensitivity: u8, "DDoS sensitivity", [
    Off => "Off",
    Low => "Low",
    Medium => "Medium",
    High => "High",
    Challenge => "Challenge",
    ChallengeInteractive => "ChallengeInteractive",
]);

forward_compat_repr_enum! {
    /// Action to take when an access list entry matches a request (spec
    /// `AccessListAction`).
    ///
    /// Values: 0 = None, 1 = Allow, 2 = Block, 3 = Challenge, 4 = Log,
    /// 5 = Bypass. Confirmed against the live `access-list enums` endpoint in
    /// iter-90; before that hoppy had 1 and 2 swapped.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccessListAction: u8 {
        None = 0,
        Allow = 1,
        Block = 2,
        Challenge = 3,
        Log = 4,
        Bypass = 5,
    }
}

repr_enum_names!(AccessListAction: u8, "access list action", [
    None => "None",
    Allow => "Allow",
    Block => "Block",
    Challenge => "Challenge",
    Log => "Log",
    Bypass => "Bypass",
]);

forward_compat_repr_enum! {
    /// Type of access list (spec `AccessListType`).
    ///
    /// Values: 0 = IP, 1 = CIDR, 2 = ASN, 3 = Country, 4 = Organization,
    /// 5 = JA4. Confirmed live in iter-90 (a CIDR list is wire `1`, and wire `3`
    /// validates its content as country codes).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccessListType: u8 {
        Ip = 0,
        Cidr = 1,
        Asn = 2,
        Country = 3,
        Organization = 4,
        Ja4 = 5,
    }
}

repr_enum_names!(AccessListType: u8, "access list type", [
    Ip => "IP",
    Cidr => "CIDR",
    Asn => "ASN",
    Country => "Country",
    Organization => "Organization",
    Ja4 => "JA4",
]);

forward_compat_repr_enum! {
    /// Action taken when a rate limit rule is breached (spec
    /// `RatelimitRuleActionType`).
    ///
    /// Values: 1 = RateLimit, 2 = Log, 3 = Challenge.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitActionType: u8 {
        RateLimit = 1,
        Log = 2,
        Challenge = 3,
    }
}

repr_enum_names!(RateLimitActionType: u8, "rate-limit action type", [
    RateLimit => "RateLimit",
    Log => "Log",
    Challenge => "Challenge",
]);

forward_compat_repr_enum! {
    /// Time window (in seconds) for rate limit counting.
    ///
    /// Values from the spec: 1, 10, 60, 300, 900, 3600.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitTimeframe: u16 {
        Sec1 = 1,
        Sec10 = 10,
        Sec60 = 60,
        Sec300 = 300,
        Sec900 = 900,
        Sec3600 = 3600,
    }
}

impl std::fmt::Display for RateLimitTimeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitTimeframe::Sec1 => write!(f, "1s"),
            RateLimitTimeframe::Sec10 => write!(f, "10s"),
            RateLimitTimeframe::Sec60 => write!(f, "60s"),
            RateLimitTimeframe::Sec300 => write!(f, "5m"),
            RateLimitTimeframe::Sec900 => write!(f, "15m"),
            RateLimitTimeframe::Sec3600 => write!(f, "1h"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Duration (in seconds) to block after a rate limit breach.
    ///
    /// Values from the spec: 30, 60, 300, 900, 1800, 3600.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitBlockDuration: u16 {
        Sec30 = 30,
        Sec60 = 60,
        Sec300 = 300,
        Sec900 = 900,
        Sec1800 = 1800,
        Sec3600 = 3600,
    }
}

impl std::fmt::Display for RateLimitBlockDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitBlockDuration::Sec30 => write!(f, "30s"),
            RateLimitBlockDuration::Sec60 => write!(f, "60s"),
            RateLimitBlockDuration::Sec300 => write!(f, "5m"),
            RateLimitBlockDuration::Sec900 => write!(f, "15m"),
            RateLimitBlockDuration::Sec1800 => write!(f, "30m"),
            RateLimitBlockDuration::Sec3600 => write!(f, "1h"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// What key to count requests by for rate limiting (spec
    /// `WafRatelimitCounterKeyType`).
    ///
    /// Values: 0 = IP, 1 = Host, 2 = Country, 3 = City, 4 = ASN,
    /// 5 = Organization, 6 = JA4, 7 = IP_JA4.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitCounterKey: u8 {
        Ip = 0,
        Host = 1,
        Country = 2,
        City = 3,
        Asn = 4,
        Organization = 5,
        Ja4 = 6,
        IpJa4 = 7,
    }
}

repr_enum_names!(RateLimitCounterKey: u8, "counter key type", [
    Ip => "IP",
    Host => "Host",
    Country => "Country",
    City => "City",
    Asn => "ASN",
    Organization => "Organization",
    Ja4 => "JA4",
    IpJa4 => "IP_JA4",
]);

forward_compat_repr_enum! {
    /// Action taken by a custom WAF rule (spec `WafRuleActionType`).
    ///
    /// Values: 1 = Block, 2 = Log, 3 = Challenge, 4 = Allow, 5 = Bypass.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleActionType: u8 {
        Block = 1,
        Log = 2,
        Challenge = 3,
        Allow = 4,
        Bypass = 5,
    }
}

repr_enum_names!(WafRuleActionType: u8, "WAF rule action type", [
    Block => "Block",
    Log => "Log",
    Challenge => "Challenge",
    Allow => "Allow",
    Bypass => "Bypass",
]);

forward_compat_repr_enum! {
    /// Operator used to match a WAF rule variable against a value.
    ///
    /// Names and numbers follow the shield spec (`WafRuleOperatorType`):
    /// 0 = BEGINSWITH, 1 = ENDSWITH, 2 = CONTAINS, 3 = CONTAINSWORD,
    /// 4 = STRMATCH, 5 = EQ, 6 = GE, 7 = GT, 8 = LE, 9 = LT, 12 = WITHIN,
    /// 13 = IPMATCH, 14 = RX, 15 = STREQ, 17 = DETECTSQLI, 18 = DETECTXSS.
    ///
    /// Before iter-87 these variants carried wrong labels (`0 = Eq`,
    /// `6 = Regex`, `15 = IpMatch`, ...). The wire numbers never changed, only
    /// the names.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleOperatorType: u8 {
        BeginsWith = 0,
        EndsWith = 1,
        Contains = 2,
        ContainsWord = 3,
        StrMatch = 4,
        Eq = 5,
        Ge = 6,
        Gt = 7,
        Le = 8,
        Lt = 9,
        Within = 12,
        IpMatch = 13,
        Rx = 14,
        StrEq = 15,
        DetectSqli = 17,
        DetectXss = 18,
    }
}

repr_enum_names!(WafRuleOperatorType: u8, "operator", [
    BeginsWith => "BEGINSWITH",
    EndsWith => "ENDSWITH",
    Contains => "CONTAINS",
    ContainsWord => "CONTAINSWORD",
    StrMatch => "STRMATCH",
    Eq => "EQ",
    Ge => "GE",
    Gt => "GT",
    Le => "LE",
    Lt => "LT",
    Within => "WITHIN",
    IpMatch => "IPMATCH",
    Rx => "RX",
    StrEq => "STREQ",
    DetectSqli => "DETECTSQLI",
    DetectXss => "DETECTXSS",
]);

forward_compat_repr_enum! {
    /// Severity level of a custom WAF rule (spec `WafRuleSeverityType`).
    ///
    /// Values: 0 = NOTICE, 1 = WARNING, 2 = CRITICAL.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleSeverityType: u8 {
        Notice = 0,
        Warning = 1,
        Critical = 2,
    }
}

repr_enum_names!(WafRuleSeverityType: u8, "severity type", [
    Notice => "NOTICE",
    Warning => "WARNING",
    Critical => "CRITICAL",
]);

forward_compat_repr_enum! {
    /// Transformation applied to the variable value before matching (spec
    /// `WafRuleTransformationType`).
    ///
    /// Values: 1 = CMDLINE, 2 = COMPRESSWHITESPACE, 3 = CSSDECODE,
    /// 4 = HEXENCODE, 5 = HTMLENTITYDECODE, 6 = JSDECODE, 7 = LENGTH,
    /// 8 = LOWERCASE, 9 = MD5, 10 = NORMALIZEPATH, 11 = NORMALISEPATH,
    /// 12 = NORMALIZEPATHWIN, 13 = NORMALISEPATHWIN, 14 = REMOVECOMMENTS,
    /// 15 = REMOVENULLS, 16 = REMOVEWHITESPACE, 17 = REPLACECOMMENTS,
    /// 18 = SHA1, 19 = URLDECODE, 20 = URLDECODEUNI, 21 = UTF8TOUNICODE.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleTransformationType: u8 {
        CmdLine = 1,
        CompressWhitespace = 2,
        CssDecode = 3,
        HexEncode = 4,
        HtmlEntityDecode = 5,
        JsDecode = 6,
        Length = 7,
        Lowercase = 8,
        Md5 = 9,
        NormalizePath = 10,
        NormalisePath = 11,
        NormalizePathWin = 12,
        NormalisePathWin = 13,
        RemoveComments = 14,
        RemoveNulls = 15,
        RemoveWhitespace = 16,
        ReplaceComments = 17,
        Sha1 = 18,
        UrlDecode = 19,
        UrlDecodeUni = 20,
        Utf8ToUnicode = 21,
    }
}

repr_enum_names!(WafRuleTransformationType: u8, "transformation type", [
    CmdLine => "CMDLINE",
    CompressWhitespace => "COMPRESSWHITESPACE",
    CssDecode => "CSSDECODE",
    HexEncode => "HEXENCODE",
    HtmlEntityDecode => "HTMLENTITYDECODE",
    JsDecode => "JSDECODE",
    Length => "LENGTH",
    Lowercase => "LOWERCASE",
    Md5 => "MD5",
    NormalizePath => "NORMALIZEPATH",
    NormalisePath => "NORMALISEPATH",
    NormalizePathWin => "NORMALIZEPATHWIN",
    NormalisePathWin => "NORMALISEPATHWIN",
    RemoveComments => "REMOVECOMMENTS",
    RemoveNulls => "REMOVENULLS",
    RemoveWhitespace => "REMOVEWHITESPACE",
    ReplaceComments => "REPLACECOMMENTS",
    Sha1 => "SHA1",
    UrlDecode => "URLDECODE",
    UrlDecodeUni => "URLDECODEUNI",
    Utf8ToUnicode => "UTF8TOUNICODE",
]);

forward_compat_repr_enum! {
    /// Bot detection execution mode (spec `BotDetectionExecutionMode`).
    ///
    /// Values: 0 = LogOnly, 1 = Challenge.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BotDetectionExecutionMode: u8 {
        LogOnly = 0,
        Challenge = 1,
    }
}

repr_enum_names!(BotDetectionExecutionMode: u8, "execution mode", [
    LogOnly => "LogOnly",
    Challenge => "Challenge",
]);

forward_compat_repr_enum! {
    /// Sensitivity level for bot detection signals (spec
    /// `BotDetectionSensitivity`).
    ///
    /// Values: 0 = Off, 1 = Low, 2 = Medium, 3 = High.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BotDetectionSensitivity: u8 {
        Off = 0,
        Low = 1,
        Medium = 2,
        High = 3,
    }
}

repr_enum_names!(BotDetectionSensitivity: u8, "sensitivity", [
    Off => "Off",
    Low => "Low",
    Medium => "Medium",
    High => "High",
]);

forward_compat_repr_enum! {
    /// Aggressiveness of browser fingerprint challenge injection (spec
    /// `BrowserFingerprintAggression`).
    ///
    /// Values: 0 = Unknown, 1 = Low, 2 = Medium, 3 = High, 4 = Custom. The
    /// spec's `Unknown` (0) is spelled `Unspecified` in Rust because
    /// `Unknown(n)` is the forward-compat catch-all variant; its display name
    /// stays `Unknown`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BrowserFingerprintAggression: u8 {
        Unspecified = 0,
        Low = 1,
        Medium = 2,
        High = 3,
        Custom = 4,
    }
}

repr_enum_names!(BrowserFingerprintAggression: u8, "fingerprint aggression", [
    Unspecified => "Unknown",
    Low => "Low",
    Medium => "Medium",
    High => "High",
    Custom => "Custom",
]);

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

// ---------------------------------------------------------------------------
// Nested error envelope (actual Shield API error shape)
// ---------------------------------------------------------------------------

/// Inner error object nested under `"error"` in Shield API error responses.
///
/// Real shape (non-2xx): `{"error": {"statusCode": 404, "errorKey": "zone.not_found", "message": "..."}, "data": null}`
/// Real shape (202 plan-gate): `{"data": null, "error": {"statusCode": 202, "success": false, "message": "...", "errorKey": "..."}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldApiErrorInner {
    #[serde(default)]
    pub status_code: Option<i32>,
    #[serde(default)]
    pub success: Option<bool>,
    #[serde(default)]
    pub error_key: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// Top-level envelope wrapping [`ShieldApiErrorInner`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldApiErrorEnvelope {
    pub error: ShieldApiErrorInner,
}

impl std::fmt::Display for ShieldApiErrorEnvelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let e = &self.error;
        let code = e.status_code.unwrap_or(0);
        match (&e.error_key, &e.message) {
            (Some(key), Some(msg)) => write!(f, "Shield API error {code}: {key}: {msg}"),
            (Some(key), None) => write!(f, "Shield API error {code}: {key}"),
            (None, Some(msg)) => write!(f, "Shield API error {code}: {msg}"),
            (None, None) => write!(f, "Shield API error {code}"),
        }
    }
}

impl std::error::Error for ShieldApiErrorEnvelope {}

/// Alternate Shield error envelope used by endpoints like
/// `/shield/event-logs/...` whose success body embeds an
/// `errorResponse` field instead of the top-level `error` envelope.
/// The inner shape mirrors `ShieldApiErrorInner` (carries `statusCode`
/// on error bodies), not bunny.net's `GenericRequestResponse`.
///
/// Real shape: `{"logs": null, ..., "errorResponse": {"statusCode": 401, "success": false, "message": "...", "errorKey": "..."}}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldErrorResponseEnvelope {
    pub error_response: ShieldApiErrorInner,
}

impl std::fmt::Display for ShieldErrorResponseEnvelope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let e = &self.error_response;
        let code = e.status_code.unwrap_or(0);
        match (&e.error_key, &e.message) {
            (Some(key), Some(msg)) => write!(f, "Shield API error {code} ({key}): {msg}"),
            (Some(key), None) => write!(f, "Shield API error {code} ({key})"),
            (None, Some(msg)) => write!(f, "Shield API error {code}: {msg}"),
            (None, None) => write!(f, "Shield API error {code}"),
        }
    }
}

impl std::error::Error for ShieldErrorResponseEnvelope {}

/// Test whether the inner error object carries a real error signal: an
/// explicit `success=false`, or a message / errorKey. Used to gate
/// `errorResponse`-style envelopes which are also present on happy-path
/// responses (with null values).
fn shield_inner_has_error_signal(inner: &ShieldApiErrorInner) -> bool {
    inner.success == Some(false) || inner.message.is_some() || inner.error_key.is_some()
}

/// Parse a Shield API error body, trying the real nested envelope first, then
/// the RFC 7807 `ProblemDetails` format as a fallback.
///
/// Returns `Some(err)` when either format is recognised, `None` otherwise.
pub fn parse_shield_error(bytes: &[u8]) -> Option<anyhow::Error> {
    if let Ok(env) = serde_json::from_slice::<ShieldApiErrorEnvelope>(bytes) {
        return Some(anyhow::anyhow!(env));
    }
    if let Ok(env) = serde_json::from_slice::<ShieldErrorResponseEnvelope>(bytes)
        && shield_inner_has_error_signal(&env.error_response)
    {
        return Some(anyhow::anyhow!(env));
    }
    if let Ok(problem) = serde_json::from_slice::<ProblemDetails>(bytes) {
        // ProblemDetails has all-optional fields, so it deserialises from any
        // JSON object. Only treat the body as RFC 7807 when at least one of
        // the spec's fields is present — otherwise return None and let the
        // caller fall back to status code + raw body.
        let has_rfc7807_field = problem.problem_type.is_some()
            || problem.title.is_some()
            || problem.status.is_some()
            || problem.detail.is_some()
            || problem.instance.is_some();
        if has_rfc7807_field {
            return Some(anyhow::anyhow!(problem));
        }
    }
    None
}

/// Check a 2xx response body for an error envelope where `error.success == false`.
///
/// bunny.net Shield returns HTTP 202 with this shape when a feature is gated by
/// the account's plan tier:
///
/// ```json
/// {"data": null, "error": {"statusCode": 202, "success": false, "message": "...", "errorKey": "..."}}
/// ```
///
/// Returns `Some(err)` when the body contains such an envelope, `None` otherwise
/// (meaning the caller should proceed to deserialise the body normally).
pub fn parse_shield_2xx_envelope_error(bytes: &[u8]) -> Option<anyhow::Error> {
    if let Ok(env) = serde_json::from_slice::<ShieldApiErrorEnvelope>(bytes) {
        // Only treat as error when success is explicitly false AND there's a
        // human-readable signal (message or errorKey). This avoids false positives
        // from happy-path 2xx responses whose embedded error object is empty or
        // only partially populated.
        if env.error.success == Some(false)
            && (env.error.message.is_some() || env.error.error_key.is_some())
        {
            return Some(anyhow::anyhow!(env));
        }
    }
    if let Ok(env) = serde_json::from_slice::<ShieldErrorResponseEnvelope>(bytes) {
        let inner = &env.error_response;
        if inner.success == Some(false) && (inner.message.is_some() || inner.error_key.is_some()) {
            return Some(anyhow::anyhow!(env));
        }
    }
    None
}

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
    // `dDoSExecutionMode` was removed from the request schema in the
    // 2026-10-23 spec, and a live update on a Basic-plan zone left the value
    // unchanged (iter-87). It stays on `ShieldZoneResponse` only.
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
    /// Negate the match (`isNegated`). Echoed back on updates so an
    /// existing negated rule keeps its meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_negated: Option<bool>,
    /// Treat the variable selectors as regular expressions (`isRegexVariable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_regex_variable: Option<bool>,
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
    /// Negate the match (`isNegated`). Echoed back on updates so an
    /// existing negated rule keeps its meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_negated: Option<bool>,
    /// Treat the variable selectors as regular expressions (`isRegexVariable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_regex_variable: Option<bool>,
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
    /// Negate the match (`isNegated`). Echoed back on updates so an
    /// existing negated rule keeps its meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_negated: Option<bool>,
    /// Treat the variable selectors as regular expressions (`isRegexVariable`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_regex_variable: Option<bool>,
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
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
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

// ---------------------------------------------------------------------------
// Metrics types
// ---------------------------------------------------------------------------

/// Overview metrics for a Shield Zone.
/// Returned by `GET /shield/metrics/overview/{shieldZoneId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldMetricsResponse {
    pub data: Option<ShieldMetricsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldMetricsData {
    pub overview: Option<ShieldOverviewSummary>,
    pub waf: Option<ShieldWafSummary>,
    #[serde(rename = "dDoS")]
    pub ddos: Option<ShieldDdosSummary>,
    pub ratelimit: Option<ShieldRatelimitSummary>,
    pub bot_detection: Option<ShieldBotDetectionSummary>,
    pub access_list: Option<ShieldAccessListSummary>,
    pub upload_scanning: Option<ShieldUploadScanningSummary>,
    #[serde(default)]
    pub total_clean_requests_limit: Option<i64>,
    #[serde(default)]
    pub total_billable_requests: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldOverviewSummary {
    #[serde(default)]
    pub d_do_s_mitigated: i64,
    #[serde(default)]
    pub waf_triggered_rules: i64,
    #[serde(default)]
    pub ratelimit_breaches: i64,
    #[serde(default)]
    pub bot_detection_challenged: i64,
    #[serde(default)]
    pub access_list_actions: i64,
    #[serde(default)]
    pub upload_scanning_blocks: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldWafSummary {
    #[serde(default)]
    pub total_triggered_rules: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldDdosSummary {
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub verified_requests: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldRatelimitSummary {
    #[serde(default)]
    pub total_breaches: i64,
    #[serde(default)]
    pub logged_breaches: i64,
    #[serde(default)]
    pub challenged_breaches: i64,
    #[serde(default)]
    pub blocked_breaches: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldBotDetectionSummary {
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldAccessListSummary {
    #[serde(default)]
    pub total_actions: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldUploadScanningSummary {
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub files_scanned: i64,
}

// ---------------------------------------------------------------------------
// Detailed metrics types
// ---------------------------------------------------------------------------

/// Detailed overview metrics for a Shield Zone.
/// Returned by `GET /shield/metrics/overview/{shieldZoneId}/detailed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldDetailedMetricsResponse {
    pub data: Option<ShieldDetailedMetricsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldDetailedMetricsData {
    pub waf: Option<WafDetailCategory>,
    #[serde(rename = "ddos")]
    pub ddos: Option<DdosDetailCategory>,
    pub rate_limit: Option<RateLimitDetailCategory>,
    pub access_lists: Option<AccessListDetailCategory>,
    pub bot_detection: Option<BotDetectionDetailCategory>,
    pub upload_scanning: Option<UploadScanningDetailCategory>,
    #[serde(default)]
    pub total_clean_requests_limit: Option<i64>,
    #[serde(default)]
    pub total_billable_requests_this_month: Option<i64>,
    #[serde(default)]
    pub resolution: Option<i32>,
}

/// Per-timestamp metrics for WAF and access lists (blocked/logged/challenged requests).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedLoggedChallengedMetrics {
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafDetailCategory {
    pub metrics: HashMap<String, BlockedLoggedChallengedMetrics>,
    pub totals: Option<BlockedLoggedChallengedMetrics>,
}

/// Per-timestamp metrics for DDoS (blocked/verified/challenged requests).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DdosDetailMetrics {
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub verified_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DdosDetailCategory {
    pub metrics: HashMap<String, DdosDetailMetrics>,
    pub totals: Option<DdosDetailMetrics>,
}

/// Per-timestamp metrics for rate limits (total/blocked/logged/challenged breaches).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitDetailMetrics {
    #[serde(default)]
    pub total_breaches: i64,
    #[serde(default)]
    pub blocked_breaches: i64,
    #[serde(default)]
    pub logged_breaches: i64,
    #[serde(default)]
    pub challenged_breaches: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitDetailCategory {
    pub metrics: HashMap<String, RateLimitDetailMetrics>,
    pub totals: Option<RateLimitDetailMetrics>,
}

/// Access list category reuses `BlockedLoggedChallengedMetrics` (same shape as WAF).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessListDetailCategory {
    pub metrics: HashMap<String, BlockedLoggedChallengedMetrics>,
    pub totals: Option<BlockedLoggedChallengedMetrics>,
}

/// Per-timestamp metrics for bot detection (logged/challenged requests).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotDetectionDetailMetrics {
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotDetectionDetailCategory {
    pub metrics: HashMap<String, BotDetectionDetailMetrics>,
    pub totals: Option<BotDetectionDetailMetrics>,
}

/// Per-timestamp metrics for upload scanning (logged/blocked requests + files scanned).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadScanningDetailMetrics {
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub files_scanned: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadScanningDetailCategory {
    pub metrics: HashMap<String, UploadScanningDetailMetrics>,
    pub totals: Option<UploadScanningDetailMetrics>,
}

// ---------------------------------------------------------------------------
// API Guardian types
// ---------------------------------------------------------------------------

/// A single API Guardian endpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGuardianEndpoint {
    #[serde(default)]
    pub api_guardian_endpoint_id: Option<i32>,
    #[serde(default)]
    pub shield_zone_id: Option<i32>,
    #[serde(default)]
    pub schema_title: Option<String>,
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub request_methods: Option<String>,
    #[serde(default)]
    pub request_path: Option<String>,
    #[serde(default)]
    pub request_body_type: Option<String>,
    #[serde(default)]
    pub request_body_schema: Option<String>,
    #[serde(default)]
    pub response_body_schema: Option<String>,
    #[serde(default)]
    pub ai_guardian: Option<bool>,
    #[serde(default)]
    pub ai_guardian_guardrails: Option<String>,
    #[serde(default)]
    pub validate_request_body_schema: Option<bool>,
    #[serde(default)]
    pub validate_response_body_schema: Option<bool>,
    #[serde(default)]
    pub validate_authorization: Option<bool>,
    #[serde(default)]
    pub authorization_configuration: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub rate_limiting_global_enabled: Option<bool>,
    #[serde(default)]
    pub rate_limiting_global_request_count: Option<i32>,
    #[serde(default)]
    pub rate_limiting_global_timeframe_seconds: Option<i32>,
    #[serde(default)]
    pub rate_limiting_global_block_time_seconds: Option<i32>,
    #[serde(default)]
    pub rate_limiting_per_ip_enabled: Option<bool>,
    #[serde(default)]
    pub rate_limiting_per_ip_request_count: Option<i32>,
    #[serde(default)]
    pub rate_limiting_per_ip_timeframe_seconds: Option<i32>,
    #[serde(default)]
    pub rate_limiting_per_ip_block_time_seconds: Option<i32>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// The list of API Guardian endpoints for a Shield Zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGuardianEndpointsResponse {
    #[serde(default)]
    pub endpoints: Option<Vec<ApiGuardianEndpoint>>,
}

/// Response wrapper for GET /shield/shield-zone/{shieldZoneId}/api-guardian.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetApiGuardianResponse {
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
    #[serde(default)]
    pub data: Option<ApiGuardianEndpointsResponse>,
}

/// Response for `GET /shield/shield-zone/{shieldZoneId}/api-guardian/enums`.
///
/// A map of enum-type name to a map of value name to value. Ordered maps keep
/// the rendered output deterministic. Example:
/// `{ "executionMode": { "log": "0", "block": "1" } }`.
pub type ApiGuardianEnumsResponse = BTreeMap<String, BTreeMap<String, String>>;

/// Request body for POST /shield/shield-zone/{shieldZoneId}/api-guardian/spec
/// (upload a new OpenAPI specification).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadOpenApiSpecificationRequest {
    /// The file contents of the OpenAPI specification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// If true, enforce authentication requirements for all endpoints defined in the spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_authorisation_validation: Option<bool>,
}

/// Response wrapper for POST /shield/shield-zone/{shieldZoneId}/api-guardian.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadOpenApiSpecificationResponse {
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
    #[serde(default)]
    pub data: Option<ApiGuardianEndpointsResponse>,
}

/// Request body for PATCH /shield/shield-zone/{shieldZoneId}/api-guardian
/// (update an existing API Guardian configuration by uploading an updated spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiGuardianRequest {
    /// The file contents of the OpenAPI specification.
    pub content: String,
    /// If true, enforce authentication requirements for all endpoints defined in the spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforce_authorisation_validation: Option<bool>,
}

/// Response wrapper for PATCH /shield/shield-zone/{shieldZoneId}/api-guardian.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiGuardianResponse {
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
    #[serde(default)]
    pub data: Option<ApiGuardianEndpointsResponse>,
}

/// Request body for PATCH /shield/shield-zone/{shieldZoneId}/api-guardian/endpoint/{endpointId}.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiGuardianEndpointRequest {
    /// Whether the endpoint is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Whether to validate the request body schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_request_body_schema: Option<bool>,
    /// Whether to validate the response body schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_response_body_schema: Option<bool>,
    /// Whether to validate authorization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate_authorization: Option<bool>,
}

/// Response wrapper for PATCH /shield/shield-zone/{shieldZoneId}/api-guardian/endpoint/{endpointId}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiGuardianEndpointResponse {
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
    #[serde(default)]
    pub data: Option<ApiGuardianEndpoint>,
}

// ---------------------------------------------------------------------------
// Upload Scanning types
// ---------------------------------------------------------------------------

forward_compat_repr_enum! {
    /// Scanner mode for upload scanning (antivirus/CSAM), spec
    /// `UploadScanningScannerMode`.
    ///
    /// Values: 0 = Disabled, 1 = Log, 2 = Block. Before iter-90 `Log` was
    /// spelled `LogOnly`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UploadScanningScannerMode: u8 {
        Disabled = 0,
        Log = 1,
        Block = 2,
    }
}

repr_enum_names!(UploadScanningScannerMode: u8, "scanning mode", [
    Disabled => "Disabled",
    Log => "Log",
    Block => "Block",
]);

/// Current upload scanning configuration state for a Shield Zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadScanningConfigurationState {
    #[serde(default)]
    pub shield_zone_id: Option<i32>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
    #[serde(default)]
    pub csam_scanning_mode: Option<UploadScanningScannerMode>,
    #[serde(default)]
    pub antivirus_scanning_mode: Option<UploadScanningScannerMode>,
}

/// Response wrapper for GET /shield/shield-zone/{shieldZoneId}/upload-scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadScanningConfigurationResponse {
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
    #[serde(default)]
    pub data: Option<UploadScanningConfigurationState>,
}

/// Request body for PATCH /shield/shield-zone/{shieldZoneId}/upload-scanning.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUploadScanningConfigurationRequest {
    pub shield_zone_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csam_scanning_mode: Option<UploadScanningScannerMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antivirus_scanning_mode: Option<UploadScanningScannerMode>,
}

/// Response wrapper for PATCH /shield/shield-zone/{shieldZoneId}/upload-scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUploadScanningConfigurationResponse {
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
    #[serde(default)]
    pub data: Option<UploadScanningConfigurationState>,
}

// ---------------------------------------------------------------------------
// Event Logs types
// ---------------------------------------------------------------------------

/// Labels attached to a WAF event log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLogLabels {
    #[serde(default)]
    pub asn: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub rule_id: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub rule_group: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub server_zone: Option<String>,
}

/// A single WAF event log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLog {
    #[serde(default)]
    pub log_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub log: Option<String>,
    #[serde(default)]
    pub labels: Option<EventLogLabels>,
}

/// Paginated response for GET /shield/event-logs/{shieldZoneId}/{date}/{continuationToken}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafLoggingResponse {
    #[serde(default)]
    pub logs: Option<Vec<EventLog>>,
    #[serde(default)]
    pub has_more_data: Option<bool>,
    #[serde(default)]
    pub continuation_token: Option<String>,
    #[serde(default)]
    pub start_token: Option<String>,
    #[serde(default)]
    pub error_response: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// WAF Triggered Rules types
// ---------------------------------------------------------------------------

forward_compat_repr_enum! {
    /// Action type for reviewing a triggered WAF rule (spec `ReviewActionType`).
    ///
    /// Values: 0 = Ignore, 1 = LogOnly, 2 = DisableRule. Before iter-90 hoppy
    /// labelled these Pending / Approve / Reject.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ReviewActionType: u8 {
        Ignore = 0,
        LogOnly = 1,
        DisableRule = 2,
    }
}

repr_enum_names!(ReviewActionType: u8, "review action", [
    Ignore => "Ignore",
    LogOnly => "LogOnly",
    DisableRule => "DisableRule",
]);

/// A single triggered WAF rule item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggeredRuleItem {
    #[serde(default)]
    pub rule_id: Option<String>,
    #[serde(default)]
    pub rule_description: Option<String>,
    #[serde(default)]
    pub top_targeted_urls: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub total_triggered_requests: Option<i32>,
    #[serde(default)]
    pub rule_logs: Option<Vec<EventLog>>,
}

/// Response for GET /shield/waf/rules/review-triggered/{shieldZoneId}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTriggeredRulesResponse {
    #[serde(default)]
    pub error_response: Option<GenericRequestResponse>,
    #[serde(default)]
    pub triggered_rules: Option<Vec<TriggeredRuleItem>>,
    #[serde(default)]
    pub total_triggered_rules: Option<i32>,
}

/// Request body for POST /shield/waf/rules/review-triggered/{shieldZoneId}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReviewTriggeredRuleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    pub action: ReviewActionType,
}

/// Response for POST /shield/waf/rules/review-triggered/{shieldZoneId}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReviewTriggeredRuleResponse {
    #[serde(default)]
    pub error_response: Option<GenericRequestResponse>,
    #[serde(default)]
    pub success: Option<bool>,
}

/// Response for GET /shield/waf/rules/review-triggered/ai-recommendation/{shieldZoneId}/{ruleId}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggeredRuleRecommendationResponse {
    #[serde(default)]
    pub shield_zone_id: Option<i64>,
    #[serde(default)]
    pub rule_id: Option<String>,
    #[serde(default)]
    pub success: Option<bool>,
    #[serde(default)]
    pub recommendation: Option<String>,
    #[serde(default)]
    pub error_response: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// Supplementary endpoint types
// ---------------------------------------------------------------------------

/// A single WAF rule entry in the plan segmentation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRuleModel {
    #[serde(default)]
    pub rule_id: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// A group of WAF rules within a main group/ruleset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRuleGroupModel {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub main_group: Option<String>,
    #[serde(default)]
    pub ruleset: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<WafRuleModel>>,
}

/// A top-level WAF rule main group with nested rule groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRuleMainGroupModel {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub ruleset: Option<String>,
    #[serde(default)]
    pub rule_groups: Option<Vec<WafRuleGroupModel>>,
}

/// WAF rules segmented by plan tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRulesByPlanModel {
    #[serde(default)]
    pub plan_value: Option<i32>,
    #[serde(default)]
    pub plan_name: Option<String>,
    #[serde(default)]
    pub rules: Option<Vec<WafRuleMainGroupModel>>,
}

/// Response for GET /shield/waf/rules/plan-segmentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWafRulesSegmentedByPlanResponse {
    #[serde(default)]
    pub data: Option<Vec<WafRulesByPlanModel>>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

/// A minimal WAF engine configuration variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVariableValueMinimal {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub value_encoded: Option<String>,
}

/// Response for GET /shield/waf/engine-config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWafEngineConfigResponse {
    #[serde(default)]
    pub data: Option<Vec<ConfigVariableValueMinimal>>,
}

/// A single mapped enum value (name + integer value + premium flag).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafMappedEnum {
    #[serde(default)]
    pub optional_input: Option<bool>,
    #[serde(default)]
    pub is_premium: Option<bool>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub value: Option<i32>,
}

/// A named list of mapped enum values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafMappedEnumList {
    #[serde(default)]
    pub enum_name: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<WafMappedEnum>>,
}

/// Response for GET /shield/ddos/enums.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWafEnumsResponse {
    #[serde(default)]
    pub data: Option<Vec<WafMappedEnumList>>,
}

/// A Shield Zone to Pull Zone mapping entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldZonePullZoneMapping {
    #[serde(default)]
    pub shield_zone_id: Option<i64>,
    #[serde(default)]
    pub pull_zone_id: Option<i64>,
}

/// Response for GET /shield/shield-zones/pullzone-mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetShieldZonePullzoneMappingResponse {
    #[serde(default)]
    pub data: Option<Vec<ShieldZonePullZoneMapping>>,
}

// ---------------------------------------------------------------------------
// Rate limit metrics types
// ---------------------------------------------------------------------------

/// Rate limits metrics for all rules in a Shield Zone.
/// Returned by `GET /shield/metrics/rate-limits/{shieldZoneId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldRateLimitsMetricsResponse {
    pub data: Option<Vec<RateLimitMetricsEntry>>,
}

/// Rate limit metrics for a single rule.
/// Returned by `GET /shield/metrics/rate-limit/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldRateLimitMetricsResponse {
    pub data: Option<RateLimitMetricsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitMetricsEntry {
    #[serde(default)]
    pub ratelimit_id: Option<i64>,
    pub overview: Option<RateLimitDetailMetrics>,
    pub ratelimit_overview_past_twenty_eight_days: Option<HashMap<String, RateLimitDetailMetrics>>,
}

// ---------------------------------------------------------------------------
// WAF rule metrics types
// ---------------------------------------------------------------------------

/// WAF rule metrics for a single rule in a Shield Zone.
/// Returned by `GET /shield/metrics/shield-zone/{shieldZoneId}/waf-rule/{ruleId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldWafRuleMetricsResponse {
    pub data: Option<WafRuleMetricsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRuleMetricsData {
    #[serde(default)]
    pub total_triggers: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
    pub overview_past_twenty_eight_days: Option<HashMap<String, WafRuleDetailMetrics>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WafRuleDetailMetrics {
    #[serde(default)]
    pub total_triggers: i64,
    #[serde(default)]
    pub blocked_requests: i64,
    #[serde(default)]
    pub logged_requests: i64,
    #[serde(default)]
    pub challenged_requests: i64,
}

// ---------------------------------------------------------------------------
// Bot detection metrics types
// ---------------------------------------------------------------------------

/// Bot detection metrics for a Shield Zone.
/// Returned by `GET /shield/metrics/shield-zone/{shieldZoneId}/bot-detection`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldBotDetectionMetricsResponse {
    pub data: Option<BotDetectionMetricsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotDetectionMetricsData {
    pub overview_past_twenty_eight_days: Option<HashMap<String, BotDetectionDetailMetrics>>,
    #[serde(default)]
    pub total_logged_requests: i64,
    #[serde(default)]
    pub total_challenged_requests: i64,
}

// ---------------------------------------------------------------------------
// Upload scanning metrics types
// ---------------------------------------------------------------------------

/// Upload scanning metrics for a Shield Zone.
/// Returned by `GET /shield/metrics/shield-zone/{shieldZoneId}/upload-scanning`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldUploadScanningMetricsResponse {
    pub data: Option<UploadScanningMetricsData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadScanningMetricsData {
    pub overview_past_twenty_eight_days: Option<HashMap<String, UploadScanningDetailMetrics>>,
    #[serde(default)]
    pub total_logged_requests: i64,
    #[serde(default)]
    pub total_blocked_requests: i64,
    #[serde(default)]
    pub total_files_scanned: i64,
}

// ---------------------------------------------------------------------------
// Bot categorization (iter-72)
// ---------------------------------------------------------------------------

forward_compat_repr_enum! {
    /// Action applied to a single categorised bot.
    ///
    /// `0 = None` (clear any override), `1 = Block`, `2 = Allow`, `3 = Ignore`.
    /// Whole categories use [`BotCategoryAction`], which has no `Ignore`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BotCategorizationAction: u8 {
        None = 0,
        Block = 1,
        Allow = 2,
        Ignore = 3,
    }
}

repr_enum_names!(BotCategorizationAction: u8, "bot action", [
    None => "None",
    Block => "Block",
    Allow => "Allow",
    Ignore => "Ignore",
]);

forward_compat_repr_enum! {
    /// Action applied to an entire bot category.
    ///
    /// `0 = None` (clear any override), `1 = Block`, `2 = Allow`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BotCategoryAction: u8 {
        None = 0,
        Block = 1,
        Allow = 2,
    }
}

repr_enum_names!(BotCategoryAction: u8, "category action", [
    None => "None",
    Block => "Block",
    Allow => "Allow",
]);

/// Bot category grouping. The API models this as an integer; `System` uses the
/// out-of-band value `255`. We keep the raw integer so unknown categories from
/// newer API revisions round-trip losslessly rather than failing to deserialise.
///
/// Known values: `0 = None`, `1 = SEO`, `2 = AIScraper`, `3 = AITool`,
/// `4 = Tool`, `5 = Ads`, `6 = Preview`, `7 = Social`, `255 = System`.
pub type BotCategory = i32;

/// One categorised bot within a [`BotCategorizationGroup`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotCategorizationDetails {
    #[serde(default)]
    pub bot_id: i32,
    #[serde(default)]
    pub user_agent_match: Option<String>,
    #[serde(default)]
    pub category: BotCategory,
    pub action: BotCategorizationAction,
    #[serde(default)]
    pub is_verifiable: bool,
}

/// A category and the bots that belong to it, with the category-wide action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotCategorizationGroup {
    #[serde(default)]
    pub category: BotCategory,
    pub category_action: BotCategoryAction,
    #[serde(default)]
    pub bots: Option<Vec<BotCategorizationDetails>>,
}

/// Response for `GET /shield/shield-zone/{shieldZoneId}/bot-categorization`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotCategorizationListResponse {
    #[serde(default)]
    pub categories: Option<Vec<BotCategorizationGroup>>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

/// Request body for `PUT …/bot-categorization/bots/{botId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotCategorizationConfigurationRequest {
    pub action: BotCategorizationAction,
}

/// Request body for `PUT …/bot-categorization/categories/{category}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotCategoryConfigurationRequest {
    pub action: BotCategoryAction,
}

/// The category-and-action pair echoed after setting a whole category's action.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotCategoryDetails {
    #[serde(default)]
    pub category: BotCategory,
    pub action: BotCategoryAction,
}

/// Response for setting the action on a single categorised bot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotCategorizationConfigurationResponse {
    #[serde(default)]
    pub data: Option<BotCategorizationDetails>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

/// Response for setting the action on a whole bot category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBotCategoryConfigurationResponse {
    #[serde(default)]
    pub data: Option<BotCategoryDetails>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// Monthly overages metrics (iter-72)
// ---------------------------------------------------------------------------

/// A single billing segment within a month's overage report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverageSegment {
    #[serde(default)]
    pub period_start: Option<String>,
    #[serde(default)]
    pub period_end: Option<String>,
    #[serde(default)]
    pub plan_type: Option<ShieldPlanType>,
    #[serde(default)]
    pub overage_allowance: i64,
    #[serde(default)]
    pub overage: i64,
}

/// Response for `GET /shield/metrics/overages/{shieldZoneId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShieldZoneMonthlyOveragesResult {
    #[serde(default)]
    pub shield_zone_id: i32,
    #[serde(default)]
    pub year: i32,
    #[serde(default)]
    pub month: i32,
    #[serde(default)]
    pub total_overage: i64,
    #[serde(default)]
    pub total_clean_requests: i64,
    #[serde(default)]
    pub segments: Option<Vec<OverageSegment>>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

// ---------------------------------------------------------------------------
// API Guardian metrics (iter-72)
// ---------------------------------------------------------------------------

/// Per-endpoint activity counters within the API Guardian 28-day breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGuardianEndpointActivity {
    #[serde(default)]
    pub blocked_requests: i32,
    #[serde(default)]
    pub logged_requests: i32,
    #[serde(default)]
    pub rate_limited: i32,
    #[serde(default)]
    pub requests_inspected: i32,
    #[serde(default)]
    pub failed_request_validation: i32,
    #[serde(default)]
    pub failed_authentication_enforcement: i32,
}

/// Aggregate API Guardian metrics for a Shield Zone (or a single endpoint).
///
/// The Shield Zone and per-endpoint metrics share the same shape, so a single
/// struct serves both `GET /shield/metrics/shield-zone/{id}/api-guardian` and
/// `.../api-guardian/endpoint/{endpointId}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGuardianMetrics {
    #[serde(default)]
    pub total_blocked_requests: i64,
    #[serde(default)]
    pub total_logged_requests: i64,
    #[serde(default)]
    pub total_rate_limited: i64,
    #[serde(default)]
    pub total_requests_inspected: i64,
    #[serde(default)]
    pub total_failed_request_validation: i64,
    #[serde(default)]
    pub total_failed_authentication_enforcement: i64,
    #[serde(default)]
    pub overview_past_twenty_eight_days: Option<HashMap<String, ApiGuardianEndpointActivity>>,
}

/// Response for the two API Guardian metrics endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetApiGuardianMetricsResponse {
    #[serde(default)]
    pub data: Option<ApiGuardianMetrics>,
    #[serde(default)]
    pub error: Option<GenericRequestResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(zone.waf_execution_mode, Some(WafExecutionMode::Block));
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
                severity_type: WafRuleSeverityType::Critical,
                transformation_types: None,
                value: Some("SELECT".to_string()),
                is_negated: None,
                is_regex_variable: None,
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

    /// Pins every operator name and number to the shield spec
    /// (`specs/shield.json`, `WafRuleOperatorType`).
    const OPERATOR_SPEC: &[(&str, u8)] = &[
        ("BEGINSWITH", 0),
        ("ENDSWITH", 1),
        ("CONTAINS", 2),
        ("CONTAINSWORD", 3),
        ("STRMATCH", 4),
        ("EQ", 5),
        ("GE", 6),
        ("GT", 7),
        ("LE", 8),
        ("LT", 9),
        ("WITHIN", 12),
        ("IPMATCH", 13),
        ("RX", 14),
        ("STREQ", 15),
        ("DETECTSQLI", 17),
        ("DETECTXSS", 18),
    ];

    #[test]
    fn waf_rule_operator_names_and_numbers_match_spec() {
        assert_eq!(WafRuleOperatorType::NAMES.len(), OPERATOR_SPEC.len());
        for (name, n) in OPERATOR_SPEC {
            let from_name: WafRuleOperatorType = name.parse().unwrap();
            let from_lower: WafRuleOperatorType = name.to_lowercase().parse().unwrap();
            let from_number: WafRuleOperatorType = n.to_string().parse().unwrap();
            assert_eq!(from_name, from_lower, "{name}");
            assert_eq!(from_name, from_number, "{name}");
            assert_eq!(from_name.to_repr(), *n, "{name}");
            assert_eq!(from_name.to_string(), *name);
            assert_eq!(serde_json::to_string(&from_name).unwrap(), n.to_string());
            let back: WafRuleOperatorType = serde_json::from_str(&n.to_string()).unwrap();
            assert_eq!(back, from_name);
        }
    }

    #[test]
    fn waf_rule_operator_rejects_old_labels_and_unknown_numbers() {
        for bad in [
            "notEq",
            "regex",
            "begins",
            "pm",
            "geolookup",
            "99",
            "10",
            "",
        ] {
            assert!(bad.parse::<WafRuleOperatorType>().is_err(), "{bad}");
        }
    }

    #[test]
    fn waf_rule_operator_unknown_value_round_trips() {
        let op: WafRuleOperatorType = serde_json::from_str("99").unwrap();
        assert_eq!(op, WafRuleOperatorType::Unknown(99));
        assert_eq!(op.to_string(), "Unknown(99)");
        assert_eq!(serde_json::to_string(&op).unwrap(), "99");
    }

    // -----------------------------------------------------------------------
    // Spec label tables (iter-90). Each enum is pinned twice: against a
    // hand-written (number, name) table below, and against
    // `x-enum-varnames` in `specs/shield.json`, so a spec refresh that moves a
    // label fails here instead of silently mislabelling a policy.
    // -----------------------------------------------------------------------

    fn shield_spec() -> serde_json::Value {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../specs/shield.json");
        let text = std::fs::read_to_string(&path).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    fn check_enum<T>(spec_name: &str, pinned: &[(u8, &str)], names: &[(T, &'static str)])
    where
        T: Copy
            + PartialEq
            + std::fmt::Debug
            + std::fmt::Display
            + std::str::FromStr<Err = String>
            + Serialize
            + serde::de::DeserializeOwned,
    {
        assert_eq!(names.len(), pinned.len(), "{spec_name}: table length");
        for (n, label) in pinned {
            let by_name: T = label.parse().unwrap();
            let by_lower: T = label.to_lowercase().parse().unwrap();
            let by_number: T = n.to_string().parse().unwrap();
            assert_eq!(by_name, by_lower, "{spec_name} {label}");
            assert_eq!(by_name, by_number, "{spec_name} {label}");
            assert_eq!(by_name.to_string(), *label, "{spec_name} display");
            assert_eq!(serde_json::to_string(&by_name).unwrap(), n.to_string());
            let back: T = serde_json::from_str(&n.to_string()).unwrap();
            assert_eq!(back, by_name);
            assert!(
                names.iter().any(|(v, l)| *v == by_name && l == label),
                "{spec_name}: NAMES lacks {n}={label}"
            );
        }
        // Undeclared numbers and names are rejected for user input.
        assert!("250".parse::<T>().is_err(), "{spec_name}: 250");
        assert!("".parse::<T>().is_err(), "{spec_name}: empty");

        // Same table, straight from the spec file.
        let spec = shield_spec();
        let schema = &spec["components"]["schemas"][spec_name];
        let numbers: Vec<u64> = schema["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{spec_name} missing in spec"))
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        let labels: Vec<&str> = schema["x-enum-varnames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let from_spec: Vec<(u64, &str)> = numbers.into_iter().zip(labels).collect();
        let from_pinned: Vec<(u64, &str)> =
            pinned.iter().map(|(n, l)| (u64::from(*n), *l)).collect();
        assert_eq!(
            from_pinned, from_spec,
            "{spec_name} differs from specs/shield.json"
        );
    }

    #[test]
    fn access_list_action_matches_spec() {
        check_enum::<AccessListAction>(
            "AccessListAction",
            &[
                (0, "None"),
                (1, "Allow"),
                (2, "Block"),
                (3, "Challenge"),
                (4, "Log"),
                (5, "Bypass"),
            ],
            AccessListAction::NAMES,
        );
        // The pre-iter-90 swap: wire 1 is Allow, wire 2 is Block.
        assert_eq!("allow".parse::<AccessListAction>().unwrap().to_repr(), 1);
        assert_eq!("block".parse::<AccessListAction>().unwrap().to_repr(), 2);
        for old in ["noaction", "logonly", "challengeinterstitial"] {
            assert!(old.parse::<AccessListAction>().is_err(), "{old}");
        }
    }

    #[test]
    fn access_list_type_matches_spec() {
        check_enum::<AccessListType>(
            "AccessListType",
            &[
                (0, "IP"),
                (1, "CIDR"),
                (2, "ASN"),
                (3, "Country"),
                (4, "Organization"),
                (5, "JA4"),
            ],
            AccessListType::NAMES,
        );
        for old in ["hostname", "useragent", "custom"] {
            assert!(old.parse::<AccessListType>().is_err(), "{old}");
        }
    }

    #[test]
    fn rate_limit_action_type_matches_spec() {
        check_enum::<RateLimitActionType>(
            "RatelimitRuleActionType",
            &[(1, "RateLimit"), (2, "Log"), (3, "Challenge")],
            RateLimitActionType::NAMES,
        );
        for old in ["block", "logonly"] {
            assert!(old.parse::<RateLimitActionType>().is_err(), "{old}");
        }
    }

    #[test]
    fn waf_rule_action_type_matches_spec() {
        check_enum::<WafRuleActionType>(
            "WafRuleActionType",
            &[
                (1, "Block"),
                (2, "Log"),
                (3, "Challenge"),
                (4, "Allow"),
                (5, "Bypass"),
            ],
            WafRuleActionType::NAMES,
        );
        for old in ["logonly", "challengeinterstitial"] {
            assert!(old.parse::<WafRuleActionType>().is_err(), "{old}");
        }
    }

    #[test]
    fn waf_rule_severity_matches_spec() {
        check_enum::<WafRuleSeverityType>(
            "WafRuleSeverityType",
            &[(0, "NOTICE"), (1, "WARNING"), (2, "CRITICAL")],
            WafRuleSeverityType::NAMES,
        );
        for old in ["low", "medium", "high"] {
            assert!(old.parse::<WafRuleSeverityType>().is_err(), "{old}");
        }
    }

    #[test]
    fn shield_plan_type_matches_spec() {
        check_enum::<ShieldPlanType>(
            "ShieldPlanType",
            &[
                (0, "Basic"),
                (1, "Advanced"),
                (2, "Business"),
                (3, "Enterprise"),
            ],
            ShieldPlanType::NAMES,
        );
        assert!("standard".parse::<ShieldPlanType>().is_err());
    }

    #[test]
    fn execution_modes_match_spec() {
        check_enum::<WafExecutionMode>(
            "WAFExecutionMode",
            &[(0, "Log"), (1, "Block")],
            WafExecutionMode::NAMES,
        );
        check_enum::<DdosExecutionMode>(
            "DDoSExecutionMode",
            &[(0, "Log"), (1, "Block")],
            DdosExecutionMode::NAMES,
        );
        check_enum::<BotDetectionExecutionMode>(
            "BotDetectionExecutionMode",
            &[(0, "LogOnly"), (1, "Challenge")],
            BotDetectionExecutionMode::NAMES,
        );
        for old in ["disabled", "enabled"] {
            assert!(old.parse::<WafExecutionMode>().is_err(), "{old}");
            assert!(old.parse::<DdosExecutionMode>().is_err(), "{old}");
            assert!(old.parse::<BotDetectionExecutionMode>().is_err(), "{old}");
        }
    }

    #[test]
    fn sensitivities_and_aggression_match_spec() {
        check_enum::<BotDetectionSensitivity>(
            "BotDetectionSensitivity",
            &[(0, "Off"), (1, "Low"), (2, "Medium"), (3, "High")],
            BotDetectionSensitivity::NAMES,
        );
        check_enum::<DdosShieldSensitivity>(
            "DDoSShieldSensitivity",
            &[
                (0, "Off"),
                (1, "Low"),
                (2, "Medium"),
                (3, "High"),
                (4, "Challenge"),
                (5, "ChallengeInteractive"),
            ],
            DdosShieldSensitivity::NAMES,
        );
        check_enum::<BrowserFingerprintAggression>(
            "BrowserFingerprintAggression",
            &[
                (0, "Unknown"),
                (1, "Low"),
                (2, "Medium"),
                (3, "High"),
                (4, "Custom"),
            ],
            BrowserFingerprintAggression::NAMES,
        );
        assert!("verylow".parse::<BrowserFingerprintAggression>().is_err());
        assert!("disabled".parse::<BotDetectionSensitivity>().is_err());
    }

    #[test]
    fn remaining_shield_enums_match_spec() {
        check_enum::<WafPayloadLimitAction>(
            "WAFPayloadLimitAction",
            &[(0, "Block"), (1, "Log"), (2, "Ignore")],
            WafPayloadLimitAction::NAMES,
        );
        check_enum::<ReviewActionType>(
            "ReviewActionType",
            &[(0, "Ignore"), (1, "LogOnly"), (2, "DisableRule")],
            ReviewActionType::NAMES,
        );
        check_enum::<UploadScanningScannerMode>(
            "UploadScanningScannerMode",
            &[(0, "Disabled"), (1, "Log"), (2, "Block")],
            UploadScanningScannerMode::NAMES,
        );
        check_enum::<RateLimitCounterKey>(
            "WafRatelimitCounterKeyType",
            &[
                (0, "IP"),
                (1, "Host"),
                (2, "Country"),
                (3, "City"),
                (4, "ASN"),
                (5, "Organization"),
                (6, "JA4"),
                (7, "IP_JA4"),
            ],
            RateLimitCounterKey::NAMES,
        );
        check_enum::<BotCategorizationAction>(
            "BotCategorizationAction",
            &[(0, "None"), (1, "Block"), (2, "Allow"), (3, "Ignore")],
            BotCategorizationAction::NAMES,
        );
        check_enum::<BotCategoryAction>(
            "BotCategoryAction",
            &[(0, "None"), (1, "Block"), (2, "Allow")],
            BotCategoryAction::NAMES,
        );
        // Category-wide actions have no Ignore.
        assert!("ignore".parse::<BotCategoryAction>().is_err());
        assert!("3".parse::<BotCategoryAction>().is_err());
        for old in ["pending", "approve", "reject"] {
            assert!(old.parse::<ReviewActionType>().is_err(), "{old}");
        }
        for old in ["global", "perip", "percountry"] {
            assert!(old.parse::<RateLimitCounterKey>().is_err(), "{old}");
        }
    }

    #[test]
    fn waf_operator_and_transformation_match_spec() {
        let operators: Vec<(u8, &str)> = OPERATOR_SPEC.iter().map(|(n, v)| (*v, *n)).collect();
        check_enum::<WafRuleOperatorType>(
            "WafRuleOperatorType",
            &operators,
            WafRuleOperatorType::NAMES,
        );
        check_enum::<WafRuleTransformationType>(
            "WafRuleTransformationType",
            &[
                (1, "CMDLINE"),
                (2, "COMPRESSWHITESPACE"),
                (3, "CSSDECODE"),
                (4, "HEXENCODE"),
                (5, "HTMLENTITYDECODE"),
                (6, "JSDECODE"),
                (7, "LENGTH"),
                (8, "LOWERCASE"),
                (9, "MD5"),
                (10, "NORMALIZEPATH"),
                (11, "NORMALISEPATH"),
                (12, "NORMALIZEPATHWIN"),
                (13, "NORMALISEPATHWIN"),
                (14, "REMOVECOMMENTS"),
                (15, "REMOVENULLS"),
                (16, "REMOVEWHITESPACE"),
                (17, "REPLACECOMMENTS"),
                (18, "SHA1"),
                (19, "URLDECODE"),
                (20, "URLDECODEUNI"),
                (21, "UTF8TOUNICODE"),
            ],
            WafRuleTransformationType::NAMES,
        );
    }

    #[test]
    fn unknown_values_round_trip_for_relabelled_enums() {
        let a: AccessListAction = serde_json::from_str("42").unwrap();
        assert_eq!(a, AccessListAction::Unknown(42));
        assert_eq!(a.to_string(), "Unknown(42)");
        assert_eq!(serde_json::to_string(&a).unwrap(), "42");
        let t: AccessListType = serde_json::from_str("42").unwrap();
        assert_eq!(serde_json::to_string(&t).unwrap(), "42");
    }

    #[test]
    fn parse_shield_error_nested_envelope() {
        // Real API shape: {"error": {"statusCode": 404, "errorKey": "...", "message": "..."}, "data": null}
        let body = br#"{"error":{"statusCode":404,"errorKey":"zone.config.not_found","message":"No api-guardian config for zone 42"},"data":null}"#;
        let err = parse_shield_error(body).expect("should parse nested envelope");
        let msg = err.to_string();
        assert!(msg.contains("404"), "expected status 404 in: {msg}");
        assert!(
            msg.contains("zone.config.not_found"),
            "expected errorKey in: {msg}"
        );
        assert!(
            msg.contains("No api-guardian config for zone 42"),
            "expected message in: {msg}"
        );
    }

    #[test]
    fn parse_shield_error_rfc7807_fallback() {
        let body = br#"{"status":403,"title":"Forbidden","detail":"Access denied"}"#;
        let err = parse_shield_error(body).expect("should parse RFC 7807");
        let msg = err.to_string();
        assert!(msg.contains("403"), "expected status 403 in: {msg}");
        assert!(msg.contains("Forbidden"), "expected title in: {msg}");
    }

    #[test]
    fn parse_shield_error_does_not_panic_on_unknown_body() {
        // Envelope requires the "error" key and ProblemDetails requires at
        // least one RFC 7807 field; this body has neither, so the parser
        // returns None and the caller can fall back to status + raw body.
        let body = br#"{"something":"unexpected"}"#;
        assert!(parse_shield_error(body).is_none());
    }

    #[test]
    fn parse_shield_error_recognises_error_response_envelope() {
        // event-logs shape: top-level body carries an `errorResponse` field.
        let body = br#"{"logs":null,"hasMoreData":false,"continuationToken":null,"startToken":null,"errorResponse":{"statusCode":401,"success":false,"message":"You can only view the past 3 days (72 hours) of Event Logs.","errorKey":"invalid_datetime_window.event_logs"}}"#;
        let err = parse_shield_error(body).expect("should parse errorResponse envelope");
        let msg = err.to_string();
        assert_eq!(
            msg,
            "Shield API error 401 (invalid_datetime_window.event_logs): You can only view the past 3 days (72 hours) of Event Logs."
        );
    }

    #[test]
    fn parse_shield_error_ignores_null_error_response_field() {
        // Happy-path success bodies on these endpoints also carry an
        // `errorResponse` field — present but null. Must not be parsed as an error.
        let body = br#"{"logs":[],"errorResponse":null}"#;
        assert!(parse_shield_error(body).is_none());
    }

    #[test]
    fn parse_shield_error_ignores_empty_error_response_object() {
        // If `errorResponse` is present but carries no message/key/success-false,
        // we must not treat the body as an error.
        let body = br#"{"errorResponse":{}}"#;
        assert!(parse_shield_error(body).is_none());
    }

    #[test]
    fn parse_shield_2xx_envelope_error_recognises_error_response_branch() {
        // 2xx body that nonetheless carries an `errorResponse` with success=false
        // and a human-readable signal must be surfaced as an error.
        let body = br#"{"logs":null,"errorResponse":{"statusCode":202,"success":false,"message":"Plan tier required.","errorKey":"plan.feature_gated"}}"#;
        let err = parse_shield_2xx_envelope_error(body)
            .expect("should detect errorResponse branch on 2xx body");
        assert_eq!(
            err.to_string(),
            "Shield API error 202 (plan.feature_gated): Plan tier required."
        );
    }

    #[test]
    fn parse_shield_2xx_envelope_error_ignores_empty_error_response() {
        // Happy-path 2xx body where `errorResponse` deserialises but carries
        // no error signal must NOT be treated as an error.
        let body = br#"{"errorResponse":{}}"#;
        assert!(parse_shield_2xx_envelope_error(body).is_none());
    }

    #[test]
    fn shield_error_response_envelope_display_paren_format() {
        let env = ShieldErrorResponseEnvelope {
            error_response: ShieldApiErrorInner {
                status_code: Some(401),
                success: Some(false),
                error_key: Some("invalid_datetime_window.event_logs".to_owned()),
                message: Some(
                    "You can only view the past 3 days (72 hours) of Event Logs.".to_owned(),
                ),
            },
        };
        let s = env.to_string();
        assert_eq!(
            s,
            "Shield API error 401 (invalid_datetime_window.event_logs): You can only view the past 3 days (72 hours) of Event Logs."
        );
    }

    #[test]
    fn shield_error_envelope_display_status_and_key() {
        let env = ShieldApiErrorEnvelope {
            error: ShieldApiErrorInner {
                status_code: Some(404),
                success: None,
                error_key: Some("zone.config.not_found".to_owned()),
                message: Some("No api-guardian config for zone 42".to_owned()),
            },
        };
        let s = env.to_string();
        assert_eq!(
            s,
            "Shield API error 404: zone.config.not_found: No api-guardian config for zone 42"
        );
    }
}
