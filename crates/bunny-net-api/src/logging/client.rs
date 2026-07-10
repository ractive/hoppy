//! HTTP client for the bunny.net CDN Logging API.
//!
//! Two endpoints are covered:
//! - v2 (`GET /v2/pullzones/{id}/logs`) — structured JSON with rich filtering
//!   and pagination, deserialized into [`LogQueryResponse`].
//! - v1 (`GET /{date}/{pullZoneId}.log`) — legacy pipe-delimited raw text,
//!   streamed chunk-by-chunk so arbitrarily large log files never buffer.
//!
//! Both authenticate with the `AccessKey` header.

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result, anyhow};
use reqwest::{Client, StatusCode};

use crate::recording::{capture_request, maybe_record_response};

use super::types::{ErrorResponse, LegacyLogParams, LogQueryParams, LogQueryResponse};

/// Production base URL for the CDN Logging API.
const BASE_URL: &str = "https://logging.bunnycdn.com";

/// Client for the bunny.net CDN Logging API.
///
/// Authenticates via the `AccessKey` header. Construct with [`LoggingClient::new`].
pub struct LoggingClient {
    http: Client,
    base_url: String,
    api_key: String,
    debug: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl LoggingClient {
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

    /// Enable recording API responses to files in the given directory.
    #[must_use]
    pub fn with_record(mut self, dir: impl Into<PathBuf>) -> Self {
        self.record_dir = Some(dir.into());
        self
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("AccessKey", &self.api_key)
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
            eprintln!("<<< {}", String::from_utf8_lossy(&bytes));
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            "logging",
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    /// Turn a non-success status + body into a structured error message.
    fn extract_error(&self, status: StatusCode, bytes: &[u8]) -> anyhow::Error {
        match serde_json::from_slice::<ErrorResponse>(bytes) {
            Ok(env) => {
                let msg = env
                    .error
                    .message
                    .unwrap_or_else(|| format!("HTTP {status}"));
                match env.error.code {
                    Some(code) => anyhow!("Logging API error {status} ({code}): {msg}"),
                    None => anyhow!("Logging API error {status}: {msg}"),
                }
            }
            Err(_) => anyhow!("Logging API returned HTTP {status}"),
        }
    }

    // -------------------------------------------------------------------------
    // v2 — structured JSON query
    // -------------------------------------------------------------------------

    /// Query CDN access logs for a pull zone (v2 structured endpoint).
    ///
    /// Every filter in [`LogQueryParams`] is pushed through as a query
    /// parameter; unset fields are omitted. Returns the paginated
    /// [`LogQueryResponse`] envelope.
    pub async fn query_logs(
        &self,
        pull_zone_id: i64,
        params: &LogQueryParams,
    ) -> Result<LogQueryResponse> {
        let mut req = self.auth(
            self.http
                .get(self.url(&format!("/v2/pullzones/{pull_zone_id}/logs"))),
        );

        if let Some(v) = &params.from {
            req = req.query(&[("from", v)]);
        }
        if let Some(v) = &params.to {
            req = req.query(&[("to", v)]);
        }
        if let Some(v) = &params.status {
            req = req.query(&[("status", v)]);
        }
        if let Some(v) = &params.cache_status {
            req = req.query(&[("cacheStatus", v)]);
        }
        if let Some(v) = &params.country {
            req = req.query(&[("country", v)]);
        }
        if let Some(v) = &params.edge_location {
            req = req.query(&[("edgeLocation", v)]);
        }
        if let Some(v) = &params.remote_ip {
            req = req.query(&[("remoteIp", v)]);
        }
        if let Some(v) = &params.url_contains {
            req = req.query(&[("urlContains", v)]);
        }
        if let Some(v) = &params.user_agent_contains {
            req = req.query(&[("userAgentContains", v)]);
        }
        if let Some(v) = &params.referer_contains {
            req = req.query(&[("refererContains", v)]);
        }
        if let Some(v) = &params.search {
            req = req.query(&[("search", v)]);
        }
        if let Some(v) = &params.request_id {
            req = req.query(&[("requestId", v)]);
        }
        if let Some(v) = params.include_origin_shield {
            req = req.query(&[("includeOriginShield", v.to_string())]);
        }
        if let Some(v) = params.limit {
            req = req.query(&[("limit", v.to_string())]);
        }
        if let Some(v) = params.offset {
            req = req.query(&[("offset", v.to_string())]);
        }
        if let Some(v) = &params.order {
            req = req.query(&[("order", v)]);
        }

        let resp = self.execute(req).await?;
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            serde_json::from_slice(&bytes).context("deserializing log query response")
        } else {
            Err(self.extract_error(status, &bytes))
        }
    }

    // -------------------------------------------------------------------------
    // v1 — legacy raw text, streamed
    // -------------------------------------------------------------------------

    /// Build the v1 legacy raw-log request builder for `date`/`pull_zone_id`
    /// with the given filters applied. Split out so it is unit-testable.
    fn legacy_request(
        &self,
        date: &str,
        pull_zone_id: i64,
        params: &LegacyLogParams,
    ) -> reqwest::RequestBuilder {
        let mut req = self.auth(
            self.http
                .get(self.url(&format!("/{date}/{pull_zone_id}.log"))),
        );
        if let Some(v) = params.start {
            req = req.query(&[("start", v.to_string())]);
        }
        if let Some(v) = params.end {
            req = req.query(&[("end", v.to_string())]);
        }
        if let Some(v) = &params.sort {
            req = req.query(&[("sort", v)]);
        }
        if let Some(v) = &params.status {
            req = req.query(&[("status", v)]);
        }
        if let Some(v) = &params.search {
            req = req.query(&[("search", v)]);
        }
        if let Some(v) = params.download {
            req = req.query(&[("download", v.to_string())]);
        }
        req
    }

    /// Stream the v1 (legacy) raw access log for a pull zone on a given date.
    ///
    /// The response body is raw pipe-delimited text and can be arbitrarily
    /// large, so it is streamed chunk-by-chunk into `writer` — peak memory
    /// stays bounded by the chunk size rather than the log size. Returns the
    /// total number of bytes written.
    ///
    /// `date` is the log date as accepted by the API (e.g. `"08-11-25"`).
    /// On a non-success status the (small) error body is buffered so a useful
    /// error can be surfaced.
    pub async fn stream_legacy_logs<W>(
        &self,
        date: &str,
        pull_zone_id: i64,
        params: &LegacyLogParams,
        writer: &mut W,
    ) -> Result<u64>
    where
        W: std::io::Write,
    {
        let req = self.legacy_request(date, pull_zone_id, params);
        let mut resp = self.execute(req).await?;

        let status = resp.status();
        if self.debug {
            eprintln!("<< {status}");
        }
        if !status.is_success() {
            let bytes = resp.bytes().await.context("failed to read error body")?;
            return Err(self.extract_error(status, &bytes));
        }

        let mut total: u64 = 0;
        while let Some(chunk) = resp
            .chunk()
            .await
            .context("failed to read response chunk")?
        {
            writer
                .write_all(&chunk)
                .context("failed to write log chunk")?;
            total += chunk.len() as u64;
        }
        writer.flush().context("failed to flush writer")?;
        Ok(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new_uses_default_base_url() {
        let client = LoggingClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn with_base_url_trims_trailing_slash() {
        let client = LoggingClient::with_base_url("k", "http://localhost:9000/");
        assert_eq!(
            client.url("/v2/pullzones/1/logs"),
            "http://localhost:9000/v2/pullzones/1/logs"
        );
    }

    #[test]
    fn with_debug_sets_flag() {
        assert!(LoggingClient::new("k").with_debug(true).debug);
        assert!(!LoggingClient::new("k").debug);
    }

    #[test]
    fn legacy_request_builds_expected_url_and_query() {
        let client = LoggingClient::with_base_url("k", "http://host");
        let params = LegacyLogParams {
            start: Some(1000),
            end: Some(2000),
            sort: Some("asc".into()),
            status: Some("500".into()),
            search: Some("error".into()),
            download: Some(true),
        };
        let req = client
            .legacy_request("08-11-25", 42, &params)
            .build()
            .unwrap();
        assert_eq!(req.url().path(), "/08-11-25/42.log");
        let q = req.url().query().unwrap();
        assert!(q.contains("start=1000"), "query was: {q}");
        assert!(q.contains("end=2000"), "query was: {q}");
        assert!(q.contains("sort=asc"), "query was: {q}");
        assert!(q.contains("status=500"), "query was: {q}");
        assert!(q.contains("search=error"), "query was: {q}");
        assert!(q.contains("download=true"), "query was: {q}");
    }

    #[test]
    fn legacy_request_omits_unset_params() {
        let client = LoggingClient::with_base_url("k", "http://host");
        let req = client
            .legacy_request("08-11-25", 42, &LegacyLogParams::default())
            .build()
            .unwrap();
        assert_eq!(req.url().path(), "/08-11-25/42.log");
        assert!(req.url().query().is_none_or(str::is_empty));
    }

    #[test]
    fn extract_error_parses_structured_envelope() {
        let client = LoggingClient::new("k");
        let body =
            br#"{"error":{"code":"logging_not_enabled","message":"Logging is not enabled"}}"#;
        let err = client.extract_error(StatusCode::NOT_FOUND, body);
        let msg = err.to_string();
        assert!(msg.contains("logging_not_enabled"), "msg was: {msg}");
        assert!(msg.contains("Logging is not enabled"), "msg was: {msg}");
    }

    #[test]
    fn extract_error_falls_back_on_non_json() {
        let client = LoggingClient::new("k");
        let err = client.extract_error(StatusCode::INTERNAL_SERVER_ERROR, b"boom");
        assert!(err.to_string().contains("HTTP 500"));
    }
}
