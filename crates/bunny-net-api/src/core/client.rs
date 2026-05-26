use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use reqwest::{
    StatusCode,
    header::{self},
};

use crate::recording::{capture_request, maybe_record_response};

use super::types::{
    AccountStatistics, AddDnsRecord, ApiError, BillingDetails, CreateDnsZone, CreatePullZone,
    CreateStorageZone, CreateVideoLibrary, DnsImportResult, DnsRecord, DnsRecordScanResult,
    DnsRecordScanTrigger, DnsSecDsRecord, DnsZone, DnsZoneStatistics, OptimizerStatistics,
    OriginShieldQueueStatistics, PaginatedList, PullZone, PurgeCache, SafeHopStatistics,
    StorageZone, StorageZoneStatistics, TriggerDnsRecordScan, UpdateDnsRecord, UpdateDnsZone,
    UpdatePullZone, UpdateStorageZone, UpdateVideoLibrary, VideoLibrary, VideoLibraryDrmStatistics,
    VideoLibraryTranscribingStatistics,
};

const DEFAULT_BASE_URL: &str = "https://api.bunny.net";
/// Maximum items per page accepted by the bunny.net API.
const DEFAULT_PER_PAGE: u32 = 1000;

/// Async client for the bunny.net HTTP API.
///
/// Create one instance per application and reuse it — the underlying
/// `reqwest::Client` manages a connection pool.
pub struct CoreClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    debug: bool,
    debug_reveal_secrets: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl Clone for CoreClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
            debug: self.debug,
            debug_reveal_secrets: self.debug_reveal_secrets,
            record_dir: self.record_dir.clone(),
            last_request: Mutex::new(None),
        }
    }
}

