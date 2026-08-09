//! HTTP client for the bunny.net Database control plane (and a small
//! data-plane convenience for `ping`).

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use reqwest::{Client, RequestBuilder};

use crate::dry_run::check_dry_run;
use crate::recording::debug::{format_debug_body, print_debug_request_body};
use crate::recording::{capture_request, maybe_record_response};

use super::types::{
    ActiveStatsResponse, CreateDatabaseGroupPayload, CreateDatabaseGroupResponse,
    CreateDatabasePayload, CreateDatabaseResponse, CreateDatabaseV2Payload,
    CreateDatabaseV2Response, DeleteDatabaseResponse, DeletedDatabaseV2Response,
    DeletedGroupResponse, ForkDatabasePayload, ForkDatabaseResponse,
    GenerateTokenDatabaseGroupPayload, GenerateTokenDatabasePayload,
    GenerateTokenDatabaseV2Payload, GenerateTokenResponse, LimitsResponse, ListConfigResponse,
    ListDatabaseGroupResponse, ListDatabaseResponse, ListDatabaseV2Response,
    ListVersionsDatabasePayload, ListVersionsResponse, LiveMetricsForDBResponse,
    LiveMetricsForGroupResponse, OptimalConfigResponse, OptimalSingleConfigResponse, PingResult,
    ReadDatabaseGroupResponse, ReadDatabaseResponse, ReadDatabaseV2Response,
    RestoreDatabaseResponse, RestoreVersionDatabasePayload, StatsResponse,
    UpdateDatabaseGroupPayload, UpdateDatabaseGroupResponse, UpdateDatabaseV2Payload,
    UpdateDatabaseV2Response, UsageResponse,
};

const BASE_URL: &str = "https://api.bunny.net/database";

