//! `bunny-net-api` — hand-written Rust client for bunny.net APIs.
//!
//! Services are gated behind Cargo features. All are enabled by default.
//!
//! # Feature flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `core` | Pull zones, DNS, storage zones, video libraries, statistics, billing |
//! | `compute` | Edge scripting |
//! | `containers` | Magic containers |
//! | `database` | libSQL managed databases |
//! | `logging` | CDN access log retrieval (v1 legacy + v2 structured) |
//! | `origin-errors` | CDN origin error log retrieval |
//! | `recording` | HTTP response recording helper (shared by other modules) |
//! | `shield` | WAF, DDoS protection, rate limiting, bot detection |
//! | `storage` | Edge storage file operations |
//! | `stream` | Video streaming |

#[cfg(feature = "recording")]
pub mod recording;

/// Global `--dry-run` interception, shared by every domain client.
#[cfg(feature = "recording")]
pub mod dry_run;

#[cfg(feature = "core")]
pub mod core;

#[cfg(feature = "compute")]
pub mod compute;

#[cfg(feature = "containers")]
pub mod containers;

#[cfg(feature = "database")]
pub mod database;

#[cfg(feature = "logging")]
pub mod logging;

#[cfg(feature = "origin-errors")]
pub mod origin_errors;

#[cfg(feature = "shield")]
pub mod shield;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "stream")]
pub mod stream;