impl std::fmt::Debug for CoreClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreClient")
            .field("base_url", &self.base_url)
            .field("debug", &self.debug)
            .field("record_dir", &self.record_dir)
            .finish()
    }
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
            debug_reveal_secrets: false,
            record_dir: None,
            last_request: Mutex::new(None),
        }
    }

    /// Enable or disable debug logging of HTTP requests and responses to stderr.
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// When debug is enabled, reveal secret field values in request/response body
    /// logs instead of replacing them with `<set, length=N>`.
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

    /// Purge a single URL from the CDN cache.
    ///
    /// This is a global purge — it does not target a specific Pull Zone.
    pub async fn purge_url(&self, url: &str) -> Result<()> {
        let endpoint = format!("{}/purge", self.base_url);
        let rb = self.auth(self.http.post(&endpoint)).query(&[("url", url)]);
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // Pull Zone hostname & SSL endpoints
    // -----------------------------------------------------------------------

    /// Add a custom hostname to a Pull Zone.
    pub async fn add_hostname(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/addHostname", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Remove a custom hostname from a Pull Zone.
    pub async fn remove_hostname(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/removeHostname", self.base_url);
        let rb = self.auth(self.http.delete(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Load a free Let's Encrypt certificate for a hostname.
    pub async fn load_free_certificate(&self, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/loadFreeCertificate", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("hostname", hostname)]);
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Set Force SSL on a hostname.
    pub async fn set_force_ssl(&self, id: i64, hostname: &str, force_ssl: bool) -> Result<()> {
        let url = format!("{}/pullzone/{id}/setForceSSL", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname,
            "ForceSSL": force_ssl
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Add a custom SSL certificate to a Pull Zone hostname.
    ///
    /// Both `certificate` and `private_key` must be Base64-encoded PEM strings
    /// as required by the bunny.net API.
    pub async fn add_certificate(
        &self,
        id: i64,
        hostname: &str,
        certificate: &str,
        private_key: &str,
    ) -> Result<()> {
        let url = format!("{}/pullzone/{id}/addCertificate", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname,
            "Certificate": certificate,
            "CertificateKey": private_key
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Remove the SSL certificate from a Pull Zone hostname.
    pub async fn remove_certificate(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/removeCertificate", self.base_url);
        let rb = self.auth(self.http.delete(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // Pull Zone access-control endpoints
    // -----------------------------------------------------------------------

    /// Add an allowed referrer to a Pull Zone.
    pub async fn add_allowed_referrer(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/addAllowedReferrer", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Remove an allowed referrer from a Pull Zone.
    pub async fn remove_allowed_referrer(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/removeAllowedReferrer", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Add a blocked referrer to a Pull Zone.
    pub async fn add_blocked_referrer(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/addBlockedReferrer", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Remove a blocked referrer from a Pull Zone.
    pub async fn remove_blocked_referrer(&self, id: i64, hostname: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/removeBlockedReferrer", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Hostname": hostname
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Add a blocked IP to a Pull Zone.
    pub async fn add_blocked_ip(&self, id: i64, ip: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/addBlockedIp", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "BlockedIp": ip
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Remove a blocked IP from a Pull Zone.
    pub async fn remove_blocked_ip(&self, id: i64, ip: &str) -> Result<()> {
        let url = format!("{}/pullzone/{id}/removeBlockedIp", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "BlockedIp": ip
        }));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // Pull Zone edge rule endpoints
    // -----------------------------------------------------------------------

    /// Create or update an edge rule on a Pull Zone (upsert by Guid).
    pub async fn add_or_update_edge_rule(
        &self,
        pull_zone_id: i64,
        body: &super::types::AddOrUpdateEdgeRule,
    ) -> Result<()> {
        let url = format!(
            "{}/pullzone/{pull_zone_id}/edgerules/addOrUpdate",
            self.base_url
        );
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Delete an edge rule from a Pull Zone.
    pub async fn delete_edge_rule(&self, pull_zone_id: i64, edge_rule_id: &str) -> Result<()> {
        let url = format!(
            "{}/pullzone/{pull_zone_id}/edgerules/{edge_rule_id}",
            self.base_url
        );
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    /// Enable or disable an edge rule on a Pull Zone.
    pub async fn set_edge_rule_enabled(
        &self,
        pull_zone_id: i64,
        edge_rule_id: &str,
        enabled: bool,
    ) -> Result<()> {
        let url = format!(
            "{}/pullzone/{pull_zone_id}/edgerules/{edge_rule_id}/setEdgeRuleEnabled",
            self.base_url
        );
        let rb = self.auth(self.http.post(&url)).json(&serde_json::json!({
            "Id": pull_zone_id,
            "Value": enabled
        }));
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

    /// Export a DNS zone as a BIND zone file (plain text).
    pub async fn export_dns_zone(&self, id: i64) -> Result<String> {
        let url = format!("{}/dnszone/{id}/export", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        let (status, bytes) = self.read_body(response).await?;
        if status.is_success() {
            return Ok(String::from_utf8_lossy(&bytes).into_owned());
        }
        Err(self.extract_api_error(status, &bytes))
    }

    /// Import DNS records from a BIND zone file.
    ///
    /// `zone_file` is the raw BIND zone file content (plain text). The bunny.net
    /// API accepts the file as multipart/form-data; we send it as the `file` part.
    pub async fn import_dns_zone(&self, id: i64, zone_file: &str) -> Result<DnsImportResult> {
        use reqwest::header::CONTENT_TYPE;
        let url = format!("{}/dnszone/{id}/import", self.base_url);
        // Build a minimal multipart body manually: bunny.net expects a field named "file".
        let boundary = format!(
            "HoppyBoundary{:016x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"zone.txt\"\r\nContent-Type: text/plain\r\n\r\n{zone_file}\r\n--{boundary}--\r\n"
        );
        let content_type = format!("multipart/form-data; boundary={boundary}");
        let rb = self
            .auth(self.http.post(&url))
            .header(CONTENT_TYPE, content_type)
            .body(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    // -----------------------------------------------------------------------
    // DNSSEC endpoints
    // -----------------------------------------------------------------------

    /// Enable DNSSEC on a DNS zone.
    ///
    /// Returns the DS record details (digest, key tag, algorithm, etc.) that
    /// must be configured at the domain registrar to complete DNSSEC setup.
    pub async fn enable_dns_zone_dnssec(&self, id: i64) -> Result<DnsSecDsRecord> {
        let url = format!("{}/dnszone/{id}/dnssec", self.base_url);
        let rb = self.auth(self.http.post(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Disable DNSSEC on a DNS zone.
    ///
    /// Disabling DNSSEC while DS records remain at the registrar will break
    /// resolution — the caller is expected to remove the DS records first.
    pub async fn disable_dns_zone_dnssec(&self, id: i64) -> Result<DnsSecDsRecord> {
        let url = format!("{}/dnszone/{id}/dnssec", self.base_url);
        let rb = self.auth(self.http.delete(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    // -----------------------------------------------------------------------
    // DNS Zone wildcard certificate endpoint
    // -----------------------------------------------------------------------

    /// Issue a free wildcard TLS certificate for a DNS zone.
    ///
    /// The zone must be properly delegated to bunny.net nameservers for the
    /// DNS-01 challenge to succeed (wildcard certificates require DNS-01).
    /// Returns `()` because the API returns an empty 200 response on success.
    pub async fn issue_dns_zone_wildcard_certificate(&self, zone_id: i64) -> Result<()> {
        let url = format!("{}/dnszone/{zone_id}/certificate/issue", self.base_url);
        let rb = self.auth(self.http.post(&url));
        let response = self.send(rb).await?;
        self.handle_empty_response(response).await
    }

    // -----------------------------------------------------------------------
    // DNS Record scan endpoints
    // -----------------------------------------------------------------------

    /// Trigger a background scan for pre-existing DNS records.
    ///
    /// The request body must specify either an existing zone (`zone_id`) or a
    /// raw `domain` (for pre-zone-creation scenarios), but not both. Returns
    /// the scan job id and initial status; results are fetched separately
    /// via [`Self::get_dns_zone_record_scan`].
    pub async fn trigger_dns_record_scan(
        &self,
        body: &TriggerDnsRecordScan,
    ) -> Result<DnsRecordScanTrigger> {
        let url = format!("{}/dnszone/records/scan", self.base_url);
        let rb = self.auth(self.http.post(&url)).json(body);
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch the latest DNS record scan job for a zone.
    pub async fn get_dns_zone_record_scan(&self, zone_id: i64) -> Result<DnsRecordScanResult> {
        let url = format!("{}/dnszone/{zone_id}/records/scan", self.base_url);
        let rb = self.auth(self.http.get(&url));
        let response = self.send(rb).await?;
        self.handle_response(response).await
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
    // Statistics endpoints
    // -----------------------------------------------------------------------

    /// Fetch account-level statistics.
    pub async fn get_statistics(
        &self,
        date_from: Option<&str>,
        date_to: Option<&str>,
        pull_zone: Option<i64>,
        hourly: bool,
    ) -> Result<AccountStatistics> {
        let url = format!("{}/statistics", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        if let Some(v) = pull_zone {
            rb = rb.query(&[("pullZone", v.to_string())]);
        }
        if hourly {
            rb = rb.query(&[("hourly", "true")]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch statistics for a Storage Zone.
    pub async fn get_storage_zone_statistics(
        &self,
        id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<StorageZoneStatistics> {
        let url = format!("{}/storagezone/{id}/statistics", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch statistics for a DNS Zone.
    pub async fn get_dns_zone_statistics(
        &self,
        id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<DnsZoneStatistics> {
        let url = format!("{}/dnszone/{id}/statistics", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch optimizer statistics for a Pull Zone.
    pub async fn get_pull_zone_optimizer_statistics(
        &self,
        pull_zone_id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
        hourly: bool,
    ) -> Result<OptimizerStatistics> {
        let url = format!(
            "{}/pullzone/{pull_zone_id}/optimizer/statistics",
            self.base_url
        );
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        if hourly {
            rb = rb.query(&[("hourly", "true")]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch origin shield queue statistics for a Pull Zone.
    pub async fn get_pull_zone_origin_shield_statistics(
        &self,
        pull_zone_id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
        hourly: bool,
    ) -> Result<OriginShieldQueueStatistics> {
        let url = format!(
            "{}/pullzone/{pull_zone_id}/originshield/queuestatistics",
            self.base_url
        );
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        if hourly {
            rb = rb.query(&[("hourly", "true")]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch SafeHop statistics for a Pull Zone.
    pub async fn get_pull_zone_safehop_statistics(
        &self,
        pull_zone_id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
        hourly: bool,
    ) -> Result<SafeHopStatistics> {
        let url = format!(
            "{}/pullzone/{pull_zone_id}/safehop/statistics",
            self.base_url
        );
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        if hourly {
            rb = rb.query(&[("hourly", "true")]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch DRM statistics for a Video Library.
    pub async fn get_video_library_drm_statistics(
        &self,
        id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<VideoLibraryDrmStatistics> {
        let url = format!("{}/videolibrary/{id}/drm/statistics", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
        let response = self.send(rb).await?;
        self.handle_response(response).await
    }

    /// Fetch transcribing statistics for a Video Library.
    pub async fn get_video_library_transcribing_statistics(
        &self,
        id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<VideoLibraryTranscribingStatistics> {
        let url = format!(
            "{}/videolibrary/{id}/transcribing/statistics",
            self.base_url
        );
        let mut rb = self.auth(self.http.get(&url));
        if let Some(v) = date_from {
            rb = rb.query(&[("dateFrom", v)]);
        }
        if let Some(v) = date_to {
            rb = rb.query(&[("dateTo", v)]);
        }
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
            let method = request.method();
            let is_mutating = method == reqwest::Method::POST
                || method == reqwest::Method::PUT
                || method == reqwest::Method::PATCH
                || method == reqwest::Method::DELETE;
            if is_mutating {
                if let Some(body_bytes) = request.body().and_then(|b| b.as_bytes()) {
                    eprintln!(
                        ">>> {}",
                        format_debug_body(body_bytes, self.debug_reveal_secrets)
                    );
                } else if request.body().is_some() {
                    eprintln!(">>> <streaming body>");
                }
            }
        }
        capture_request(
            &self.last_request,
            request.method().as_ref(),
            request.url().path(),
        );
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
            eprintln!(
                "<<< {}",
                format_debug_body(&bytes, self.debug_reveal_secrets)
            );
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            "core",
            &self.last_request,
            status.is_success(),
            &bytes,
        );
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
// Debug body formatting helpers
// ---------------------------------------------------------------------------

const DEBUG_BODY_TRUNCATE: usize = 4096;

/// Format a request/response body for debug output.
///
/// - If the bytes are valid JSON, pretty-prints them (with optional redaction).
/// - Otherwise returns a UTF-8 lossy representation truncated at 4 KiB.
pub fn format_debug_body(bytes: &[u8], reveal: bool) -> String {
    if bytes.is_empty() {
        return "<empty>".to_owned();
    }
    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if !reveal {
            redact_debug_body(&mut value);
        }
        return serde_json::to_string_pretty(&value).unwrap_or_else(|_| "<json error>".to_owned());
    }
    let text = String::from_utf8_lossy(bytes);
    if bytes.len() > DEBUG_BODY_TRUNCATE {
        format!(
            "{}… ({} bytes total)",
            &text[..DEBUG_BODY_TRUNCATE],
            bytes.len()
        )
    } else {
        text.into_owned()
    }
}

/// Walk a JSON value and redact string-valued fields whose names suggest
/// they hold secrets (token, password, secret, _key).
fn redact_debug_body(value: &mut serde_json::Value) {
    const KEY_SUFFIX_NOT_SECRET: &[&str] = &["zonesecuritykey", "userpkkey", "publickey"];
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower = k.to_lowercase();
                let is_allowlisted = KEY_SUFFIX_NOT_SECRET.iter().any(|s| lower.ends_with(s));
                let is_secret = !is_allowlisted
                    && (lower.ends_with("password")
                        || lower.ends_with("_password")
                        || lower.ends_with("secret")
                        || lower.ends_with("_secret")
                        || lower.ends_with("token")
                        || lower.ends_with("_token")
                        || lower.ends_with("apikey")
                        || lower.ends_with("api_key")
                        || lower.ends_with("_key")
                        || lower.contains("credential"));
                if is_secret {
                    if let serde_json::Value::String(raw) = v {
                        let len = raw.chars().count();
                        if len == 0 {
                            *v = serde_json::Value::String("<unset>".to_owned());
                        } else {
                            *v = serde_json::Value::String(format!("<set, length={len}>"));
                        }
                    } else if v.is_null() {
                        *v = serde_json::Value::String("<unset>".to_owned());
                    } else {
                        redact_debug_body(v);
                    }
                } else {
                    redact_debug_body(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                redact_debug_body(v);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_debug_body_redacts_token_by_default() {
        let body = serde_json::json!({"LogForwardingToken": "abc"});
        let bytes = serde_json::to_vec(&body).unwrap();
        let out = format_debug_body(&bytes, false);
        assert!(
            out.contains("<set, length=3>"),
            "expected redacted token, got: {out}"
        );
        assert!(!out.contains("abc"), "expected token to be redacted");
    }

    #[test]
    fn format_debug_body_reveals_token_when_reveal_true() {
        let body = serde_json::json!({"LogForwardingToken": "abc"});
        let bytes = serde_json::to_vec(&body).unwrap();
        let out = format_debug_body(&bytes, true);
        assert!(out.contains("abc"), "expected token revealed, got: {out}");
    }

    #[test]
    fn format_debug_body_empty() {
        assert_eq!(format_debug_body(&[], false), "<empty>");
    }

    #[test]
    fn format_debug_body_non_json_truncation() {
        let long = "x".repeat(5000);
        let bytes = long.as_bytes();
        let out = format_debug_body(bytes, false);
        assert!(
            out.contains("5000 bytes total"),
            "expected truncation note, got: {out}"
        );
    }

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
