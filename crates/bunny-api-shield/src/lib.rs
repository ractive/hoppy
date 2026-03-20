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
    // Response types
    AccessListDetails,
    AccessListType,
    AccessListsDetailsResponse,
    BotDetectionConfigurationResponse,
    BotDetectionConfigurationState,
    BotDetectionExecutionMode,
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
    RateLimitRule,
    RateLimitRuleConfiguration,
    RateLimitTimeframe,
    RequestIntegrityConfiguration,
    // Metrics types
    ShieldAccessListSummary,
    ShieldBotDetectionSummary,
    ShieldDdosSummary,
    ShieldMetricsData,
    ShieldMetricsResponse,
    ShieldOverviewSummary,
    ShieldPlanType,
    ShieldRatelimitSummary,
    ShieldUploadScanningSummary,
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
    WafChainedRuleCondition,
    WafExecutionMode,
    WafPayloadLimitAction,
    WafProfileMinimal,
    WafRuleActionType,
    WafRuleConfiguration,
    WafRuleOperatorType,
    WafRuleSeverityType,
    WafRuleTransformationType,
};
