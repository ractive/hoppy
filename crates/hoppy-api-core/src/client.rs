use anyhow::{Context, Result};
use reqwest::{
    Method, StatusCode,
    header::{self, HeaderMap, HeaderValue},
};

use crate::types::{ApiError, CreatePullZone, PaginatedList, PullZone, PurgeCache, UpdatePullZone};

const DEFAULT_BASE_URL: &str = "https://api.bunny.net";

/// Async client for the bunny.net HTTP API.
///
/// Create one instance per application and reuse it — the underlying
/// `reqwest::Client` manages a connection pool.
#[derive(Debug, Clone)]
pub struct BunnyClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl BunnyClient {
    /// Create a client with the production base URL.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Create a client pointing at a custom base URL (useful for tests /
    /// staging environments).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .build()
            .context("failed to build reqwest client")?;

        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
        })
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
        let mut url = format!("{}/pullzone", self.base_url);
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(pp) = per_page {
            params.push(("perPage", pp.to_string()));
        }
        if let Some(q) = search {
            params.push(("search", q.to_owned()));
        }

        if !params.is_empty() {
            let qs = params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&");
            url = format!("{url}?{qs}");
        }

        self.request(Method::GET, &url, None::<&()>).await
    }

    /// Fetch a single Pull Zone by its numeric ID.
    pub async fn get_pull_zone(&self, id: i64) -> Result<PullZone> {
        let url = format!("{}/pullzone/{id}", self.base_url);
        self.request(Method::GET, &url, None::<&()>).await
    }

    /// Create a new Pull Zone.
    pub async fn create_pull_zone(&self, body: &CreatePullZone) -> Result<PullZone> {
        let url = format!("{}/pullzone", self.base_url);
        self.request(Method::POST, &url, Some(body)).await
    }

    /// Update an existing Pull Zone.
    ///
    /// The bunny.net API uses `POST` (not `PATCH`) for updates.
    pub async fn update_pull_zone(&self, id: i64, body: &UpdatePullZone) -> Result<PullZone> {
        let url = format!("{}/pullzone/{id}", self.base_url);
        self.request(Method::POST, &url, Some(body)).await
    }

    /// Delete a Pull Zone permanently.
    pub async fn delete_pull_zone(&self, id: i64) -> Result<()> {
        let url = format!("{}/pullzone/{id}", self.base_url);
        self.request_no_body(Method::DELETE, &url).await
    }

    /// Purge the cache for a Pull Zone.
    ///
    /// Pass [`PurgeCache::all()`] to purge everything, or
    /// [`PurgeCache::by_tag()`] to limit the purge to a cache tag.
    pub async fn purge_pull_zone_cache(&self, id: i64, body: &PurgeCache) -> Result<()> {
        let url = format!("{}/pullzone/{id}/purgeCache", self.base_url);
        self.request_no_body_with_json(Method::POST, &url, body)
            .await
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn auth_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let key = HeaderValue::from_str(&self.api_key)
            .context("API key contains invalid header characters")?;
        headers.insert("AccessKey", key);
        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// Send a request that deserialises a JSON response body into `T`.
    async fn request<T, B>(&self, method: Method, url: &str, body: Option<&B>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let headers = self.auth_headers()?;
        let mut builder = self.http.request(method, url).headers(headers);

        if let Some(b) = body {
            builder = builder.json(b);
        }

        let response = builder.send().await.context("HTTP request failed")?;
        self.handle_response(response).await
    }

    /// Send a request that expects an empty / no response body on success.
    async fn request_no_body(&self, method: Method, url: &str) -> Result<()> {
        let headers = self.auth_headers()?;
        let response = self
            .http
            .request(method, url)
            .headers(headers)
            .send()
            .await
            .context("HTTP request failed")?;

        self.handle_empty_response(response).await
    }

    /// Send a request with a JSON body that expects an empty response on success.
    async fn request_no_body_with_json<B>(&self, method: Method, url: &str, body: &B) -> Result<()>
    where
        B: serde::Serialize + ?Sized,
    {
        let headers = self.auth_headers()?;
        let response = self
            .http
            .request(method, url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .context("HTTP request failed")?;

        self.handle_empty_response(response).await
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

        match serde_json::from_slice::<ApiError>(&bytes) {
            Ok(mut api_err) => {
                // Fill in status code if the body didn't include it.
                if api_err.status_code == 0 {
                    api_err.status_code = status.as_u16();
                }
                anyhow::Error::new(api_err)
            }
            Err(_) => {
                let body_text = String::from_utf8_lossy(&bytes);
                anyhow::anyhow!("HTTP {status}: {body_text}")
            }
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
        let client = BunnyClient::new("test-key-123").unwrap();
        assert_eq!(client.api_key, "test-key-123");
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn with_base_url_strips_trailing_slash() {
        let client = BunnyClient::with_base_url("key", "https://example.com/").unwrap();
        assert_eq!(client.base_url, "https://example.com");
    }

    #[test]
    fn auth_headers_contain_access_key() {
        let client = BunnyClient::new("my-secret").unwrap();
        let headers = client.auth_headers().unwrap();
        let access_key = headers.get("AccessKey").unwrap().to_str().unwrap();
        assert_eq!(access_key, "my-secret");
    }
}
