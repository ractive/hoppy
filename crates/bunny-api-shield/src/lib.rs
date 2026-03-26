//! Hand-written bunny.net Shield API client.
//!
//! Covers the main Shield resources: Shield Zones, custom WAF rules,
//! rate limit rules, access lists, and bot detection configuration.
//!
//! # Quick start
//!
//! ```no_run
//! use bunny_api_shield::ShieldClient;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = ShieldClient::new("your-api-key");
//! let zone = client.get_shield_zone(12345).await?;
//! println!("Shield Zone: {:?}", zone.shield_zone_id);
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod types;

pub use client::ShieldClient;
pub use types::{
    // Enums
    AccessListAction,
    // Metrics types
    AccessListDetailCategory,
    // Response types
    AccessListDetails,
    AccessListType,
    AccessListsDetailsResponse,
    BlockedLoggedChallengedMetrics,
    BotDetectionConfigurationResponse,
    BotDetectionConfigurationState,
    BotDetectionDetailCategory,
    BotDetectionDetailMetrics,
    BotDetectionExecutionMode,
    BotDetectionMetricsData,
    BotDetectionSensitivity,
    BrowserFingerprintAggression,
    BrowserFingerprintConfiguration,
    // Request types
    CreateCustomAccessList,
    CreateCustomWafRule,
    CreateRateLimitRule,
    CreateShieldZoneRequest,
    CustomAccessList,
    CustomAccessListResponse,
    CustomWafRule,
    DdosDetailCategory,
    DdosDetailMetrics,
    DdosExecutionMode,
    DdosShieldSensitivity,
    GenericRequestResponse,
    GetCustomWafRulesResponse,
    GetRateLimitRulesResponse,
    GetShieldZoneResponse,
    GetShieldZonesResponse,
    IpAddressConfiguration,
    PaginationResponse,
    ProblemDetails,
    RateLimitActionType,
    RateLimitBlockDuration,
    RateLimitCounterKey,
    RateLimitDetailCategory,
    RateLimitDetailMetrics,
    RateLimitMetricsEntry,
    RateLimitRule,
    RateLimitRuleConfiguration,
    RateLimitTimeframe,
    RequestIntegrityConfiguration,
    ShieldAccessListSummary,
    ShieldBotDetectionMetricsResponse,
    ShieldBotDetectionSummary,
    ShieldDdosSummary,
    ShieldDetailedMetricsData,
    ShieldDetailedMetricsResponse,
    ShieldMetricsData,
    ShieldMetricsResponse,
    ShieldOverviewSummary,
    ShieldPlanType,
    ShieldRateLimitMetricsResponse,
    ShieldRateLimitsMetricsResponse,
    ShieldRatelimitSummary,
    ShieldUploadScanningMetricsResponse,
    ShieldUploadScanningSummary,
    ShieldWafRuleMetricsResponse,
    ShieldWafSummary,
    ShieldZoneRequest,
    ShieldZoneResponse,
    UpdateAccessListConfiguration,
    UpdateBotDetection,
    UpdateBotDetectionResponse,
    UpdateCustomAccessList,
    UpdateCustomWafRule,
    UpdateRateLimitRule,
    UpdateShieldZoneRequest,
    UploadScanningDetailCategory,
    UploadScanningDetailMetrics,
    UploadScanningMetricsData,
    WafChainedRuleCondition,
    WafDetailCategory,
    WafExecutionMode,
    WafPayloadLimitAction,
    WafProfileMinimal,
    WafRuleActionType,
    WafRuleConfiguration,
    WafRuleDetailMetrics,
    WafRuleMetricsData,
    WafRuleOperatorType,
    WafRuleSeverityType,
    WafRuleTransformationType,
};
