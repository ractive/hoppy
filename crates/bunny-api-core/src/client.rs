use anyhow::{Context, Result};
use reqwest::{
    StatusCode,
    header::{self},
};

use crate::types::{
    AddDnsRecord, ApiError, BillingDetails, CreateDnsZone, CreatePullZone, CreateStorageZone,
    CreateVideoLibrary, DnsRecord, DnsZone, PaginatedList, PullZone, PurgeCache, StorageZone,
    UpdateDnsRecord, UpdateDnsZone, UpdatePullZone, UpdateStorageZone, UpdateVideoLibrary,
    VideoLibrary,
};

const DEFAULT_BASE_URL: &str = "https://api.bunny.net";
/// Maximum items per page accepted by the bunny.net API.
const DEFAULT_PER_PAGE: u32 = 1000;

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
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    // -----------------------------------------------------------------------
    // Pull Zone endpoints
    // -----------------------------------------------------------------------

    /// List Pull Zones with optional pagination and search.
    ///
    /// `page` is 1-based; defaults to 1. `per_page` defaults to 1000 (the API
    /// maximum). Both are always sent so the API returns a paginated envelope.
    /// Callers with more than 1000 zones must page manually.
    pub async fn list_pull_zones(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<&str>,
    ) -> Result<PaginatedList<PullZone>> {
        let url = format!("{}/pullzone", self.base_url);
        // Always send pagination params — without them the bunny.net API
        // returns a bare JSON array instead of the paginated envelope.
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
        let mut rb = self.auth(self.http.get(&url)).query(&[
            ("page", page.to_string()),
            ("perPage", per_page.to_string()),
        ]);
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
    // Storage Zone endpoints
    // -----------------------------------------------------------------------

    /// List Storage Zones with optional pagination, search, and deleted filter.
    ///
    /// `page` is 1-based; defaults to 1. `per_page` defaults to 1000 (the API
    /// maximum). Both are always sent so the API returns a paginated envelope.
    pub async fn list_storage_zones(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<&str>,
        include_deleted: Option<bool>,
    ) -> Result<PaginatedList<StorageZone>> {
        let url = format!("{}/storagezone", self.base_url);
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
        let mut rb = self.auth(self.http.get(&url)).query(&[
            ("page", page.to_string()),
            ("perPage", per_page.to_string()),
        ]);
        if let Some(q) = search {
            rb = rb.query(&[("search", q)]);
        }
        if let Some(deleted) = include_deleted {
            rb = rb.query(&[("includeDeleted", deleted.to_string())]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch a single Storage Zone by its numeric ID.
    pub async fn get_storage_zone(&self, id: i64) -> Result<StorageZone> {
        let url = format!("{}/storagezone/{id}", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Create a new Storage Zone.
    pub async fn create_storage_zone(&self, body: &CreateStorageZone) -> Result<StorageZone> {
        let url = format!("{}/storagezone", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Update an existing Storage Zone.
    ///
    /// The bunny.net API uses `POST` (not `PATCH`) for updates and returns 204.
    pub async fn update_storage_zone(&self, id: i64, body: &UpdateStorageZone) -> Result<()> {
        let url = format!("{}/storagezone/{id}", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Delete a Storage Zone permanently.
    pub async fn delete_storage_zone(&self, id: i64) -> Result<()> {
        let url = format!("{}/storagezone/{id}", self.base_url);
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // DNS Zone endpoints
    // -----------------------------------------------------------------------

    /// List DNS Zones with optional pagination and search.
    pub async fn list_dns_zones(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<&str>,
    ) -> Result<PaginatedList<DnsZone>> {
        let url = format!("{}/dnszone", self.base_url);
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
        let mut rb = self.auth(self.http.get(&url)).query(&[
            ("page", page.to_string()),
            ("perPage", per_page.to_string()),
        ]);
        if let Some(q) = search {
            rb = rb.query(&[("search", q)]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch a single DNS Zone by its numeric ID.
    pub async fn get_dns_zone(&self, id: i64) -> Result<DnsZone> {
        let url = format!("{}/dnszone/{id}", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Create a new DNS Zone.
    pub async fn create_dns_zone(&self, body: &CreateDnsZone) -> Result<DnsZone> {
        let url = format!("{}/dnszone", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Update an existing DNS Zone.
    pub async fn update_dns_zone(&self, id: i64, body: &UpdateDnsZone) -> Result<DnsZone> {
        let url = format!("{}/dnszone/{id}", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Delete a DNS Zone permanently.
    pub async fn delete_dns_zone(&self, id: i64) -> Result<()> {
        let url = format!("{}/dnszone/{id}", self.base_url);
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // DNS Record endpoints
    // -----------------------------------------------------------------------

    /// Add a DNS record to a zone.
    ///
    /// Note: bunny.net uses `PUT` (not `POST`) for record creation.
    pub async fn add_dns_record(&self, zone_id: i64, body: &AddDnsRecord) -> Result<DnsRecord> {
        let url = format!("{}/dnszone/{zone_id}/records", self.base_url);
        let rb = self.auth(self.http.put(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Update a DNS record.
    pub async fn update_dns_record(
        &self,
        zone_id: i64,
        record_id: i64,
        body: &UpdateDnsRecord,
    ) -> Result<()> {
        let url = format!("{}/dnszone/{zone_id}/records/{record_id}", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Delete a DNS record.
    pub async fn delete_dns_record(&self, zone_id: i64, record_id: i64) -> Result<()> {
        let url = format!("{}/dnszone/{zone_id}/records/{record_id}", self.base_url);
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // Video Library endpoints
    // -----------------------------------------------------------------------

    /// List Video Libraries with optional pagination and search.
    pub async fn list_video_libraries(
        &self,
        page: Option<u32>,
        per_page: Option<u32>,
        search: Option<&str>,
    ) -> Result<PaginatedList<VideoLibrary>> {
        let url = format!("{}/videolibrary", self.base_url);
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(DEFAULT_PER_PAGE);
        let mut rb = self.auth(self.http.get(&url)).query(&[
            ("page", page.to_string()),
            ("perPage", per_page.to_string()),
        ]);
        if let Some(q) = search {
            rb = rb.query(&[("search", q)]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch a single Video Library by its numeric ID.
    pub async fn get_video_library(&self, id: i64) -> Result<VideoLibrary> {
        let url = format!("{}/videolibrary/{id}", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Create a new Video Library.
    pub async fn create_video_library(&self, body: &CreateVideoLibrary) -> Result<VideoLibrary> {
        let url = format!("{}/videolibrary", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Update an existing Video Library.
    ///
    /// The bunny.net API uses `POST` (not `PATCH`) for updates and returns 200.
    pub async fn update_video_library(
        &self,
        id: i64,
        body: &UpdateVideoLibrary,
    ) -> Result<VideoLibrary> {
        let url = format!("{}/videolibrary/{id}", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Delete a Video Library permanently.
    pub async fn delete_video_library(&self, id: i64) -> Result<()> {
        let url = format!("{}/videolibrary/{id}", self.base_url);
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // Billing / account endpoints
    // -----------------------------------------------------------------------

    /// Fetch account billing details including balance and monthly charges.
    ///
    /// A successful response confirms that the API key is valid and returns
    /// the current account financial summary.
    pub async fn get_billing(&self) -> Result<BillingDetails> {
        let url = format!("{}/billing", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Execute a prepared request, logging method and URL before sending when
    /// debug mode is enabled.
    ///
    /// Only method and URL are logged — the `AccessKey` header value is never
    /// included. Status and body are logged by [`read_body`].
    ///
    /// [`read_body`]: CoreClient::read_body
    async fn send(&self, rb: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let request = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", request.method(), request.url());
        }
        self.http
            .execute(request)
            .await
            .context("HTTP request failed")
    }

    fn auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        rb.header("AccessKey", &self.api_key)
            .header(header::ACCEPT, "application/json")
    }

    /// Read the response body, logging status and body when debug is enabled.
    async fn read_body(&self, response: reqwest::Response) -> Result<(StatusCode, bytes::Bytes)> {
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!("<<< {}", String::from_utf8_lossy(&bytes));
        }
        Ok((status, bytes))
    }

    /// Convert a successful or error response into `Ok(T)` or an `Err`.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T> {
        let (status, bytes) = self.read_body(response).await?;

        if status.is_success() {
            return serde_json::from_slice(&bytes).context("failed to deserialise response body");
        }

        Err(self.extract_api_error(status, &bytes))
    }

    /// Convert a successful or error response that carries no JSON body.
    async fn handle_empty_response(&self, response: reqwest::Response) -> Result<()> {
        let (status, bytes) = self.read_body(response).await?;

        if status.is_success() {
            return Ok(());
        }

        Err(self.extract_api_error(status, &bytes))
    }

    /// Try to parse a bunny.net `ApiError` JSON body; fall back to a plain
    /// status-code error when the body is not parseable JSON.
    fn extract_api_error(&self, status: StatusCode, bytes: &bytes::Bytes) -> anyhow::Error {
        if let Ok(mut api_err) = serde_json::from_slice::<ApiError>(bytes) {
            // Fill in status code if the body didn't include it.
            if api_err.status_code == 0 {
                api_err.status_code = status.as_u16();
            }
            anyhow::Error::new(api_err)
        } else {
            let body_text = String::from_utf8_lossy(bytes);
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
