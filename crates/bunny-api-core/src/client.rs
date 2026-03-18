use anyhow::{Context, Result};
use reqwest::{
    StatusCode,
    header::{self},
};

use crate::types::{ApiError, CreatePullZone, PaginatedList, PullZone, PurgeCache, UpdatePullZone};

const DEFAULT_BASE_URL: &str = "https://api.bunny.net";

/// Async client for the bunny.net HTTP API.
///
/// Create one instance per application and reuse it — the underlying
/// `reqwest::Client` manages a connection pool.
#[derive(Debug, Clone)]
pub struct CoreClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    debug: bool,
}

impl CoreClient {
    /// Create a client with the production base URL.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Create a client pointing at a custom base URL (useful for tests /
    /// staging environments).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            debug: false,
        }
    }

    /// Enable or disable debug logging of HTTP requests and responses to stderr.
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    // -----------------------------------------------------------------------
    // Pull Zone endpoints
    // -----------------------------------------------------------------------

    /// List Pull Zones with optional pagination and search.
    ///
    /// `page` is 1-based. Pass `None` to use the API default (page 1).
    pub async fn list_pull_zones(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<&str>,
    ) -> Result<PaginatedList<PullZone>> {
        let url = format!("{}/pullzone", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(p) = page {
            rb = rb.query(&[("page", p.to_string())]);
        }
        if let Some(pp) = per_page {
            rb = rb.query(&[("perPage", pp.to_string())]);
        }
        if let Some(q) = search {
            rb = rb.query(&[("search", q)]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch a single Pull Zone by its numeric ID.
    pub async fn get_pull_zone(&self, id: i64) -> Result<PullZone> {
        let url = format!("{}/pullzone/{id}", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Create a new Pull Zone.
    pub async fn create_pull_zone(&self, body: &CreatePullZone) -> Result<PullZone> {
        let url = format!("{}/pullzone", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Update an existing Pull Zone.
    ///
    /// The bunny.net API uses `POST` (not `PATCH`) for updates.
    pub async fn update_pull_zone(&self, id: i64, body: &UpdatePullZone) -> Result<PullZone> {
        let url = format!("{}/pullzone/{id}", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Delete a Pull Zone permanently.
    pub async fn delete_pull_zone(&self, id: i64) -> Result<()> {
        let url = format!("{}/pullzone/{id}", self.base_url);
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Purge the cache for a Pull Zone.
    ///
    /// Pass [`PurgeCache::all()`] to purge everything, or
    /// [`PurgeCache::by_tag()`] to limit the purge to a cache tag.
    pub async fn purge_pull_zone_cache(&self, id: i64, body: &PurgeCache) -> Result<()> {
        let url = format!("{}/pullzone/{id}/purgeCache", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Execute a prepared request, logging method and URL before sending and
    /// status after receiving when debug mode is enabled.
    ///
    /// Only method, URL, and status are logged — the `AccessKey` header value
    /// is never included.
    async fn send(&self, rb: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let request = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", request.method(), request.url());
        }
        let response = self
            .http
            .execute(request)
            .await
            .context("HTTP request failed")?;
        if self.debug {
            eprintln!("<< {}", response.status());
        }
        Ok(response)
    }

    fn auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.header("AccessKey", &self.api_key)
            .header(header::ACCEPT, "application/json")
    }

    /// Convert a successful or error response into `Ok(T)` or an `Err`.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let body = response
                .json::<T>()
                .await
                .context("failed to deserialise response body")?;
            return Ok(body);
        }

        Err(self.extract_api_error(status, response).await)
    }

    /// Convert a successful or error response that carries no JSON body.
    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<()> {
        let status = response.status();

        if status.is_success() {
            return Ok(());
        }

        Err(self.extract_api_error(status, response).await)
    }

    /// Try to parse a bunny.net `ApiError` JSON body; fall back to a plain
    /// status-code error when the body is not parseable JSON.
    async fn extract_api_error(
        &self,
        status: StatusCode,
        response: reqwest::Response,
    ) -> anyhow::Error {
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return anyhow::anyhow!("HTTP {status}: (could not read response body: {e})");
            }
        };

        if let Ok(mut api_err) = serde_json::from_slice::<ApiError>(&bytes) {
            // Fill in status code if the body didn't include it.
            if api_err.status_code == 0 {
                api_err.status_code = status.as_u16();
            }
            anyhow::Error::new(api_err)
        } else {
            let body_text = String::from_utf8_lossy(&bytes);
            anyhow::anyhow!("HTTP {status}: {body_text}")
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_client_accepts_string_api_key() {
        let client = CoreClient::new("test-key-123");
        assert_eq!(client.api_key, "test-key-123");
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn with_base_url_strips_trailing_slash() {
        let client = CoreClient::with_base_url("key", "https://example.com/");
        assert_eq!(client.base_url, "https://example.com");
    }

    #[test]
    fn auth_sets_access_key_header() {
        let client = CoreClient::new("my-secret");
        // Build a request and verify the header is attached.
        let rb = client.auth(client.http.get("http://localhost"));
        let req = rb.build().unwrap();
        let access_key = req.headers().get("AccessKey").unwrap().to_str().unwrap();
        assert_eq!(access_key, "my-secret");
        let accept = req.headers().get(header::ACCEPT).unwrap().to_str().unwrap();
        assert_eq!(accept, "application/json");
    }
}
