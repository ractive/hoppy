//! Hand-written bunny.net CDN Logging API client.
//!
//! Retrieves raw request logs (up to 3 days retention) for a pull zone via two
//! endpoints against `https://logging.bunnycdn.com`:
//!
//! - **v2** ([`LoggingClient::query_logs`]) — structured JSON with rich
//!   filtering and pagination.
//! - **v1** ([`LoggingClient::stream_legacy_logs`]) — legacy pipe-delimited raw
//!   text, streamed chunk-by-chunk so large logs never buffer in memory.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bunny_net_api::logging::{LoggingClient, LogQueryParams};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let client = LoggingClient::new("YOUR_API_KEY");
//! let params = LogQueryParams {
//!     status: Some("5xx".into()),
//!     limit: Some(50),
//!     ..Default::default()
//! };
//! let page = client.query_logs(12345, &params).await?;
//! println!("{} entries", page.data.len());
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod types;

pub use client::LoggingClient;
pub use types::{
    ErrorBody, ErrorResponse, LegacyLogParams, LogEntry, LogQueryParams, LogQueryResponse,
    PaginationInfo, QuerySummary,
};
