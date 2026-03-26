//! Hand-written bunny.net Stream (Video) API client.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use bunny_api_stream::{StreamClient, CreateVideo};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let client = StreamClient::new("YOUR_API_KEY");
//!
//! // Create a video record, then upload the bytes
//! let video = client.create_video(12345, &CreateVideo::new("My Video")).await?;
//! client
//!     .upload_video(12345, &video.guid, std::fs::read("video.mp4")?)
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod types;

// Flatten the most-used items into the crate root for ergonomic imports.
pub use client::StreamClient;
pub use types::{
    Caption, Collection, CreateCollection, CreateVideo, FetchVideo, PaginatedList, StatusMessage,
    UpdateCollection, UpdateVideo, Video, VideoStatistics, VideoStatus,
};
