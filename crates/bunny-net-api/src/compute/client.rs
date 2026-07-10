use crate::recording::{capture_request, maybe_record_response};
use anyhow::{Context, Result, anyhow};
use reqwest::{Client, Request, StatusCode};
use std::path::PathBuf;
use std::sync::Mutex;

use super::types::{
    AddSecret, AddVariable, ApiError, CreateEdgeScript, EdgeScript, EdgeScriptCode,
    EdgeScriptRelease, EdgeScriptSecret, EdgeScriptStatistics, EdgeScriptVariable, PaginatedList,
    PublishScript, SecretList, UpdateEdgeScript, UpdateEdgeScriptCode, UpdateSecret,
    UpdateVariable, UpsertSecret, UpsertVariable,
};

const BASE_URL: &str = "https://api.bunny.net";

/// Client for the bunny.net Edge Scripting (Compute) API.
///
/// Authenticates via the `AccessKey` header. Construct with [`ComputeClient::new`].
pub struct ComputeClient {
    http: Client,
    base_url: String,
    api_key: String,
    debug: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl ComputeClient {
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

    /// Attach the API key header to every request.
    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("AccessKey", &self.api_key)
    }

    /// Build and execute a request, printing method and URL to stderr when debug is enabled.
    async fn execute(&self, rb: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let req: Request = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", req.method(), req.url());
        }
        if self.record_dir.is_some() {
            capture_request(&self.last_request, req.method().as_ref(), req.url().path());
        }
        self.http.execute(req).await.context("request failed")
    }

    /// Read the response body, logging status and body when debug is enabled.
    async fn read_body(&self, resp: reqwest::Response) -> Result<(StatusCode, bytes::Bytes)> {
        let status = resp.status();
        let bytes = resp.bytes().await.context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!("<<< {}", String::from_utf8_lossy(&bytes));
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            "compute",
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    /// Deserialise a successful JSON body, or surface a structured [`ApiError`] on 4xx.
    async fn json_or_error<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            let api_err = serde_json::from_slice::<ApiError>(&bytes).unwrap_or(ApiError {
                error_key: None,
                field: None,
                message: Some(format!("HTTP {status}")),
            });
            Err(anyhow!("{api_err}"))
        }
    }

    /// Assert a no-body success (2xx) or surface an error.
    async fn expect_no_body(&self, resp: reqwest::Response) -> Result<()> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            Ok(())
        } else {
            let api_err = serde_json::from_slice::<ApiError>(&bytes).unwrap_or(ApiError {
                error_key: None,
                field: None,
                message: Some(format!("HTTP {status}")),
            });
            Err(anyhow!("{api_err}"))
        }
    }

    // -------------------------------------------------------------------------
    // Scripts
    // -------------------------------------------------------------------------

    /// List all edge scripts on the account, with optional pagination, search,
    /// and server-side filters.
    ///
    /// `types` filters by script type (repeatable; each value is sent as its
    /// own `type` query param). `integration_id` restricts the result to a
    /// single integration, and `include_linked_pullzones` asks the API to
    /// embed each script's linked Pull Zones in the response.
    pub async fn list_scripts(
        &self,
        page: Option<i32>,
        per_page: Option<i32>,
        search: Option<&str>,
        types: &[i32],
        integration_id: Option<i64>,
        include_linked_pullzones: bool,
    ) -> Result<PaginatedList<EdgeScript>> {
        let mut req = self.auth(self.http.get(self.url("/compute/script")));
        req = req.query(&[("page", page.unwrap_or(1).to_string())]);
        req = req.query(&[("perPage", per_page.unwrap_or(1000).to_string())]);
        if let Some(s) = search {
            req = req.query(&[("search", s)]);
        }
        for t in types {
            req = req.query(&[("type", t.to_string())]);
        }
        if let Some(id) = integration_id {
            req = req.query(&[("integrationId", id.to_string())]);
        }
        if include_linked_pullzones {
            req = req.query(&[("includeLinkedPullZones", "true")]);
        }
        let resp = self.execute(req).await?;
        self.json_or_error(resp).await
    }

    /// Get a single edge script by ID.
    pub async fn get_script(&self, id: i64) -> Result<EdgeScript> {
        let resp = self
            .execute(self.auth(self.http.get(self.url(&format!("/compute/script/{id}")))))
            .await?;
        self.json_or_error(resp).await
    }

    /// Create a new edge script.
    pub async fn create_script(&self, body: &CreateEdgeScript) -> Result<EdgeScript> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/compute/script")))
                    .json(body),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Update an existing edge script (name and/or type).
    pub async fn update_script(&self, id: i64, body: &UpdateEdgeScript) -> Result<EdgeScript> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url(&format!("/compute/script/{id}"))))
                    .json(body),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Delete an edge script. Pass `delete_linked_pull_zones: true` to also
    /// remove any pull zones linked to this script.
    pub async fn delete_script(&self, id: i64, delete_linked_pull_zones: bool) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .delete(self.url(&format!("/compute/script/{id}")))
                        .query(&[(
                            "deleteLinkedPullZones",
                            delete_linked_pull_zones.to_string(),
                        )]),
                ),
            )
            .await?;
        self.expect_no_body(resp).await
    }

    /// Get the current (draft) source code of a script.
    pub async fn get_script_code(&self, id: i64) -> Result<EdgeScriptCode> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/compute/script/{id}/code"))),
                ),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Replace the draft source code of a script.
    pub async fn update_script_code(&self, id: i64, code: &str) -> Result<()> {
        let body = UpdateEdgeScriptCode {
            code: Some(code.to_owned()),
        };
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/compute/script/{id}/code"))),
                )
                .json(&body),
            )
            .await?;
        self.expect_no_body(resp).await
    }

    /// Publish the current draft code as a new release.
    pub async fn publish_script(&self, id: i64, body: &PublishScript) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/compute/script/{id}/publish"))),
                )
                .json(body),
            )
            .await?;
        self.expect_no_body(resp).await
    }

    /// Rotate the deployment key for a script.
    pub async fn rotate_deployment_key(&self, id: i64) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/compute/script/{id}/deploymentKey/rotate"))),
                ),
            )
            .await?;
        self.expect_no_body(resp).await
    }

    // -------------------------------------------------------------------------
    // Statistics
    // -------------------------------------------------------------------------

    /// Get usage statistics for a script. Optionally filter by date range.
    ///
    /// Set `load_latest` to have the API return the most recent data point
    /// even when it falls outside the requested `date_from` / `date_to` window.
    pub async fn get_script_statistics(
        &self,
        id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
        hourly: bool,
        load_latest: bool,
    ) -> Result<EdgeScriptStatistics> {
        let mut req = self.auth(
            self.http
                .get(self.url(&format!("/compute/script/{id}/statistics"))),
        );
        if let Some(df) = date_from {
            req = req.query(&[("dateFrom", df)]);
        }
        if let Some(dt) = date_to {
            req = req.query(&[("dateTo", dt)]);
        }
        req = req.query(&[("hourly", hourly.to_string())]);
        if load_latest {
            req = req.query(&[("loadLatest", "true")]);
        }
        let resp = self.execute(req).await?;
        self.json_or_error(resp).await
    }

    // -------------------------------------------------------------------------
    // Releases
    // -------------------------------------------------------------------------

    /// List all published releases for a script.
    pub async fn list_releases(
        &self,
        id: i64,
        page: Option<i32>,
        per_page: Option<i32>,
    ) -> Result<PaginatedList<EdgeScriptRelease>> {
        let mut req = self.auth(
            self.http
                .get(self.url(&format!("/compute/script/{id}/releases"))),
        );
        req = req.query(&[("page", page.unwrap_or(1).to_string())]);
        req = req.query(&[("perPage", per_page.unwrap_or(1000).to_string())]);
        let resp = self.execute(req).await?;
        self.json_or_error(resp).await
    }

    /// Get the currently active (live) release for a script.
    pub async fn get_active_release(&self, id: i64) -> Result<EdgeScriptRelease> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/compute/script/{id}/releases/active"))),
                ),
            )
            .await?;
        self.json_or_error(resp).await
    }

    // -------------------------------------------------------------------------
    // Variables
    // -------------------------------------------------------------------------

    /// Add a new environment variable to a script.
    pub async fn add_variable(&self, id: i64, body: &AddVariable) -> Result<EdgeScriptVariable> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/compute/script/{id}/variables/add"))),
                )
                .json(body),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Get a specific variable by its ID.
    pub async fn get_variable(
        &self,
        script_id: i64,
        variable_id: i64,
    ) -> Result<EdgeScriptVariable> {
        let resp = self
            .execute(self.auth(self.http.get(self.url(&format!(
                "/compute/script/{script_id}/variables/{variable_id}"
            )))))
            .await?;
        self.json_or_error(resp).await
    }

    /// Update a variable's default value or required flag.
    pub async fn update_variable(
        &self,
        script_id: i64,
        variable_id: i64,
        body: &UpdateVariable,
    ) -> Result<EdgeScriptVariable> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url(&format!(
                    "/compute/script/{script_id}/variables/{variable_id}"
                ))))
                .json(body),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Upsert (create or update) a variable identified by name.
    pub async fn upsert_variable(
        &self,
        id: i64,
        body: &UpsertVariable,
    ) -> Result<EdgeScriptVariable> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/compute/script/{id}/variables"))),
                )
                .json(body),
            )
            .await?;
        // 200 = created, 204 = updated (both may carry a body per spec)
        let (status, bytes) = self.read_body(resp).await?;
        if status == StatusCode::NO_CONTENT {
            // Spec says 204 may return a body but in practice it may be empty.
            // Return a minimal placeholder; callers that need the full model
            // should call get_variable afterwards.
            Ok(EdgeScriptVariable {
                id: 0,
                name: Some(body.name.clone()),
                required: body.required.unwrap_or(false),
                default_value: body.default_value.clone(),
            })
        } else if status.is_success() {
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            let api_err = serde_json::from_slice::<ApiError>(&bytes).unwrap_or(ApiError {
                error_key: None,
                field: None,
                message: Some(format!("HTTP {status}")),
            });
            Err(anyhow!("{api_err}"))
        }
    }

    /// Delete a variable by its ID.
    pub async fn delete_variable(&self, script_id: i64, variable_id: i64) -> Result<()> {
        let resp = self
            .execute(self.auth(self.http.delete(self.url(&format!(
                "/compute/script/{script_id}/variables/{variable_id}"
            )))))
            .await?;
        self.expect_no_body(resp).await
    }

    // -------------------------------------------------------------------------
    // Secrets
    // -------------------------------------------------------------------------

    /// List all secrets for a script (values are never returned).
    pub async fn list_secrets(&self, id: i64) -> Result<SecretList> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/compute/script/{id}/secrets"))),
                ),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Add a new secret to a script.
    pub async fn add_secret(&self, id: i64, body: &AddSecret) -> Result<EdgeScriptSecret> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/compute/script/{id}/secrets"))),
                )
                .json(body),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Upsert (create or update) a secret identified by name.
    pub async fn upsert_secret(&self, id: i64, body: &UpsertSecret) -> Result<EdgeScriptSecret> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/compute/script/{id}/secrets"))),
                )
                .json(body),
            )
            .await?;
        // 200 = created (body returned), 204 = updated (no body per spec)
        let (status, bytes) = self.read_body(resp).await?;
        if status == StatusCode::NO_CONTENT {
            Ok(EdgeScriptSecret {
                id: 0,
                name: body.name.clone(),
                last_modified: String::new(),
            })
        } else if status.is_success() {
            Ok(serde_json::from_slice(&bytes)?)
        } else {
            let api_err = serde_json::from_slice::<ApiError>(&bytes).unwrap_or(ApiError {
                error_key: None,
                field: None,
                message: Some(format!("HTTP {status}")),
            });
            Err(anyhow!("{api_err}"))
        }
    }

    /// Update a secret's value.
    pub async fn update_secret(
        &self,
        script_id: i64,
        secret_id: i64,
        body: &UpdateSecret,
    ) -> Result<EdgeScriptSecret> {
        let resp = self
            .execute(
                self.auth(
                    self.http.post(
                        self.url(&format!("/compute/script/{script_id}/secrets/{secret_id}")),
                    ),
                )
                .json(body),
            )
            .await?;
        self.json_or_error(resp).await
    }

    /// Delete a secret by its ID.
    pub async fn delete_secret(&self, script_id: i64, secret_id: i64) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http.delete(
                        self.url(&format!("/compute/script/{script_id}/secrets/{secret_id}")),
                    ),
                ),
            )
            .await?;
        self.expect_no_body(resp).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::types::UpsertVariable;

    #[test]
    fn client_new_uses_default_base_url() {
        let client = ComputeClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn client_with_base_url_overrides() {
        let client = ComputeClient::with_base_url("key", "http://localhost:9000");
        assert_eq!(
            client.url("/compute/script"),
            "http://localhost:9000/compute/script"
        );
    }

    #[test]
    fn client_debug_defaults_false() {
        let client = ComputeClient::new("key");
        assert!(!client.debug);
    }

    #[test]
    fn client_with_debug_sets_flag() {
        let client = ComputeClient::new("key").with_debug(true);
        assert!(client.debug);
    }

    #[test]
    fn upsert_variable_placeholder_construction() {
        // The placeholder built inside upsert_variable when the API returns 204.
        let body = UpsertVariable {
            name: "MY_VAR".to_owned(),
            required: Some(true),
            default_value: Some("default".to_owned()),
        };

        // Simulate the placeholder logic from upsert_variable.
        let placeholder = crate::compute::types::EdgeScriptVariable {
            id: 0,
            name: Some(body.name.clone()),
            required: body.required.unwrap_or(false),
            default_value: body.default_value.clone(),
        };

        assert_eq!(placeholder.id, 0);
        assert_eq!(placeholder.name.as_deref(), Some("MY_VAR"));
        assert!(placeholder.required);
        assert_eq!(placeholder.default_value.as_deref(), Some("default"));
    }
}
