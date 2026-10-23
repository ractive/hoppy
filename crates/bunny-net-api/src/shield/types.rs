use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

forward_compat_repr_enum! {
    /// WAF execution mode for a Shield Zone.
    ///
    /// Values from the spec: 0 = Disabled, 1 = Enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafExecutionMode: u8 {
        Disabled = 0,
        Enabled = 1,
    }
}

impl std::fmt::Display for WafExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WafExecutionMode::Disabled => write!(f, "Disabled"),
            WafExecutionMode::Enabled => write!(f, "Enabled"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Action taken when WAF payload size limit is exceeded.
    ///
    /// Values from the spec: 0 = Allow, 1 = Block, 2 = Ignore.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafPayloadLimitAction: u8 {
        Allow = 0,
        Block = 1,
        Ignore = 2,
    }
}

impl std::fmt::Display for WafPayloadLimitAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WafPayloadLimitAction::Allow => write!(f, "Allow"),
            WafPayloadLimitAction::Block => write!(f, "Block"),
            WafPayloadLimitAction::Ignore => write!(f, "Ignore"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Shield plan tier.
    ///
    /// Values from the spec: 0 = Basic, 1 = Standard, 2 = Advanced, 3 = Enterprise.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ShieldPlanType: u8 {
        Basic = 0,
        Standard = 1,
        Advanced = 2,
        Enterprise = 3,
    }
}

impl std::fmt::Display for ShieldPlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShieldPlanType::Basic => write!(f, "Basic"),
            ShieldPlanType::Standard => write!(f, "Standard"),
            ShieldPlanType::Advanced => write!(f, "Advanced"),
            ShieldPlanType::Enterprise => write!(f, "Enterprise"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// DDoS protection execution mode.
    ///
    /// Values from the spec: 0 = Disabled, 1 = Enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DdosExecutionMode: u8 {
        Disabled = 0,
        Enabled = 1,
    }
}

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

impl std::fmt::Display for DdosShieldSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DdosShieldSensitivity::Off => write!(f, "Off"),
            DdosShieldSensitivity::Low => write!(f, "Low"),
            DdosShieldSensitivity::Medium => write!(f, "Medium"),
            DdosShieldSensitivity::High => write!(f, "High"),
            DdosShieldSensitivity::Challenge => write!(f, "Challenge"),
            DdosShieldSensitivity::ChallengeInteractive => write!(f, "ChallengeInteractive"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Action to take when an access list entry matches a request.
    ///
    /// Values from the spec: 0 = NoAction, 1 = Block, 2 = Allow, 3 = LogOnly,
    /// 4 = Challenge, 5 = ChallengeInterstitial.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccessListAction: u8 {
        NoAction = 0,
        Block = 1,
        Allow = 2,
        LogOnly = 3,
        Challenge = 4,
        ChallengeInterstitial = 5,
    }
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
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Type of access list (IP, Country, ASN, etc.).
    ///
    /// Values from the spec: 0 = Ip, 1 = Country, 2 = Asn, 3 = Hostname, 4 = UserAgent, 5 = Custom.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AccessListType: u8 {
        Ip = 0,
        Country = 1,
        Asn = 2,
        Hostname = 3,
        UserAgent = 4,
        Custom = 5,
    }
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
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Action taken when a rate limit rule is breached.
    ///
    /// Values from the spec: 1 = Block, 2 = LogOnly, 3 = Challenge.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitActionType: u8 {
        Block = 1,
        LogOnly = 2,
        Challenge = 3,
    }
}

impl std::fmt::Display for RateLimitActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitActionType::Block => write!(f, "Block"),
            RateLimitActionType::LogOnly => write!(f, "LogOnly"),
            RateLimitActionType::Challenge => write!(f, "Challenge"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

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
    /// What key to count requests by for rate limiting.
    ///
    /// Values from the spec: 0 = Global, 1 = PerIp, 2 = PerCountry, 3 = PerAsn,
    /// 4 = PerHeader, 5 = PerCookie, 6 = PerQuery, 7 = PerFingerprint.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum RateLimitCounterKey: u8 {
        Global = 0,
        PerIp = 1,
        PerCountry = 2,
        PerAsn = 3,
        PerHeader = 4,
        PerCookie = 5,
        PerQuery = 6,
        PerFingerprint = 7,
    }
}

