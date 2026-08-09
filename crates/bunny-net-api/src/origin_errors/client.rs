//! HTTP client for the bunny.net Origin Errors API.
//!
//! Covers the single endpoint `GET /{pullZoneId}/{dateTime}` against
//! `https://cdn-origin-logging.bunny.net`, returning origin error logs for a
//! pull zone on a given date. Authenticates with the `AccessKey` header.

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::{Client, StatusCode};

use crate::recording::{capture_request, maybe_record_response};

use super::types::LogResponse;

/// Production base URL for the Origin Errors API.
const BASE_URL: &str = "https://cdn-origin-logging.bunny.net";

/// Client for the bunny.net Origin Errors API.
///
/// Authenticates via the `AccessKey` header. Construct with [`OriginErrorsClient::new`].
pub struct OriginErrorsClient {
    http: Client,
    base_url: String,
    api_key: String,
    debug: bool,
    debug_reveal_secrets: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl OriginErrorsClient {
    /// Create a new client with the given API key, using the production base URL.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, BASE_URL)
    }

    /// Create a client pointing at a custom base URL (useful for tests / staging).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            debug: false,
            debug_reveal_secrets: false,
            record_dir: None,
            last_request: Mutex::new(None),
        }
    }

    /// Enable or disable debug logging of HTTP method and URL to stderr.
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Reveal secret-shaped fields in `--debug` body output instead of
    /// redacting them.
    #[must_use]
    pub fn with_debug_reveal_secrets(mut self, reveal: bool) -> Self {
        self.debug_reveal_secrets = reveal;
        self
    }

    /// Enable recording API responses to files in the given directory.
    #[must_use]
    pub fn with_record(mut self, dir: impl Into<PathBuf>) -> Self {
        self.record_dir = Some(dir.into());
        self
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// Validate a `MM-dd-yyyy` date string as required by the API path.
    ///
    /// Rejecting a malformed date up front avoids issuing a request that the
    /// API would bounce, and keeps a caller-controlled value from injecting
    /// extra path segments (e.g. a `/`).
    fn validate_date(date: &str) -> Result<()> {
        let parts: Vec<&str> = date.split('-').collect();
        let valid = parts.len() == 3
            && parts[0].len() == 2
            && parts[1].len() == 2
            && parts[2].len() == 4
            && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()));
        if !valid {
            bail!("invalid date {date:?}: expected MM-DD-YYYY (e.g. 10-29-2025)");
        }
        Ok(())
    }

    async fn execute(&self, rb: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let req = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", req.method(), req.url());
        }
        if self.record_dir.is_some() {
            capture_request(&self.last_request, req.method().as_ref(), req.url().path());
        }
        self.http.execute(req).await.context("request failed")
    }

    async fn read_body(&self, resp: reqwest::Response) -> Result<(StatusCode, bytes::Bytes)> {
        let status = resp.status();
        let bytes = resp.bytes().await.context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!(
                "<<< {}",
                crate::recording::debug::format_debug_body(&bytes, self.debug_reveal_secrets)
            );
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            "origin-errors",
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    /// Get origin error logs for a pull zone on a given date.
    ///
    /// `date` must be formatted `MM-DD-YYYY` (e.g. `"10-29-2025"`); it is
    /// validated locally before the request is sent.
    pub async fn get_origin_errors(&self, pull_zone_id: i64, date: &str) -> Result<LogResponse> {
        Self::validate_date(date)?;
        let req = self
            .http
            .get(self.url(&format!("/{pull_zone_id}/{date}")))
            .header("AccessKey", &self.api_key);
        let resp = self.execute(req).await?;
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            serde_json::from_slice(&bytes).context("deserializing origin error response")
        } else {
            Err(anyhow!("Origin Errors API returned HTTP {status}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new_uses_default_base_url() {
        let client = OriginErrorsClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn with_base_url_trims_trailing_slash() {
        let client = OriginErrorsClient::with_base_url("k", "http://localhost:9000/");
        assert_eq!(
            client.url("/1/10-29-2025"),
            "http://localhost:9000/1/10-29-2025"
        );
    }

    #[test]
    fn validate_date_accepts_well_formed() {
        assert!(OriginErrorsClient::validate_date("10-29-2025").is_ok());
        assert!(OriginErrorsClient::validate_date("01-01-2000").is_ok());
    }

    #[test]
    fn validate_date_rejects_bad_shapes() {
        assert!(OriginErrorsClient::validate_date("2025-10-29").is_err());
        assert!(OriginErrorsClient::validate_date("1-1-2025").is_err());
        assert!(OriginErrorsClient::validate_date("10/29/2025").is_err());
        assert!(OriginErrorsClient::validate_date("10-29-25").is_err());
        assert!(OriginErrorsClient::validate_date("ab-cd-efgh").is_err());
        assert!(OriginErrorsClient::validate_date("").is_err());
    }
}