/// Client for the bunny.net Database (libSQL) control plane.
pub struct DatabaseClient {
    http: Client,
    base_url: String,
    api_key: String,
    debug: bool,
    debug_reveal_secrets: bool,
    dry_run: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl DatabaseClient {
    /// Create a new client using the provided API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: BASE_URL.to_string(),
            api_key: api_key.into(),
            debug: false,
            debug_reveal_secrets: false,
            dry_run: false,
            record_dir: None,
            last_request: Mutex::new(None),
        }
    }

    /// Override the base URL (useful for testing against a mock server).
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_owned();
        self
    }

    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// When debug is enabled, reveal secret field values in request/response
    /// bodies instead of redacting them.
    #[must_use]
    pub fn with_debug_reveal_secrets(mut self, reveal: bool) -> Self {
        self.debug_reveal_secrets = reveal;
        self
    }

    /// Preview mutating (POST/PUT/PATCH/DELETE) requests instead of sending
    /// them. Read-only requests (GET/HEAD) are unaffected.
    #[must_use]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    #[must_use]
    pub fn with_record(mut self, dir: impl Into<PathBuf>) -> Self {
        self.record_dir = Some(dir.into());
        self
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("AccessKey", &self.api_key)
    }

    fn encode(id: &str) -> String {
        urlencoding::encode(id).into_owned()
    }

    async fn send(&self, rb: RequestBuilder) -> Result<reqwest::Response> {
        let request = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", request.method(), request.url());
            print_debug_request_body(&request, self.debug_reveal_secrets);
        }
        capture_request(
            &self.last_request,
            request.method().as_ref(),
            request.url().path(),
        );
        check_dry_run(&request, self.dry_run, self.debug_reveal_secrets)?;
        self.http
            .execute(request)
            .await
            .context("HTTP request failed")
    }

    async fn read_body(
        &self,
        resp: reqwest::Response,
    ) -> Result<(reqwest::StatusCode, bytes::Bytes)> {
        let status = resp.status();
        let bytes = resp.bytes().await.context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!(
                "<<< {}",
                format_debug_body(&bytes, self.debug_reveal_secrets)
            );
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            "database",
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    async fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            serde_json::from_slice(&bytes)
                .with_context(|| format!("deserialising response (HTTP {status}): {body}"))
        } else {
            bail!("HTTP {status}: {}", String::from_utf8_lossy(&bytes));
        }
    }

    /// Drop the response body but check for a non-2xx status. Used by the
    /// `204 No Content` invalidate endpoints.
    async fn check_status(&self, resp: reqwest::Response) -> Result<()> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            Ok(())
        } else {
            bail!("HTTP {status}: {}", String::from_utf8_lossy(&bytes));
        }
    }

    // -----------------------------------------------------------------------
    // Config
    // -----------------------------------------------------------------------

    pub async fn get_config(&self) -> Result<ListConfigResponse> {
        let url = format!("{}/v1/config", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn get_config_limits(&self) -> Result<LimitsResponse> {
        let url = format!("{}/v1/config/limits", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    /// `GET /v1/config/optimal` — multi-region recommendation.
    ///
    /// The spec marks `cdn_server_token` as a required query parameter; omitting
    /// it makes bunny.net return HTTP 400.
    pub async fn get_optimal(&self, cdn_server_token: &str) -> Result<OptimalConfigResponse> {
        let url = format!("{}/v1/config/optimal", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("cdn_server_token", cdn_server_token)]);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// `GET /v1/config/optimal_single` — single-region recommendation.
    ///
    /// Like [`get_optimal`](Self::get_optimal), the spec requires the
    /// `cdn_server_token` query parameter.
    pub async fn get_optimal_single(
        &self,
        cdn_server_token: &str,
    ) -> Result<OptimalSingleConfigResponse> {
        let url = format!("{}/v1/config/optimal_single", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("cdn_server_token", cdn_server_token)]);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Database (v1)
    // -----------------------------------------------------------------------

    pub async fn list_databases(&self, group_id: Option<&str>) -> Result<ListDatabaseResponse> {
        let url = format!("{}/v1/databases", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(g) = group_id {
            rb = rb.query(&[("group_id", g)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn get_database(&self, db_id: &str) -> Result<ReadDatabaseResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn create_database(
        &self,
        body: &CreateDatabasePayload,
    ) -> Result<CreateDatabaseResponse> {
        let url = format!("{}/v1/databases", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn delete_database(&self, db_id: &str) -> Result<DeleteDatabaseResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}", self.base_url);
        let resp = self.send(self.auth(self.http.delete(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn fork_database(
        &self,
        db_id: &str,
        body: &ForkDatabasePayload,
    ) -> Result<ForkDatabaseResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}/fork", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn restore_database(
        &self,
        db_id: &str,
        body: &RestoreVersionDatabasePayload,
    ) -> Result<RestoreDatabaseResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}/restore", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn list_database_versions(
        &self,
        db_id: &str,
        body: &ListVersionsDatabasePayload,
    ) -> Result<ListVersionsResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}/list_versions", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn mint_database_token(
        &self,
        db_id: &str,
        body: &GenerateTokenDatabasePayload,
    ) -> Result<GenerateTokenResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}/auth/tokens", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn invalidate_database_keys(&self, db_id: &str) -> Result<()> {
        let id = Self::encode(db_id);
        let url = format!("{}/v1/databases/{id}/auth/invalidate", self.base_url);
        let resp = self.send(self.auth(self.http.post(&url))).await?;
        self.check_status(resp).await
    }

    // -----------------------------------------------------------------------
    // Database (v2)
    // -----------------------------------------------------------------------

    pub async fn list_databases_v2(
        &self,
        page: u32,
        per_page: Option<u32>,
        search: Option<&str>,
    ) -> Result<ListDatabaseV2Response> {
        let url = format!("{}/v2/databases", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        rb = rb.query(&[("page", page.to_string())]);
        if let Some(n) = per_page {
            rb = rb.query(&[("per_page", n.to_string())]);
        }
        if let Some(s) = search {
            rb = rb.query(&[("search", s)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn get_database_v2(&self, db_id: &str) -> Result<ReadDatabaseV2Response> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn create_database_v2(
        &self,
        body: &CreateDatabaseV2Payload,
    ) -> Result<CreateDatabaseV2Response> {
        let url = format!("{}/v2/databases", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn delete_database_v2(&self, db_id: &str) -> Result<DeletedDatabaseV2Response> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}", self.base_url);
        let resp = self.send(self.auth(self.http.delete(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn update_database_v2(
        &self,
        db_id: &str,
        body: &UpdateDatabaseV2Payload,
    ) -> Result<UpdateDatabaseV2Response> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}", self.base_url);
        let resp = self
            .send(self.auth(self.http.patch(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn get_active_usage_v2(&self) -> Result<ActiveStatsResponse> {
        let url = format!("{}/v2/databases/active_usage", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn get_database_statistics_v2(
        &self,
        db_id: &str,
        from: &str,
        to: &str,
    ) -> Result<StatsResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}/statistics", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("from", from), ("to", to)]);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn get_database_usage_v2(
        &self,
        db_id: &str,
        from: &str,
        to: &str,
    ) -> Result<UsageResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}/usage", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("from", from), ("to", to)]);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn mint_database_token_v2(
        &self,
        db_id: &str,
        body: &GenerateTokenDatabaseV2Payload,
    ) -> Result<GenerateTokenResponse> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}/auth/generate", self.base_url);
        let resp = self.send(self.auth(self.http.put(&url)).json(body)).await?;
        self.parse_response(resp).await
    }

    pub async fn revoke_database_token_v2(&self, db_id: &str) -> Result<()> {
        let id = Self::encode(db_id);
        let url = format!("{}/v2/databases/{id}/auth/revoke", self.base_url);
        let resp = self.send(self.auth(self.http.post(&url))).await?;
        self.check_status(resp).await
    }

    // -----------------------------------------------------------------------
    // DatabaseGroup
    // -----------------------------------------------------------------------

    pub async fn list_groups(&self, search: Option<&str>) -> Result<ListDatabaseGroupResponse> {
        let url = format!("{}/v1/groups", self.base_url);
        let mut rb = self.auth(self.http.get(&url));
        if let Some(s) = search {
            rb = rb.query(&[("search", s)]);
        }
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn get_group(&self, group_id: &str) -> Result<ReadDatabaseGroupResponse> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}", self.base_url);
        let resp = self.send(self.auth(self.http.get(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn create_group(
        &self,
        body: &CreateDatabaseGroupPayload,
    ) -> Result<CreateDatabaseGroupResponse> {
        let url = format!("{}/v1/groups", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn update_group(
        &self,
        group_id: &str,
        body: &UpdateDatabaseGroupPayload,
    ) -> Result<UpdateDatabaseGroupResponse> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}", self.base_url);
        let resp = self
            .send(self.auth(self.http.patch(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn delete_group(&self, group_id: &str) -> Result<DeletedGroupResponse> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}", self.base_url);
        let resp = self.send(self.auth(self.http.delete(&url))).await?;
        self.parse_response(resp).await
    }

    pub async fn get_group_stats(
        &self,
        group_id: &str,
        from: &str,
        to: &str,
    ) -> Result<StatsResponse> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}/stats", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("from", from), ("to", to)]);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn get_group_aggregated_usage(
        &self,
        group_id: &str,
        from: &str,
        to: &str,
    ) -> Result<UsageResponse> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}/aggregated_usage", self.base_url);
        let rb = self
            .auth(self.http.get(&url))
            .query(&[("from", from), ("to", to)]);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    pub async fn generate_group_keys(
        &self,
        group_id: &str,
        body: &GenerateTokenDatabaseGroupPayload,
    ) -> Result<GenerateTokenResponse> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}/auth/generate", self.base_url);
        let resp = self
            .send(self.auth(self.http.post(&url)).json(body))
            .await?;
        self.parse_response(resp).await
    }

    pub async fn invalidate_group_keys(&self, group_id: &str) -> Result<()> {
        let id = Self::encode(group_id);
        let url = format!("{}/v1/groups/{id}/auth/invalidate", self.base_url);
        let resp = self.send(self.auth(self.http.post(&url))).await?;
        self.check_status(resp).await
    }

    // -----------------------------------------------------------------------
    // Live metrics (custom request headers)
    // -----------------------------------------------------------------------

    /// Live DB metrics. The bunny API expects a comma-joined list of database
    /// IDs in the `db-ids` request header (non-standard).
    pub async fn live_metrics_db(&self, db_ids: &[String]) -> Result<LiveMetricsForDBResponse> {
        let url = format!("{}/v1/live/live_db", self.base_url);
        let body = serde_json::json!({ "db_ids": db_ids });
        let rb = self
            .auth(self.http.post(&url))
            .header("db-ids", db_ids.join(","))
            .json(&body);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    /// Live group metrics. The bunny API expects a comma-joined list of group
    /// IDs in the `group-ids` request header (non-standard).
    pub async fn live_metrics_group(
        &self,
        group_ids: &[String],
    ) -> Result<LiveMetricsForGroupResponse> {
        let url = format!("{}/v1/live/live_group", self.base_url);
        let body = serde_json::json!({ "group_ids": group_ids });
        let rb = self
            .auth(self.http.post(&url))
            .header("group-ids", group_ids.join(","))
            .json(&body);
        let resp = self.send(rb).await?;
        self.parse_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Data plane (libSQL): ping
    // -----------------------------------------------------------------------

    /// Ping a Database via its libSQL HTTP endpoint with `SELECT 1`.
    ///
    /// `database_url` should be the value of `Database.url` — typically
    /// `libsql://<group>-<slug>.lite.bunnydb.net/`. Casing and trailing slash
    /// must be preserved exactly. The libSQL HTTP endpoint lives at
    /// `<https-host>/v2/pipeline`; this method handles the scheme rewrite.
    pub async fn ping(&self, database_url: &str, token: &str) -> PingResult {
        let endpoint = match libsql_pipeline_url(database_url) {
            Ok(u) => u,
            Err(e) => {
                return PingResult {
                    ok: false,
                    latency_ms: 0,
                    error: Some(e.to_string()),
                };
            }
        };
        let body = serde_json::json!({
            "requests": [
                {"type": "execute", "stmt": {"sql": "SELECT 1"}},
                {"type": "close"}
            ]
        });
        let start = Instant::now();
        let req = self
            .http
            .post(&endpoint)
            .bearer_auth(token)
            .json(&body)
            .build();
        let req = match req {
            Ok(r) => r,
            Err(e) => {
                return PingResult {
                    ok: false,
                    latency_ms: 0,
                    error: Some(format!("request build failed: {e}")),
                };
            }
        };
        if self.debug {
            eprintln!(">> {} {}", req.method(), req.url());
        }
        match self.http.execute(req).await {
            Ok(resp) => {
                let status = resp.status();
                let latency_ms = start.elapsed().as_millis() as u64;
                let body = resp.text().await.unwrap_or_default();
                if status.is_success() {
                    PingResult {
                        ok: true,
                        latency_ms,
                        error: None,
                    }
                } else {
                    PingResult {
                        ok: false,
                        latency_ms,
                        error: Some(format!("HTTP {status}: {body}")),
                    }
                }
            }
            Err(e) => PingResult {
                ok: false,
                latency_ms: start.elapsed().as_millis() as u64,
                error: Some(format!("request failed: {e}")),
            },
        }
    }
}

/// Convert a `libsql://host/` URL into the matching HTTPS pipeline endpoint.
///
/// Accepts `libsql://`, `https://`, or `http://` (the last for tests). The
/// trailing path is preserved up to a single slash, then `v2/pipeline` is
/// appended.
fn libsql_pipeline_url(database_url: &str) -> Result<String> {
    let (scheme, rest) = if let Some(rest) = database_url.strip_prefix("libsql://") {
        ("https://", rest)
    } else if let Some(rest) = database_url.strip_prefix("https://") {
        ("https://", rest)
    } else if let Some(rest) = database_url.strip_prefix("http://") {
        ("http://", rest)
    } else {
        bail!("expected libsql://, https:// or http:// URL, got: {database_url}");
    };
    let trimmed = rest.trim_end_matches('/');
    Ok(format!("{scheme}{trimmed}/v2/pipeline"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_url_libsql() {
        let u = libsql_pipeline_url("libsql://group_01-my-app.lite.bunnydb.net/").unwrap();
        assert_eq!(u, "https://group_01-my-app.lite.bunnydb.net/v2/pipeline");
    }

    #[test]
    fn pipeline_url_http_for_tests() {
        let u = libsql_pipeline_url("http://127.0.0.1:8080/").unwrap();
        assert_eq!(u, "http://127.0.0.1:8080/v2/pipeline");
    }

    #[test]
    fn pipeline_url_rejects_unknown_scheme() {
        assert!(libsql_pipeline_url("ftp://x/").is_err());
    }

    #[test]
    fn client_constructs() {
        let c = DatabaseClient::new("k")
            .with_base_url("http://localhost")
            .with_debug(true);
        assert_eq!(c.base_url, "http://localhost");
        assert!(c.debug);
        assert_eq!(c.api_key, "k");
    }
}
