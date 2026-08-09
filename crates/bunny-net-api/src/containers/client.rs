use anyhow::{Context, Result, bail};
use bytes::Bytes;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::dry_run::check_dry_run;
use crate::recording::debug::{format_debug_body, print_debug_request_body};
use crate::recording::{capture_request, maybe_record_response};

use super::types::*;

const BASE_URL: &str = "https://api.bunny.net/mc";

/// Client for the bunny.net Magic Containers API.
///
/// All methods are `async` and return `anyhow::Result<T>`. HTTP errors are
/// surfaced either as [`ErrorDetails`] / [`ProblemDetails`] (when the server
/// returns a structured body) or as a plain anyhow error with the status code.
pub struct ContainersClient {
    http: Client,
    base_url: String,
    api_key: String,
    debug: bool,
    debug_reveal_secrets: bool,
    dry_run: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl ContainersClient {
    /// Create a new client with the given API key, using the production base URL.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, BASE_URL)
    }

    /// Create a client pointing at a custom base URL (useful for tests / staging).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
            debug: false,
            debug_reveal_secrets: false,
            dry_run: false,
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

    /// Enable recording API responses to files in the given directory.
    #[must_use]
    pub fn with_record(mut self, dir: impl Into<PathBuf>) -> Self {
        self.record_dir = Some(dir.into());
        self
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("AccessKey", &self.api_key)
    }

