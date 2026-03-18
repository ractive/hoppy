use anyhow::{Result, anyhow};
use reqwest::{Client, StatusCode};

use crate::types::{
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
    api_key: String,
}

impl ComputeClient {
    /// Create a new client with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            api_key: api_key.into(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{BASE_URL}{path}")
    }

    /// Attach the API key header to every request.
    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.header("AccessKey", &self.api_key)
    }

    /// Deserialise a successful JSON body, or surface a structured [`ApiError`] on 4xx.
    async fn json_or_error<T: serde::de::DeserializeOwned>(
        &self,
        resp: reqwest::Response,
    ) -> Result<T> {
        let status = resp.status();
        if status.is_success() {
            Ok(resp.json::<T>().await?)
        } else {
            let api_err = resp.json::<ApiError>().await.unwrap_or(ApiError {
                error_key: None,
                field: None,
                message: Some(format!("HTTP {status}")),
            });
            Err(anyhow!("{api_err}"))
        }
    }

    /// Assert a no-body success (2xx) or surface an error.
    async fn expect_no_body(&self, resp: reqwest::Response) -> Result<()> {
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let api_err = resp.json::<ApiError>().await.unwrap_or(ApiError {
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

    /// List all edge scripts on the account, with optional pagination and search.
    pub async fn list_scripts(
        &self,
        page: Option<i32>,
        per_page: Option<i32>,
        search: Option<&str>,
    ) -> Result<PaginatedList<EdgeScript>> {
        let mut req = self.auth(self.http.get(self.url("/compute/script")));
        if let Some(p) = page {
            req = req.query(&[("page", p.to_string())]);
        }
        if let Some(pp) = per_page {
            req = req.query(&[("perPage", pp.to_string())]);
        }
        if let Some(s) = search {
            req = req.query(&[("search", s)]);
        }
        let resp = req.send().await?;
        self.json_or_error(resp).await
    }

    /// Get a single edge script by ID.
    pub async fn get_script(&self, id: i64) -> Result<EdgeScript> {
        let resp = self
            .auth(self.http.get(self.url(&format!("/compute/script/{id}"))))
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    /// Create a new edge script.
    pub async fn create_script(&self, body: &CreateEdgeScript) -> Result<EdgeScript> {
        let resp = self
            .auth(self.http.post(self.url("/compute/script")))
            .json(body)
            .send()
            .await?;

        // API returns 201 on success — treat any 2xx as success.
        let status = resp.status();
        if status.is_success() {
            Ok(resp.json::<EdgeScript>().await?)
        } else {
            let api_err = resp.json::<ApiError>().await.unwrap_or(ApiError {
                error_key: None,
                field: None,
                message: Some(format!("HTTP {status}")),
            });
            Err(anyhow!("{api_err}"))
        }
    }

    /// Update an existing edge script (name and/or type).
    pub async fn update_script(&self, id: i64, body: &UpdateEdgeScript) -> Result<EdgeScript> {
        let resp = self
            .auth(self.http.post(self.url(&format!("/compute/script/{id}"))))
            .json(body)
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    /// Delete an edge script. Pass `delete_linked_pull_zones: true` to also
    /// remove any pull zones linked to this script.
    pub async fn delete_script(&self, id: i64, delete_linked_pull_zones: bool) -> Result<()> {
        let resp = self
            .auth(
                self.http
                    .delete(self.url(&format!("/compute/script/{id}")))
                    .query(&[(
                        "deleteLinkedPullZones",
                        delete_linked_pull_zones.to_string(),
                    )]),
            )
            .send()
            .await?;
        self.expect_no_body(resp).await
    }

    /// Get the current (draft) source code of a script.
    pub async fn get_script_code(&self, id: i64) -> Result<EdgeScriptCode> {
        let resp = self
            .auth(
                self.http
                    .get(self.url(&format!("/compute/script/{id}/code"))),
            )
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    /// Replace the draft source code of a script.
    pub async fn update_script_code(&self, id: i64, code: &str) -> Result<()> {
        let body = UpdateEdgeScriptCode {
            code: Some(code.to_owned()),
        };
        let resp = self
            .auth(
                self.http
                    .post(self.url(&format!("/compute/script/{id}/code"))),
            )
            .json(&body)
            .send()
            .await?;
        self.expect_no_body(resp).await
    }

    /// Publish the current draft code as a new release.
    pub async fn publish_script(&self, id: i64, body: &PublishScript) -> Result<()> {
        let resp = self
            .auth(
                self.http
                    .post(self.url(&format!("/compute/script/{id}/publish"))),
            )
            .json(body)
            .send()
            .await?;
        self.expect_no_body(resp).await
    }

    /// Rotate the deployment key for a script.
    pub async fn rotate_deployment_key(&self, id: i64) -> Result<()> {
        let resp = self
            .auth(
                self.http
                    .post(self.url(&format!("/compute/script/{id}/deploymentKey/rotate"))),
            )
            .send()
            .await?;
        self.expect_no_body(resp).await
    }

    // -------------------------------------------------------------------------
    // Statistics
    // -------------------------------------------------------------------------

    /// Get usage statistics for a script. Optionally filter by date range.
    pub async fn get_script_statistics(
        &self,
        id: i64,
        date_from: Option<&str>,
        date_to: Option<&str>,
        hourly: bool,
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
        let resp = req.send().await?;
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
        if let Some(p) = page {
            req = req.query(&[("page", p.to_string())]);
        }
        if let Some(pp) = per_page {
            req = req.query(&[("perPage", pp.to_string())]);
        }
        let resp = req.send().await?;
        self.json_or_error(resp).await
    }

    /// Get the currently active (live) release for a script.
    pub async fn get_active_release(&self, id: i64) -> Result<EdgeScriptRelease> {
        let resp = self
            .auth(
                self.http
                    .get(self.url(&format!("/compute/script/{id}/releases/active"))),
            )
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    // -------------------------------------------------------------------------
    // Variables
    // -------------------------------------------------------------------------

    /// Add a new environment variable to a script.
    pub async fn add_variable(&self, id: i64, body: &AddVariable) -> Result<EdgeScriptVariable> {
        let resp = self
            .auth(
                self.http
                    .post(self.url(&format!("/compute/script/{id}/variables/add"))),
            )
            .json(body)
            .send()
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
            .auth(self.http.get(self.url(&format!(
                "/compute/script/{script_id}/variables/{variable_id}"
            ))))
            .send()
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
            .auth(self.http.post(self.url(&format!(
                "/compute/script/{script_id}/variables/{variable_id}"
            ))))
            .json(body)
            .send()
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
            .auth(
                self.http
                    .put(self.url(&format!("/compute/script/{id}/variables"))),
            )
            .json(body)
            .send()
            .await?;
        // 200 = created, 204 = updated (both may carry a body per spec)
        let status = resp.status();
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
        } else {
            self.json_or_error(resp).await
        }
    }

    /// Delete a variable by its ID.
    pub async fn delete_variable(&self, script_id: i64, variable_id: i64) -> Result<()> {
        let resp = self
            .auth(self.http.delete(self.url(&format!(
                "/compute/script/{script_id}/variables/{variable_id}"
            ))))
            .send()
            .await?;
        self.expect_no_body(resp).await
    }

    // -------------------------------------------------------------------------
    // Secrets
    // -------------------------------------------------------------------------

    /// List all secrets for a script (values are never returned).
    pub async fn list_secrets(&self, id: i64) -> Result<SecretList> {
        let resp = self
            .auth(
                self.http
                    .get(self.url(&format!("/compute/script/{id}/secrets"))),
            )
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    /// Add a new secret to a script.
    pub async fn add_secret(&self, id: i64, body: &AddSecret) -> Result<EdgeScriptSecret> {
        let resp = self
            .auth(
                self.http
                    .post(self.url(&format!("/compute/script/{id}/secrets"))),
            )
            .json(body)
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    /// Upsert (create or update) a secret identified by name.
    pub async fn upsert_secret(&self, id: i64, body: &UpsertSecret) -> Result<EdgeScriptSecret> {
        let resp = self
            .auth(
                self.http
                    .put(self.url(&format!("/compute/script/{id}/secrets"))),
            )
            .json(body)
            .send()
            .await?;
        // 200 = created (body returned), 204 = updated (no body per spec)
        let status = resp.status();
        if status == StatusCode::NO_CONTENT {
            Ok(EdgeScriptSecret {
                id: 0,
                name: body.name.clone(),
                last_modified: String::new(),
            })
        } else {
            self.json_or_error(resp).await
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
            .auth(
                self.http
                    .post(self.url(&format!("/compute/script/{script_id}/secrets/{secret_id}"))),
            )
            .json(body)
            .send()
            .await?;
        self.json_or_error(resp).await
    }

    /// Delete a secret by its ID.
    pub async fn delete_secret(&self, script_id: i64, secret_id: i64) -> Result<()> {
        let resp = self
            .auth(
                self.http
                    .delete(self.url(&format!("/compute/script/{script_id}/secrets/{secret_id}"))),
            )
            .send()
            .await?;
        self.expect_no_body(resp).await
    }
}
