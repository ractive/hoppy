//! Hand-written bunny.net Database (libSQL) API client.
//!
//! Covers the control-plane REST API at `https://api.bunny.net/database`
//! and a small data-plane convenience (`ping`) against the per-database
//! libSQL HTTP endpoint.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bunny_net_api::database::{DatabaseClient, CreateDatabasePayload};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let client = DatabaseClient::new("YOUR_API_KEY");
//! let db = client
//!     .create_database(&CreateDatabasePayload::new("my-app", "group_01H..."))
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod types;

pub use client::DatabaseClient;
pub use types::{
    Authorization, CreateDatabaseGroupPayload, CreateDatabasePayload, CreateDatabaseV2Payload,
    Database, DatabaseGroup, GenerateTokenDatabasePayload, PingResult,
};
