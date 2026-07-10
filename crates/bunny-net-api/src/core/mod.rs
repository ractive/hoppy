//! `bunny-net-api-core` — hand-written bunny.net API client.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bunny_net_api::core::{CoreClient, CreatePullZone};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = CoreClient::new("your-api-key");
//!
//!     // List the first page of pull zones
//!     let zones = client.list_pull_zones(None, None, None).await?;
//!     println!("Total zones: {}", zones.total_items);
//!
//!     // Create a new pull zone
//!     let zone = client
//!         .create_pull_zone(
//!             &CreatePullZone::new("my-zone", "https://origin.example.com"),
//!         )
//!         .await?;
//!     println!("Created zone id={}", zone.id);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod serde_helpers;
pub mod types;

// Flat re-exports for ergonomic `use bunny_net_api::core::CoreClient` imports.
pub use client::CoreClient;
pub use types::{
    AccountStatistics, AddDnsRecord, AddOrUpdateEdgeRule, ApiError, ApiKey, AuditLogOrder,
    BillingDetails, BillingSummaryEntry, Country, CreateDnsZone, CreatePullZone, CreateStorageZone,
    CreateVideoLibrary, DnsDiscoveredRecord, DnsDiscoveredRecordType, DnsImportResult, DnsRecord,
    DnsRecordScanResult, DnsRecordScanTrigger, DnsRecordType, DnsScanJobStatus, DnsSecDsRecord,
    DnsZone, DnsZoneStatistics, EdgeRule, EdgeRuleActionType, EdgeRuleExtraAction, EdgeRuleTrigger,
    EdgeScriptExecutionPhase, ExternalDnsCertificateRecord, HostnameInfo, LogAnonymizationType,
    MatchingType, OptimizerStatistics, OptimizerWatermarkPosition, OriginShieldQueueStatistics,
    OriginType, PaginatedList, PaymentRequest, PermaCacheType, PreloadingScreenTheme, PullZone,
    PullZoneLogForwarderProtocolType, PullZonePrivateKeyType, PullZoneTierType, PullZoneType,
    PurgeCache, Region, SafeHopStatistics, SearchResultItem, SearchResults, StatisticsQuery,
    StickySessionType, StorageZone, StorageZoneStatistics, TriggerDnsRecordScan, TriggerType,
    UpdateDnsRecord, UpdateDnsZone, UpdatePullZone, UpdateStorageZone, UpdateVideoLibrary,
    UserAuditLog, UserAuditLogList, UserAuditQuery, VideoLanguage, VideoLibrary,
    VideoLibraryDrmStatistics, VideoLibraryTranscribingStatistics,
};