impl std::fmt::Display for RateLimitCounterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitCounterKey::Global => write!(f, "Global"),
            RateLimitCounterKey::PerIp => write!(f, "PerIp"),
            RateLimitCounterKey::PerCountry => write!(f, "PerCountry"),
            RateLimitCounterKey::PerAsn => write!(f, "PerAsn"),
            RateLimitCounterKey::PerHeader => write!(f, "PerHeader"),
            RateLimitCounterKey::PerCookie => write!(f, "PerCookie"),
            RateLimitCounterKey::PerQuery => write!(f, "PerQuery"),
            RateLimitCounterKey::PerFingerprint => write!(f, "PerFingerprint"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Action taken by a custom WAF rule.
    ///
    /// Values from the spec: 1 = Block, 2 = LogOnly, 3 = Challenge,
    /// 4 = ChallengeInterstitial, 5 = Allow.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleActionType: u8 {
        Block = 1,
        LogOnly = 2,
        Challenge = 3,
        ChallengeInterstitial = 4,
        Allow = 5,
    }
}

impl std::fmt::Display for WafRuleActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WafRuleActionType::Block => write!(f, "Block"),
            WafRuleActionType::LogOnly => write!(f, "LogOnly"),
            WafRuleActionType::Challenge => write!(f, "Challenge"),
            WafRuleActionType::ChallengeInterstitial => write!(f, "ChallengeInterstitial"),
            WafRuleActionType::Allow => write!(f, "Allow"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

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

impl WafRuleOperatorType {
    /// Spec name for every declared operator. Single source of truth for
    /// [`Display`](std::fmt::Display), [`FromStr`](std::str::FromStr) and the
    /// CLI help text.
    pub const NAMES: &'static [(WafRuleOperatorType, &'static str)] = &[
        (Self::BeginsWith, "BEGINSWITH"),
        (Self::EndsWith, "ENDSWITH"),
        (Self::Contains, "CONTAINS"),
        (Self::ContainsWord, "CONTAINSWORD"),
        (Self::StrMatch, "STRMATCH"),
        (Self::Eq, "EQ"),
        (Self::Ge, "GE"),
        (Self::Gt, "GT"),
        (Self::Le, "LE"),
        (Self::Lt, "LT"),
        (Self::Within, "WITHIN"),
        (Self::IpMatch, "IPMATCH"),
        (Self::Rx, "RX"),
        (Self::StrEq, "STREQ"),
        (Self::DetectSqli, "DETECTSQLI"),
        (Self::DetectXss, "DETECTXSS"),
    ];

    /// The spec name (`"IPMATCH"`), or `None` for [`Self::Unknown`].
    #[must_use]
    pub fn name(self) -> Option<&'static str> {
        Self::NAMES
            .iter()
            .find_map(|(op, name)| (*op == self).then_some(*name))
    }

    /// `"0=BEGINSWITH, 1=ENDSWITH, ..."`, for help and error text.
    #[must_use]
    pub fn describe_all() -> String {
        Self::NAMES
            .iter()
            .map(|(op, name)| format!("{}={name}", op.to_repr()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl std::fmt::Display for WafRuleOperatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name() {
            Some(name) => f.write_str(name),
            None => write!(f, "Unknown({})", self.to_repr()),
        }
    }
}

/// Parses a spec name (case-insensitive, for example `beginswith` or `rx`) or
/// a declared wire number (`13`). Old pre-iter-87 labels are not accepted.
impl std::str::FromStr for WafRuleOperatorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        let found = if let Ok(n) = t.parse::<u8>() {
            Self::try_from_repr(n)
        } else {
            Self::NAMES
                .iter()
                .find_map(|(op, name)| name.eq_ignore_ascii_case(t).then_some(*op))
        };
        found.ok_or_else(|| {
            format!(
                "invalid operator '{s}': expected a name or number ({})",
                Self::describe_all()
            )
        })
    }
}

forward_compat_repr_enum! {
    /// Severity level of a custom WAF rule.
    ///
    /// Values from the spec: 0 = Low, 1 = Medium, 2 = High.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleSeverityType: u8 {
        Low = 0,
        Medium = 1,
        High = 2,
    }
}

forward_compat_repr_enum! {
    /// Transformation applied to the variable value before matching.
    ///
    /// Values from the spec: 1–21.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WafRuleTransformationType: u8 {
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
}

forward_compat_repr_enum! {
    /// Bot detection execution mode.
    ///
    /// Values from the spec: 0 = Disabled, 1 = Enabled.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BotDetectionExecutionMode: u8 {
        Disabled = 0,
        Enabled = 1,
    }
}

