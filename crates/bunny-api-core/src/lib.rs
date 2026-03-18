//! `bunny-api-core` — hand-written bunny.net API client.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bunny_api_core::{CoreClient, CreatePullZone};
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
pub mod types;

// Flat re-exports for ergonomic `use bunny_api_core::CoreClient` imports.
pub use client::CoreClient;
pub use types::{
    AddDnsRecord, ApiError, CreateDnsZone, CreatePullZone, CreateStorageZone, DnsRecord,
    DnsRecordType, DnsZone, HostnameInfo, OriginType, PaginatedList, PullZone, PullZoneType,
    PurgeCache, StorageZone, UpdateDnsRecord, UpdateDnsZone, UpdatePullZone, UpdateStorageZone,
};