    async fn execute(&self, rb: RequestBuilder) -> Result<Response> {
        let req = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", req.method(), req.url());
            print_debug_request_body(&req, self.debug_reveal_secrets);
        }
        capture_request(&self.last_request, req.method().as_ref(), req.url().path());
        check_dry_run(&req, self.dry_run, self.debug_reveal_secrets)?;
        self.http.execute(req).await.context("request failed")
    }

    /// Read the response body, logging status and body when debug is enabled.
    async fn read_body(&self, resp: Response) -> Result<(StatusCode, Bytes)> {
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
            "containers",
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(&self, resp: Response) -> Result<T> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            return serde_json::from_slice(&bytes).context("failed to decode success response");
        }
        self.surface_error(status, &bytes)
    }

    async fn handle_empty_response(&self, resp: Response) -> Result<()> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            return Ok(());
        }
        self.surface_error(status, &bytes)
    }

    fn surface_error<T>(&self, status: StatusCode, bytes: &Bytes) -> Result<T> {
        let body = String::from_utf8_lossy(bytes);
        // Try ErrorDetails first (more structured), then ProblemDetails.
        if let Ok(err) = serde_json::from_slice::<ErrorDetails>(bytes)
            && (err.title.is_some() || err.status.is_some())
        {
            bail!(err);
        }
        if let Ok(problem) = serde_json::from_slice::<ProblemDetails>(bytes)
            && (problem.title.is_some() || problem.status.is_some())
        {
            bail!(problem);
        }
        bail!("API error (HTTP {status}): {body}");
    }

    // -------------------------------------------------------------------------
    // Applications
    // -------------------------------------------------------------------------

    /// List all applications.
    ///
    /// `GET /apps`
    pub async fn list_applications(
        &self,
        next_cursor: Option<&str>,
        limit: Option<i32>,
    ) -> Result<CursorList<AppListItem>> {
        let mut req = self.auth(self.http.get(self.url("/apps")));
        if let Some(c) = next_cursor {
            req = req.query(&[("nextCursor", c)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        let resp = self.execute(req).await?;
        self.handle_response(resp).await
    }

    /// Get a single application by ID.
    ///
    /// `GET /apps/{appId}`
    pub async fn get_application(&self, app_id: &str) -> Result<Application> {
        let resp = self
            .execute(self.auth(self.http.get(self.url(&format!("/apps/{app_id}")))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get an application overview (status indicators, region/pod details, costs).
    ///
    /// `GET /apps/{appId}/overview`
    pub async fn get_application_overview(&self, app_id: &str) -> Result<ApplicationOverview> {
        let resp = self
            .execute(self.auth(self.http.get(self.url(&format!("/apps/{app_id}/overview")))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get an application usage summary.
    ///
    /// `GET /apps/{appId}/summary`
    ///
    /// The bunny.net spec documents this operation ("Get Application Usage
    /// Summary") without a response schema, so the raw JSON is returned as a
    /// [`serde_json::Value`] rather than a typed struct. Live-verify the shape
    /// before adding a typed model (see api-coverage research §4.5).
    pub async fn get_application_summary(&self, app_id: &str) -> Result<serde_json::Value> {
        let resp = self
            .execute(self.auth(self.http.get(self.url(&format!("/apps/{app_id}/summary")))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get application statistics (time-series charts).
    ///
    /// `GET /apps/{appId}/statistics`
    pub async fn get_application_statistics(
        &self,
        app_id: &str,
        from_date: &str,
        granularity: Granularity,
        to_date: Option<&str>,
    ) -> Result<ApplicationStatistics> {
        let gran_str = match granularity {
            Granularity::Daily => "Daily",
            Granularity::Hourly => "Hourly",
            Granularity::Minute => "Minute",
        };
        let mut req = self.auth(
            self.http
                .get(self.url(&format!("/apps/{app_id}/statistics"))),
        );
        req = req.query(&[("fromDate", from_date), ("granularity", gran_str)]);
        if let Some(to) = to_date {
            req = req.query(&[("toDate", to)]);
        }
        let resp = self.execute(req).await?;
        self.handle_response(resp).await
    }

    /// Create a new application.
    ///
    /// `POST /apps`
    pub async fn add_application(
        &self,
        body: &AddApplicationRequest,
    ) -> Result<AddApplicationResponse> {
        let resp = self
            .execute(self.auth(self.http.post(self.url("/apps"))).json(body))
            .await?;
        self.handle_response(resp).await
    }

    /// Full-replace update of an application.
    ///
    /// `PUT /apps/{appId}`
    pub async fn update_application(
        &self,
        app_id: &str,
        body: &AddApplicationRequest,
    ) -> Result<AddApplicationResponse> {
        let resp = self
            .execute(
                self.auth(self.http.put(self.url(&format!("/apps/{app_id}"))))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Partially update an application.
    ///
    /// `PATCH /apps/{appId}`
    pub async fn patch_application(
        &self,
        app_id: &str,
        body: &PatchApplicationRequest,
    ) -> Result<AddApplicationResponse> {
        let resp = self
            .execute(
                self.auth(self.http.patch(self.url(&format!("/apps/{app_id}"))))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Deploy an application.
    ///
    /// `POST /apps/{appId}/deploy`
    pub async fn deploy_application(&self, app_id: &str) -> Result<()> {
        let resp = self
            .execute(self.auth(self.http.post(self.url(&format!("/apps/{app_id}/deploy")))))
            .await?;
        self.handle_empty_response(resp).await
    }

    /// Undeploy an application.
    ///
    /// `POST /apps/{appId}/undeploy`
    pub async fn undeploy_application(&self, app_id: &str) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/apps/{app_id}/undeploy"))),
                ),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    /// Restart an application.
    ///
    /// `POST /apps/{appId}/restart`
    pub async fn restart_application(&self, app_id: &str) -> Result<()> {
        let resp = self
            .execute(self.auth(self.http.post(self.url(&format!("/apps/{app_id}/restart")))))
            .await?;
        self.handle_empty_response(resp).await
    }

    /// Delete an application.
    ///
    /// `DELETE /apps/{appId}`
    pub async fn delete_application(&self, app_id: &str) -> Result<()> {
        let resp = self
            .execute(self.auth(self.http.delete(self.url(&format!("/apps/{app_id}")))))
            .await?;
        self.handle_empty_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Autoscaling
    // -------------------------------------------------------------------------

    /// Get autoscaling settings for an application.
    ///
    /// `GET /apps/{appId}/autoscaling`
    pub async fn get_autoscaling(&self, app_id: &str) -> Result<AutoscalingSettings> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/apps/{app_id}/autoscaling"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update autoscaling settings for an application.
    ///
    /// `PUT /apps/{appId}/autoscaling`
    pub async fn update_autoscaling(&self, app_id: &str, body: &AutoscalingSettings) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/apps/{app_id}/autoscaling"))),
                )
                .json(body),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Region settings
    // -------------------------------------------------------------------------

    /// Get region settings for an application.
    ///
    /// `GET /apps/{appId}/region-settings`
    pub async fn get_region_settings(&self, app_id: &str) -> Result<RegionSettings> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/apps/{app_id}/region-settings"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update region settings for an application.
    ///
    /// `PUT /apps/{appId}/region-settings`
    pub async fn update_region_settings(
        &self,
        app_id: &str,
        body: &UpdateRegionSettingsRequest,
    ) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/apps/{app_id}/region-settings"))),
                )
                .json(body),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Container templates
    // -------------------------------------------------------------------------

    /// Add a container template to an application.
    ///
    /// `POST /apps/{appId}/containers`
    pub async fn add_container(
        &self,
        app_id: &str,
        body: &AddContainerRequest,
    ) -> Result<ContainerTemplate> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/apps/{app_id}/containers"))),
                )
                .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get a container template.
    ///
    /// `GET /apps/{appId}/containers/{containerId}`
    pub async fn get_container(
        &self,
        app_id: &str,
        container_id: &str,
    ) -> Result<ContainerTemplate> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/apps/{app_id}/containers/{container_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Partially update a container template.
    ///
    /// `PATCH /apps/{appId}/containers/{containerId}`
    pub async fn patch_container(
        &self,
        app_id: &str,
        container_id: &str,
        body: &PatchContainerRequest,
    ) -> Result<ContainerTemplate> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .patch(self.url(&format!("/apps/{app_id}/containers/{container_id}"))),
                )
                .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Delete a container template.
    ///
    /// `DELETE /apps/{appId}/containers/{containerId}`
    pub async fn delete_container(&self, app_id: &str, container_id: &str) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .delete(self.url(&format!("/apps/{app_id}/containers/{container_id}"))),
                ),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    /// Set environment variables on a container (full replace).
    ///
    /// `PUT /apps/{appId}/containers/{containerId}/env`
    pub async fn set_container_env(
        &self,
        app_id: &str,
        container_id: &str,
        env: &HashMap<String, String>,
    ) -> Result<ContainerTemplate> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/apps/{app_id}/containers/{container_id}/env"))),
                )
                .json(env),
            )
            .await?;
        self.handle_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Endpoints (networking)
    // -------------------------------------------------------------------------

    /// List all endpoints for an application.
    ///
    /// `GET /apps/{appId}/endpoints`
    pub async fn list_endpoints(&self, app_id: &str) -> Result<CursorList<EndpointListItem>> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/apps/{app_id}/endpoints"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Add an endpoint to a container.
    ///
    /// `POST /apps/{appId}/containers/{containerId}/endpoints`
    pub async fn add_endpoint(
        &self,
        app_id: &str,
        container_id: &str,
        body: &EndpointRequest,
    ) -> Result<SaveEndpointResponse> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url(&format!(
                    "/apps/{app_id}/containers/{container_id}/endpoints"
                ))))
                .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update an endpoint.
    ///
    /// `PUT /apps/{appId}/endpoints/{endpointId}`
    pub async fn update_endpoint(
        &self,
        app_id: &str,
        endpoint_id: &str,
        body: &EndpointRequest,
    ) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/apps/{app_id}/endpoints/{endpoint_id}"))),
                )
                .json(body),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    /// Delete an endpoint.
    ///
    /// `DELETE /apps/{appId}/endpoints/{endpointId}`
    pub async fn delete_endpoint(&self, app_id: &str, endpoint_id: &str) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .delete(self.url(&format!("/apps/{app_id}/endpoints/{endpoint_id}"))),
                ),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Volumes
    // -------------------------------------------------------------------------

    /// List volumes for an application.
    ///
    /// `GET /apps/{appId}/volumes`
    pub async fn list_volumes(&self, app_id: &str) -> Result<ListVolumesResponse> {
        let resp = self
            .execute(self.auth(self.http.get(self.url(&format!("/apps/{app_id}/volumes")))))
            .await?;
        self.handle_response(resp).await
    }

    /// Partially update a volume (name and/or size).
    ///
    /// `PATCH /apps/{appId}/volumes/{volumeId}`
    pub async fn update_volume(
        &self,
        app_id: &str,
        volume_id: &str,
        body: &PatchVolumeRequest,
    ) -> Result<UpdateVolumeResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .patch(self.url(&format!("/apps/{app_id}/volumes/{volume_id}"))),
                )
                .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Detach a volume from all pods.
    ///
    /// `POST /apps/{appId}/volumes/{volumeId}/detach`
    pub async fn detach_volume(
        &self,
        app_id: &str,
        volume_id: &str,
    ) -> Result<DetachVolumeResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/apps/{app_id}/volumes/{volume_id}/detach"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Delete all instances of a volume.
    ///
    /// `DELETE /apps/{appId}/volumes/{volumeId}`
    pub async fn delete_all_volume_instances(
        &self,
        app_id: &str,
        volume_id: &str,
    ) -> Result<DeleteAllVolumeInstancesResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .delete(self.url(&format!("/apps/{app_id}/volumes/{volume_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Delete a single volume instance.
    ///
    /// `DELETE /apps/{appId}/volumes/{volumeId}/instances/{instanceId}`
    pub async fn delete_volume_instance(
        &self,
        app_id: &str,
        volume_id: &str,
        instance_id: &str,
    ) -> Result<DeleteVolumeInstanceResponse> {
        let resp = self
            .execute(self.auth(self.http.delete(self.url(&format!(
                "/apps/{app_id}/volumes/{volume_id}/instances/{instance_id}"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Container registries
    // -------------------------------------------------------------------------

    /// List all container registries.
    ///
    /// `GET /registries`
    pub async fn list_registries(&self) -> Result<CursorList<ContainerRegistry>> {
        let resp = self
            .execute(self.auth(self.http.get(self.url("/registries"))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get a container registry by ID.
    ///
    /// `GET /registries/{registryId}`
    pub async fn get_registry(&self, registry_id: i64) -> Result<ContainerRegistry> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/registries/{registry_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Create a new container registry.
    ///
    /// `POST /registries`
    pub async fn add_registry(
        &self,
        body: &ContainerRegistryRequest,
    ) -> Result<SaveContainerRegistryResult> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update a container registry.
    ///
    /// `PUT /registries/{registryId}`
    pub async fn update_registry(
        &self,
        registry_id: i64,
        body: &ContainerRegistryRequest,
    ) -> Result<SaveContainerRegistryResult> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/registries/{registry_id}"))),
                )
                .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Delete a container registry.
    ///
    /// `DELETE /registries/{registryId}`
    pub async fn delete_registry(&self, registry_id: i64) -> Result<RemoveContainerRegistryResult> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .delete(self.url(&format!("/registries/{registry_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// List container images in a registry.
    ///
    /// `POST /registries/images`
    pub async fn list_container_images(
        &self,
        body: &ListContainerImagesRequest,
    ) -> Result<Vec<ContainerImage>> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries/images")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get the image configuration (labels, exposed ports, entrypoint) for a
    /// container image tag.
    ///
    /// `POST /registries/image-config`
    ///
    /// The bunny.net spec documents this operation without a response schema,
    /// so the raw JSON is returned as a [`serde_json::Value`]. Live-verify the
    /// shape before adding a typed model.
    pub async fn get_image_config(
        &self,
        body: &GetContainerImageDigestRequest,
    ) -> Result<serde_json::Value> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries/image-config")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// List tags for a container image.
    ///
    /// `POST /registries/tags`
    pub async fn list_container_image_tags(
        &self,
        body: &ListContainerImageTagsRequest,
    ) -> Result<Vec<ContainerImageTag>> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries/tags")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get the digest for a container image tag.
    ///
    /// `POST /registries/digest`
    pub async fn get_container_image_digest(
        &self,
        body: &GetContainerImageDigestRequest,
    ) -> Result<ImageTagInfo> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries/digest")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get configuration suggestions for a container image.
    ///
    /// `POST /registries/config-suggestions`
    pub async fn get_config_suggestions(
        &self,
        body: &GetContainerConfigSuggestionsRequest,
    ) -> Result<ContainerConfigSuggestions> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries/config-suggestions")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Search public container images.
    ///
    /// `POST /registries/public-images/search`
    pub async fn search_public_images(
        &self,
        body: &SearchPublicContainerImagesRequest,
    ) -> Result<Vec<ContainerImage>> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/registries/public-images/search")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Regions & nodes
    // -------------------------------------------------------------------------

    /// List available regions.
    ///
    /// `GET /regions`
    pub async fn list_regions(
        &self,
        next_cursor: Option<&str>,
        limit: Option<i32>,
    ) -> Result<CursorList<Region>> {
        let mut req = self.auth(self.http.get(self.url("/regions")));
        if let Some(c) = next_cursor {
            req = req.query(&[("nextCursor", c)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        let resp = self.execute(req).await?;
        self.handle_response(resp).await
    }

    /// Get the optimal base region.
    ///
    /// `GET /regions/optimal`
    pub async fn get_optimal_region(
        &self,
        cdn_server_token: Option<&str>,
    ) -> Result<OptimalBaseRegionResponse> {
        let mut req = self.auth(self.http.get(self.url("/regions/optimal")));
        if let Some(t) = cdn_server_token {
            req = req.query(&[("cdnServerToken", t)]);
        }
        let resp = self.execute(req).await?;
        self.handle_response(resp).await
    }

    /// List nodes (IP addresses).
    ///
    /// `GET /nodes`
    pub async fn list_nodes(
        &self,
        next_cursor: Option<&str>,
        limit: Option<i32>,
    ) -> Result<CursorList<String>> {
        let mut req = self.auth(self.http.get(self.url("/nodes")));
        if let Some(c) = next_cursor {
            req = req.query(&[("nextCursor", c)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        let resp = self.execute(req).await?;
        self.handle_response(resp).await
    }

    /// List node IP addresses in plain form.
    ///
    /// `GET /nodes/plain`
    ///
    /// This spec-only operation has no documented response schema, so the raw
    /// JSON is returned as a [`serde_json::Value`]. Live-verify before adding a
    /// typed model.
    pub async fn list_nodes_plain(&self) -> Result<serde_json::Value> {
        let resp = self
            .execute(self.auth(self.http.get(self.url("/nodes/plain"))))
            .await?;
        self.handle_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Pods
    // -------------------------------------------------------------------------

    /// Recreate (restart) a single pod.
    ///
    /// `POST /apps/{appId}/pods/{podId}/recreate`
    pub async fn recreate_pod(&self, app_id: &str, pod_id: &str) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .post(self.url(&format!("/apps/{app_id}/pods/{pod_id}/recreate"))),
                ),
            )
            .await?;
        self.handle_empty_response(resp).await
    }

    // -------------------------------------------------------------------------
    // User limits
    // -------------------------------------------------------------------------

    /// Get account limits for Magic Containers.
    ///
    /// `GET /limits`
    pub async fn get_user_limits(&self) -> Result<UserLimits> {
        let resp = self
            .execute(self.auth(self.http.get(self.url("/limits"))))
            .await?;
        self.handle_response(resp).await
    }

    // -------------------------------------------------------------------------
    // Log forwarding
    // -------------------------------------------------------------------------

    /// List all log forwarding configurations.
    ///
    /// `GET /log/forwarding`
    pub async fn list_log_forwarding(&self) -> Result<ListLogForwardingResponse> {
        let resp = self
            .execute(self.auth(self.http.get(self.url("/log/forwarding"))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get a log forwarding configuration by app ID.
    ///
    /// `GET /log/forwarding/{appId}`
    pub async fn get_log_forwarding(&self, app_id: &str) -> Result<LogForwardingConfiguration> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .get(self.url(&format!("/log/forwarding/{app_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Create a log forwarding configuration.
    ///
    /// `POST /log/forwarding`
    pub async fn create_log_forwarding(
        &self,
        body: &LogForwardingRequest,
    ) -> Result<LogForwardingConfiguration> {
        let resp = self
            .execute(
                self.auth(self.http.post(self.url("/log/forwarding")))
                    .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update a log forwarding configuration.
    ///
    /// `PUT /log/forwarding/{appId}`
    pub async fn update_log_forwarding(
        &self,
        app_id: &str,
        body: &LogForwardingRequest,
    ) -> Result<LogForwardingConfiguration> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .put(self.url(&format!("/log/forwarding/{app_id}"))),
                )
                .json(body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Delete a log forwarding configuration.
    ///
    /// `DELETE /log/forwarding/{appId}`
    pub async fn delete_log_forwarding(&self, app_id: &str) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.http
                        .delete(self.url(&format!("/log/forwarding/{app_id}"))),
                ),
            )
            .await?;
        self.handle_empty_response(resp).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new_uses_default_base_url() {
        let client = ContainersClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn client_with_base_url_overrides() {
        let client = ContainersClient::with_base_url("key", "http://localhost:9000");
        assert_eq!(client.url("/apps"), "http://localhost:9000/apps");
    }

    #[test]
    fn client_debug_defaults_false() {
        let client = ContainersClient::new("key");
        assert!(!client.debug);
    }

    #[test]
    fn client_with_debug_sets_flag() {
        let client = ContainersClient::new("key").with_debug(true);
        assert!(client.debug);
    }

    #[test]
    fn auth_sets_access_key_header() {
        let client = ContainersClient::new("secret-key");
        let rb = client.auth(client.http.get("http://localhost"));
        let req = rb.build().unwrap();
        let access_key = req.headers().get("AccessKey").unwrap().to_str().unwrap();
        assert_eq!(access_key, "secret-key");
    }
}