impl std::fmt::Display for BotDetectionExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotDetectionExecutionMode::Disabled => write!(f, "Disabled"),
            BotDetectionExecutionMode::Enabled => write!(f, "Enabled"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Sensitivity level for bot detection signals.
    ///
    /// Values from the spec: 0 = Disabled, 1 = Low, 2 = Medium, 3 = High.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BotDetectionSensitivity: u8 {
        Disabled = 0,
        Low = 1,
        Medium = 2,
        High = 3,
    }
}

impl std::fmt::Display for BotDetectionSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BotDetectionSensitivity::Disabled => write!(f, "Disabled"),
            BotDetectionSensitivity::Low => write!(f, "Low"),
            BotDetectionSensitivity::Medium => write!(f, "Medium"),
            BotDetectionSensitivity::High => write!(f, "High"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

forward_compat_repr_enum! {
    /// Aggressiveness of browser fingerprint challenge injection.
    ///
    /// Values from the spec: 0 = Disabled, 1 = VeryLow, 2 = Low, 3 = Medium, 4 = High.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BrowserFingerprintAggression: u8 {
        Disabled = 0,
        VeryLow = 1,
        Low = 2,
        Medium = 3,
        High = 4,
    }
}

impl std::fmt::Display for BrowserFingerprintAggression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserFingerprintAggression::Disabled => write!(f, "Disabled"),
            BrowserFingerprintAggression::VeryLow => write!(f, "VeryLow"),
            BrowserFingerprintAggression::Low => write!(f, "Low"),
            BrowserFingerprintAggression::Medium => write!(f, "Medium"),
            BrowserFingerprintAggression::High => write!(f, "High"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
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
    /// Scanner mode for upload scanning (antivirus/CSAM).
    ///
    /// Values from the spec: 0 = Disabled, 1 = LogOnly, 2 = Block.
    /// The spec provides only integer values 0/1/2 without explicit names;
    /// these names reflect Bunny's actual semantics.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UploadScanningScannerMode: u8 {
        Disabled = 0,
        LogOnly = 1,
        Block = 2,
    }
}

impl std::fmt::Display for UploadScanningScannerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadScanningScannerMode::Disabled => write!(f, "Disabled"),
            UploadScanningScannerMode::LogOnly => write!(f, "LogOnly"),
            UploadScanningScannerMode::Block => write!(f, "Block"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

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
    /// Action type for reviewing a triggered WAF rule.
    ///
    /// Values from the spec: 0 = Pending, 1 = Approve, 2 = Reject.
    /// The spec provides only integer values 0/1/2 without explicit names.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ReviewActionType: u8 {
        Pending = 0,
        Approve = 1,
        Reject = 2,
    }
}

impl std::fmt::Display for ReviewActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewActionType::Pending => write!(f, "Pending"),
            ReviewActionType::Approve => write!(f, "Approve"),
            ReviewActionType::Reject => write!(f, "Reject"),
            Self::Unknown(n) => write!(f, "Unknown({n})"),
        }
    }
}

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

    #[test]
    fn upload_scanning_scanner_mode_round_trips() {
        // Disabled = 0
        let json = serde_json::to_string(&UploadScanningScannerMode::Disabled).unwrap();
        assert_eq!(json, "0");
        let decoded: UploadScanningScannerMode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, UploadScanningScannerMode::Disabled);

        // LogOnly = 1
        let json = serde_json::to_string(&UploadScanningScannerMode::LogOnly).unwrap();
        assert_eq!(json, "1");
        let decoded: UploadScanningScannerMode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, UploadScanningScannerMode::LogOnly);

        // Block = 2
        let json = serde_json::to_string(&UploadScanningScannerMode::Block).unwrap();
        assert_eq!(json, "2");
        let decoded: UploadScanningScannerMode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, UploadScanningScannerMode::Block);
    }

    #[test]
    fn review_action_type_round_trips() {
        // Pending = 0
        let json = serde_json::to_string(&ReviewActionType::Pending).unwrap();
        assert_eq!(json, "0");
        let decoded: ReviewActionType = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ReviewActionType::Pending);

        // Approve = 1
        let json = serde_json::to_string(&ReviewActionType::Approve).unwrap();
        assert_eq!(json, "1");
        let decoded: ReviewActionType = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ReviewActionType::Approve);

        // Reject = 2
        let json = serde_json::to_string(&ReviewActionType::Reject).unwrap();
        assert_eq!(json, "2");
        let decoded: ReviewActionType = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ReviewActionType::Reject);
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
