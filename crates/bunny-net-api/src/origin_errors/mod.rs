//! Hand-written bunny.net Origin Errors API client.
//!
//! Retrieves CDN origin error logs for a pull zone on a given date via
//! `GET /{pullZoneId}/{dateTime}` against `https://cdn-origin-logging.bunny.net`.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bunny_net_api::origin_errors::OriginErrorsClient;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let client = OriginErrorsClient::new("YOUR_API_KEY");
//! let resp = client.get_origin_errors(12345, "10-29-2025").await?;
//! println!("{} origin errors", resp.logs.len());
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod types;

pub use client::OriginErrorsClient;
pub use types::{LogResponse, OriginErrorEntry, OriginErrorLabels};
