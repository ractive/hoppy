use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result, bail};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::{Client, RequestBuilder, Response, StatusCode};

use crate::recording::{capture_request, maybe_record_response};

/// Encode set for path segments per RFC 3986 §3.3.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'+')
    .add(b'=')
    .add(b'&');

use super::types::{
    AccessListsDetailsResponse, BotDetectionConfigurationResponse, CreateCustomAccessList,
    CreateCustomWafRule, CreateRateLimitRule, CreateShieldZoneRequest, CustomAccessList,
    CustomAccessListResponse, CustomWafRule, GetApiGuardianResponse, GetCustomWafRulesResponse,
    GetRateLimitRulesResponse, GetShieldZonePullzoneMappingResponse, GetShieldZoneResponse,
    GetShieldZonesResponse, GetTriggeredRulesResponse, GetWafEngineConfigResponse,
    GetWafEnumsResponse, GetWafRulesSegmentedByPlanResponse, RateLimitRule,
    ShieldBotDetectionMetricsResponse, ShieldDetailedMetricsResponse, ShieldMetricsResponse,
    ShieldRateLimitMetricsResponse, ShieldRateLimitsMetricsResponse,
    ShieldUploadScanningMetricsResponse, ShieldWafRuleMetricsResponse, ShieldZoneResponse,
    TriggeredRuleRecommendationResponse, UpdateAccessListConfiguration,
    UpdateApiGuardianEndpointRequest, UpdateApiGuardianEndpointResponse, UpdateApiGuardianRequest,
    UpdateApiGuardianResponse, UpdateBotDetection, UpdateBotDetectionResponse,
    UpdateCustomAccessList, UpdateCustomWafRule, UpdateRateLimitRule,
    UpdateReviewTriggeredRuleRequest, UpdateReviewTriggeredRuleResponse, UpdateShieldZoneRequest,
    UpdateUploadScanningConfigurationRequest, UpdateUploadScanningConfigurationResponse,
    UploadOpenApiSpecificationRequest, UploadOpenApiSpecificationResponse,
    UploadScanningConfigurationResponse, WafLoggingResponse, WafProfileMinimal, parse_shield_error,
};

const BASE_URL: &str = "https://api.bunny.net";

/// Client for the bunny.net Shield (WAF/Security) API.
///
/// All methods are `async` and return `anyhow::Result<T>`. HTTP errors are
/// surfaced as a structured error (nested envelope or RFC 7807 problem details)
/// when the body is parseable, otherwise as a plain anyhow error with the status code.
pub struct ShieldClient {
    client: Client,
    api_key: String,
    base_url: String,
    debug: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl Clone for ShieldClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            api_key: self.api_key.clone(),
            base_url: self.base_url.clone(),
            debug: self.debug,
            record_dir: self.record_dir.clone(),
            last_request: Mutex::new(None),
        }
    }
}

impl std::fmt::Debug for ShieldClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShieldClient")
            .field("base_url", &self.base_url)
            .field("debug", &self.debug)
            .field("record_dir", &self.record_dir)
            .finish()
    }
}

impl ShieldClient {
    /// Create a new `ShieldClient` with the given API key.
    ///
    /// Uses `https://api.bunny.net` as the base URL.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, BASE_URL)
    }

    /// Create a client pointing at a custom base URL (useful for testing against a mock server).
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
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

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn auth(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("AccessKey", &self.api_key)
    }

    /// Build and execute a request, printing method and URL to stderr when debug is enabled.
    async fn execute(&self, rb: RequestBuilder) -> Result<Response> {
        let req = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", req.method(), req.url());
        }
        capture_request(&self.last_request, req.method().as_ref(), req.url().path());
        self.client.execute(req).await.context("request failed")
    }

    /// Read the response body, logging status and body when debug is enabled.
    async fn read_body(&self, resp: Response) -> Result<(StatusCode, bytes::Bytes)> {
        let status = resp.status();
        let bytes = resp.bytes().await.context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!("<<< {}", String::from_utf8_lossy(&bytes));
        }
        maybe_record_response(
            self.record_dir.as_deref(),
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

        // Try to extract a structured error from the body (nested envelope or RFC 7807).
        if let Some(err) = parse_shield_error(&bytes) {
            return Err(err);
        }

        bail!("Shield API returned status {status}");
    }

    async fn handle_empty_response(&self, resp: Response) -> Result<()> {
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() || status == StatusCode::NO_CONTENT {
            return Ok(());
        }
        if let Some(err) = parse_shield_error(&bytes) {
            return Err(err);
        }
        bail!("Shield API returned status {status}");
    }

    // -----------------------------------------------------------------------
    // Shield Zones
    // -----------------------------------------------------------------------

    /// Get a Shield Zone by its ID.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}`
    pub async fn get_shield_zone(&self, shield_zone_id: i64) -> Result<ShieldZoneResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/shield-zone/{shield_zone_id}"))),
                ),
            )
            .await?;

        let wrapper: GetShieldZoneResponse = self.handle_response(resp).await?;
        wrapper
            .data
            .ok_or_else(|| anyhow::anyhow!("response contained no data"))
    }

    /// Get a Shield Zone by the Pull Zone it is attached to.
    ///
    /// `GET /shield/shield-zone/get-by-pullzone/{pullZoneId}`
    pub async fn get_shield_zone_by_pull_zone(
        &self,
        pull_zone_id: i64,
    ) -> Result<ShieldZoneResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/get-by-pullzone/{pull_zone_id}"
            )))))
            .await?;

        let wrapper: GetShieldZoneResponse = self.handle_response(resp).await?;
        wrapper
            .data
            .ok_or_else(|| anyhow::anyhow!("response contained no data"))
    }

    /// List all Shield Zones for the account.
    ///
    /// `GET /shield/shield-zones`
    pub async fn list_shield_zones(&self) -> Result<GetShieldZonesResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url("/shield/shield-zones"))))
            .await?;

        self.handle_response(resp).await
    }

    /// Create a new Shield Zone for a Pull Zone.
    ///
    /// `POST /shield/shield-zone`
    pub async fn create_shield_zone(&self, pull_zone_id: i64) -> Result<ShieldZoneResponse> {
        let body = CreateShieldZoneRequest {
            pull_zone_id,
            shield_zone: None,
        };
        let resp = self
            .execute(
                self.auth(self.client.post(self.url("/shield/shield-zone")))
                    .json(&body),
            )
            .await?;

        // The spec shows CreateShieldZoneResponse wrapping a CreateShieldZoneRequest
        // (unusual echo pattern). We parse it as the response type directly.
        // In practice the meaningful response is the resulting zone — fetch it
        // by re-reading the zone list isn't ideal; the API does return the zone
        // data so we deserialize it as-is.
        let wrapper: serde_json::Value = self.handle_response(resp).await?;
        let zone_val = wrapper
            .get("data")
            .and_then(|d| d.get("shieldZone"))
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // Fall back to just deserializing the outer object as a ShieldZoneResponse
        // if the nested structure isn't present (API may return different shapes).
        if zone_val.is_null() {
            // Try the root
            serde_json::from_value(wrapper).context("failed to parse create shield zone response")
        } else {
            serde_json::from_value(zone_val)
                .context("failed to parse shield zone from create response")
        }
    }

    /// Update an existing Shield Zone's configuration.
    ///
    /// `PATCH /shield/shield-zone`
    pub async fn update_shield_zone(&self, body: UpdateShieldZoneRequest) -> Result<()> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url("/shield/shield-zone")))
                    .json(&body),
            )
            .await?;

        self.handle_empty_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Custom WAF rules
    // -----------------------------------------------------------------------

    /// List custom WAF rules for a Shield Zone.
    ///
    /// `GET /shield/waf/custom-rules/{shieldZoneId}`
    pub async fn list_waf_rules(&self, shield_zone_id: i64) -> Result<Vec<CustomWafRule>> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/waf/custom-rules/{shield_zone_id}"))),
                ),
            )
            .await?;

        let response: GetCustomWafRulesResponse = self.handle_response(resp).await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Get a single custom WAF rule by its ID.
    ///
    /// `GET /shield/waf/custom-rule/{id}`
    pub async fn get_waf_rule(&self, id: i64) -> Result<CustomWafRule> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/waf/custom-rule/{id}"))),
                ),
            )
            .await?;

        self.handle_response(resp).await
    }

    /// Create a new custom WAF rule.
    ///
    /// `POST /shield/waf/custom-rule`
    pub async fn create_waf_rule(&self, body: CreateCustomWafRule) -> Result<CustomWafRule> {
        let resp = self
            .execute(
                self.auth(self.client.post(self.url("/shield/waf/custom-rule")))
                    .json(&body),
            )
            .await?;

        self.handle_response(resp).await
    }

    /// Update an existing custom WAF rule.
    ///
    /// `PATCH /shield/waf/custom-rule/{id}`
    pub async fn update_waf_rule(
        &self,
        id: i64,
        body: UpdateCustomWafRule,
    ) -> Result<CustomWafRule> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .patch(self.url(&format!("/shield/waf/custom-rule/{id}")))
                        .json(&body),
                ),
            )
            .await?;

        self.handle_response(resp).await
    }

    /// Delete a custom WAF rule.
    ///
    /// `DELETE /shield/waf/custom-rule/{id}`
    pub async fn delete_waf_rule(&self, id: i64) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .delete(self.url(&format!("/shield/waf/custom-rule/{id}"))),
                ),
            )
            .await?;

        self.handle_empty_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Rate limit rules
    // -----------------------------------------------------------------------

    /// List rate limit rules for a Shield Zone.
    ///
    /// `GET /shield/rate-limits/{shieldZoneId}`
    pub async fn list_rate_limit_rules(&self, shield_zone_id: i64) -> Result<Vec<RateLimitRule>> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/rate-limits/{shield_zone_id}"))),
                ),
            )
            .await?;

        let response: GetRateLimitRulesResponse = self.handle_response(resp).await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Get a single rate limit rule by its ID.
    ///
    /// `GET /shield/rate-limit/{id}`
    pub async fn get_rate_limit_rule(&self, id: i64) -> Result<RateLimitRule> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/rate-limit/{id}"))),
                ),
            )
            .await?;

        self.handle_response(resp).await
    }

    /// Create a new rate limit rule.
    ///
    /// `POST /shield/rate-limit`
    pub async fn create_rate_limit_rule(&self, body: CreateRateLimitRule) -> Result<RateLimitRule> {
        let resp = self
            .execute(
                self.auth(self.client.post(self.url("/shield/rate-limit")))
                    .json(&body),
            )
            .await?;

        self.handle_response(resp).await
    }

    /// Update an existing rate limit rule.
    ///
    /// `PATCH /shield/rate-limit/{id}`
    pub async fn update_rate_limit_rule(
        &self,
        id: i64,
        body: UpdateRateLimitRule,
    ) -> Result<RateLimitRule> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .patch(self.url(&format!("/shield/rate-limit/{id}")))
                        .json(&body),
                ),
            )
            .await?;

        self.handle_response(resp).await
    }

    /// Delete a rate limit rule.
    ///
    /// `DELETE /shield/rate-limit/{id}`
    pub async fn delete_rate_limit_rule(&self, id: i64) -> Result<()> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .delete(self.url(&format!("/shield/rate-limit/{id}"))),
                ),
            )
            .await?;

        self.handle_empty_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Access lists
    // -----------------------------------------------------------------------

    /// Get all access lists (managed + custom) for a Shield Zone.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}/access-lists`
    pub async fn get_access_lists(
        &self,
        shield_zone_id: i64,
    ) -> Result<AccessListsDetailsResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists"
            )))))
            .await?;

        self.handle_response(resp).await
    }

    /// Get a specific custom access list.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}/access-lists/{id}`
    pub async fn get_custom_access_list(
        &self,
        shield_zone_id: i64,
        id: i64,
    ) -> Result<CustomAccessList> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/{id}"
            )))))
            .await?;

        let wrapper: CustomAccessListResponse = self.handle_response(resp).await?;
        wrapper
            .data
            .ok_or_else(|| anyhow::anyhow!("response contained no data"))
    }

    /// Create a new custom access list for a Shield Zone.
    ///
    /// `POST /shield/shield-zone/{shieldZoneId}/access-lists`
    pub async fn create_access_list(
        &self,
        shield_zone_id: i64,
        body: CreateCustomAccessList,
    ) -> Result<CustomAccessList> {
        let resp = self
            .execute(
                self.auth(self.client.post(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/access-lists"
                ))))
                .json(&body),
            )
            .await?;

        let wrapper: CustomAccessListResponse = self.handle_response(resp).await?;
        wrapper
            .data
            .ok_or_else(|| anyhow::anyhow!("response contained no data"))
    }

    /// Update a custom access list's content or name.
    ///
    /// `PATCH /shield/shield-zone/{shieldZoneId}/access-lists/{id}`
    pub async fn update_custom_access_list(
        &self,
        shield_zone_id: i64,
        id: i64,
        body: UpdateCustomAccessList,
    ) -> Result<CustomAccessList> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/access-lists/{id}"
                ))))
                .json(&body),
            )
            .await?;

        let wrapper: CustomAccessListResponse = self.handle_response(resp).await?;
        wrapper
            .data
            .ok_or_else(|| anyhow::anyhow!("response contained no data"))
    }

    /// Update the configuration (enabled/action) of an access list.
    ///
    /// `PATCH /shield/shield-zone/{shieldZoneId}/access-lists/configurations/{id}`
    pub async fn update_access_list_configuration(
        &self,
        shield_zone_id: i64,
        id: i64,
        body: UpdateAccessListConfiguration,
    ) -> Result<()> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/access-lists/configurations/{id}"
                ))))
                .json(&body),
            )
            .await?;

        self.handle_empty_response(resp).await
    }

    /// Delete a custom access list.
    ///
    /// `DELETE /shield/shield-zone/{shieldZoneId}/access-lists/{id}`
    pub async fn delete_access_list(&self, shield_zone_id: i64, id: i64) -> Result<()> {
        let resp = self
            .execute(self.auth(self.client.delete(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/{id}"
            )))))
            .await?;

        self.handle_empty_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Bot Detection
    // -----------------------------------------------------------------------

    /// Get the current bot detection configuration for a Shield Zone.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}/bot-detection`
    pub async fn get_bot_detection(
        &self,
        shield_zone_id: i64,
    ) -> Result<BotDetectionConfigurationResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/bot-detection"
            )))))
            .await?;

        self.handle_response(resp).await
    }

    /// Update bot detection settings for a Shield Zone.
    ///
    /// `PATCH /shield/shield-zone/{shieldZoneId}/bot-detection`
    pub async fn update_bot_detection(
        &self,
        shield_zone_id: i64,
        body: UpdateBotDetection,
    ) -> Result<UpdateBotDetectionResponse> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/bot-detection"
                ))))
                .json(&body),
            )
            .await?;

        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // WAF Profiles
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Metrics endpoints
    // -----------------------------------------------------------------------

    /// Get a metrics overview for a Shield Zone.
    ///
    /// `GET /shield/metrics/overview/{shieldZoneId}`
    pub async fn get_metrics_overview(&self, shield_zone_id: i64) -> Result<ShieldMetricsResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/metrics/overview/{shield_zone_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get detailed metrics for a Shield Zone.
    ///
    /// `GET /shield/metrics/overview/{shieldZoneId}/detailed`
    pub async fn get_metrics_detailed(
        &self,
        shield_zone_id: i64,
    ) -> Result<ShieldDetailedMetricsResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/metrics/overview/{shield_zone_id}/detailed"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get rate limit metrics for all rules in a Shield Zone.
    ///
    /// `GET /shield/metrics/rate-limits/{shieldZoneId}`
    pub async fn get_metrics_rate_limits(
        &self,
        shield_zone_id: i64,
    ) -> Result<ShieldRateLimitsMetricsResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/metrics/rate-limits/{shield_zone_id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get rate limit metrics for a single rule.
    ///
    /// `GET /shield/metrics/rate-limit/{id}`
    pub async fn get_metrics_rate_limit(&self, id: i64) -> Result<ShieldRateLimitMetricsResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url(&format!("/shield/metrics/rate-limit/{id}"))),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get metrics for a single WAF rule in a Shield Zone.
    ///
    /// `GET /shield/metrics/shield-zone/{shieldZoneId}/waf-rule/{ruleId}`
    pub async fn get_metrics_waf_rule(
        &self,
        shield_zone_id: i64,
        rule_id: i64,
    ) -> Result<ShieldWafRuleMetricsResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/metrics/shield-zone/{shield_zone_id}/waf-rule/{rule_id}"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get bot detection metrics for a Shield Zone.
    ///
    /// `GET /shield/metrics/shield-zone/{shieldZoneId}/bot-detection`
    pub async fn get_metrics_bot_detection(
        &self,
        shield_zone_id: i64,
    ) -> Result<ShieldBotDetectionMetricsResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/metrics/shield-zone/{shield_zone_id}/bot-detection"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get upload scanning metrics for a Shield Zone.
    ///
    /// `GET /shield/metrics/shield-zone/{shieldZoneId}/upload-scanning`
    pub async fn get_metrics_upload_scanning(
        &self,
        shield_zone_id: i64,
    ) -> Result<ShieldUploadScanningMetricsResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/metrics/shield-zone/{shield_zone_id}/upload-scanning"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // API Guardian
    // -----------------------------------------------------------------------

    /// Get the API Guardian configuration for a Shield Zone.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}/api-guardian`
    pub async fn get_api_guardian(&self, shield_zone_id: i64) -> Result<GetApiGuardianResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/api-guardian"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    /// Upload a new OpenAPI specification to API Guardian.
    ///
    /// `POST /shield/shield-zone/{shieldZoneId}/api-guardian`
    pub async fn upload_api_guardian_spec(
        &self,
        shield_zone_id: i64,
        body: UploadOpenApiSpecificationRequest,
    ) -> Result<UploadOpenApiSpecificationResponse> {
        let resp = self
            .execute(
                self.auth(self.client.post(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/api-guardian"
                ))))
                .json(&body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update the API Guardian configuration by uploading an updated OpenAPI spec.
    ///
    /// `PATCH /shield/shield-zone/{shieldZoneId}/api-guardian`
    pub async fn update_api_guardian(
        &self,
        shield_zone_id: i64,
        body: UpdateApiGuardianRequest,
    ) -> Result<UpdateApiGuardianResponse> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/api-guardian"
                ))))
                .json(&body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Update an individual API Guardian endpoint configuration.
    ///
    /// `PATCH /shield/shield-zone/{shieldZoneId}/api-guardian/endpoint/{endpointId}`
    pub async fn update_api_guardian_endpoint(
        &self,
        shield_zone_id: i64,
        endpoint_id: i64,
        body: UpdateApiGuardianEndpointRequest,
    ) -> Result<UpdateApiGuardianEndpointResponse> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/api-guardian/endpoint/{endpoint_id}"
                ))))
                .json(&body),
            )
            .await?;
        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Upload Scanning
    // -----------------------------------------------------------------------

    /// Get the upload scanning configuration for a Shield Zone.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}/upload-scanning`
    pub async fn get_upload_scanning(
        &self,
        shield_zone_id: i64,
    ) -> Result<UploadScanningConfigurationResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/upload-scanning"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    /// Update the upload scanning configuration for a Shield Zone.
    ///
    /// `PATCH /shield/shield-zone/{shieldZoneId}/upload-scanning`
    pub async fn update_upload_scanning(
        &self,
        shield_zone_id: i64,
        body: UpdateUploadScanningConfigurationRequest,
    ) -> Result<UpdateUploadScanningConfigurationResponse> {
        let resp = self
            .execute(
                self.auth(self.client.patch(self.url(&format!(
                    "/shield/shield-zone/{shield_zone_id}/upload-scanning"
                ))))
                .json(&body),
            )
            .await?;
        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Event Logs
    // -----------------------------------------------------------------------

    /// Get event logs for a Shield Zone on a given date.
    ///
    /// `GET /shield/event-logs/{shieldZoneId}/{date}/{continuationToken}`
    ///
    /// - `date` must be in `MM-dd-yyyy` format.
    /// - `continuation_token` is required by the path; pass `""` for the first page.
    ///   The token is percent-encoded before being placed in the URL.
    pub async fn get_event_logs(
        &self,
        shield_zone_id: i64,
        date: &str,
        continuation_token: &str,
    ) -> Result<WafLoggingResponse> {
        let encoded_token = utf8_percent_encode(continuation_token, PATH_SEGMENT).to_string();
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/event-logs/{shield_zone_id}/{date}/{encoded_token}"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // WAF Triggered Rules
    // -----------------------------------------------------------------------

    /// List all triggered WAF rules for a Shield Zone.
    ///
    /// `GET /shield/waf/rules/review-triggered/{shieldZoneId}`
    pub async fn get_triggered_waf_rules(
        &self,
        shield_zone_id: i64,
    ) -> Result<GetTriggeredRulesResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/waf/rules/review-triggered/{shield_zone_id}"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    /// Review and update the action for a triggered WAF rule.
    ///
    /// `POST /shield/waf/rules/review-triggered/{shieldZoneId}`
    pub async fn review_triggered_waf_rule(
        &self,
        shield_zone_id: i64,
        body: UpdateReviewTriggeredRuleRequest,
    ) -> Result<UpdateReviewTriggeredRuleResponse> {
        let resp = self
            .execute(
                self.auth(self.client.post(self.url(&format!(
                    "/shield/waf/rules/review-triggered/{shield_zone_id}"
                ))))
                .json(&body),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get an AI recommendation for a triggered WAF rule.
    ///
    /// `GET /shield/waf/rules/review-triggered/ai-recommendation/{shieldZoneId}/{ruleId}`
    pub async fn get_triggered_waf_rule_recommendation(
        &self,
        shield_zone_id: i64,
        rule_id: &str,
    ) -> Result<TriggeredRuleRecommendationResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/waf/rules/review-triggered/ai-recommendation/{shield_zone_id}/{rule_id}"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // Supplementary endpoints
    // -----------------------------------------------------------------------

    /// Get WAF rules segmented by plan tier.
    ///
    /// `GET /shield/waf/rules/plan-segmentation`
    pub async fn get_waf_plan_segmentation(&self) -> Result<GetWafRulesSegmentedByPlanResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url("/shield/waf/rules/plan-segmentation")),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get the WAF engine configuration variables.
    ///
    /// `GET /shield/waf/engine-config`
    pub async fn get_waf_engine_config(&self) -> Result<GetWafEngineConfigResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url("/shield/waf/engine-config"))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get DDoS enum mappings.
    ///
    /// `GET /shield/ddos/enums`
    pub async fn get_ddos_enums(&self) -> Result<GetWafEnumsResponse> {
        let resp = self
            .execute(self.auth(self.client.get(self.url("/shield/ddos/enums"))))
            .await?;
        self.handle_response(resp).await
    }

    /// Get the mapping of Shield Zones to Pull Zones.
    ///
    /// `GET /shield/shield-zones/pullzone-mapping`
    pub async fn get_shield_zones_pullzone_mapping(
        &self,
    ) -> Result<GetShieldZonePullzoneMappingResponse> {
        let resp = self
            .execute(
                self.auth(
                    self.client
                        .get(self.url("/shield/shield-zones/pullzone-mapping")),
                ),
            )
            .await?;
        self.handle_response(resp).await
    }

    /// Get the current promotional state for the account.
    ///
    /// `GET /shield/promo/state`
    pub async fn get_promo_state(&self) -> Result<serde_json::Value> {
        let resp = self
            .execute(self.auth(self.client.get(self.url("/shield/promo/state"))))
            .await?;
        let (status, bytes) = self.read_body(resp).await?;
        if status.is_success() {
            // The spec declares no response body for this endpoint; return the raw value.
            if bytes.is_empty() {
                return Ok(serde_json::Value::Null);
            }
            return serde_json::from_slice(&bytes).context("failed to decode promo state");
        }
        if let Some(err) = parse_shield_error(&bytes) {
            return Err(err);
        }
        anyhow::bail!("Shield API returned status {status}");
    }

    /// Get access list enum types and their values for a Shield Zone.
    ///
    /// `GET /shield/shield-zone/{shieldZoneId}/access-lists/enums`
    pub async fn get_access_list_enums(
        &self,
        shield_zone_id: i64,
    ) -> Result<std::collections::HashMap<String, std::collections::HashMap<String, String>>> {
        let resp = self
            .execute(self.auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/enums"
            )))))
            .await?;
        self.handle_response(resp).await
    }

    // -----------------------------------------------------------------------
    // WAF Profiles
    // -----------------------------------------------------------------------

    /// List available WAF profiles.
    ///
    /// `GET /shield/waf/profiles`
    pub async fn list_waf_profiles(&self) -> Result<Vec<WafProfileMinimal>> {
        let resp = self
            .execute(self.auth(self.client.get(self.url("/shield/waf/profiles"))))
            .await?;

        // The spec returns `GetWafProfilesResponse { data: Vec<Vec<WafProfileMinimal>> }`.
        // We flatten that nested structure for convenience.
        let raw: serde_json::Value = self.handle_response(resp).await?;
        let nested = raw
            .get("data")
            .and_then(|d| d.as_array())
            .cloned()
            .unwrap_or_default();

        let mut profiles = Vec::new();
        for group in nested {
            if let Some(arr) = group.as_array() {
                for item in arr {
                    let p: WafProfileMinimal = serde_json::from_value(item.clone())
                        .context("failed to parse WafProfileMinimal")?;
                    profiles.push(p);
                }
            }
        }
        Ok(profiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_url_construction() {
        let client = ShieldClient::new("test-key");
        assert_eq!(
            client.url("/shield/shield-zone/42"),
            "https://api.bunny.net/shield/shield-zone/42"
        );
    }

    #[test]
    fn client_with_base_url_overrides() {
        let client = ShieldClient::with_base_url("test-key", "http://localhost:8080");
        assert_eq!(
            client.url("/shield/waf/custom-rule/1"),
            "http://localhost:8080/shield/waf/custom-rule/1"
        );
    }

    #[test]
    fn client_stores_api_key() {
        let client = ShieldClient::new("my-api-key-12345");
        assert_eq!(client.api_key, "my-api-key-12345");
    }

    #[test]
    fn client_debug_defaults_false() {
        let client = ShieldClient::new("key");
        assert!(!client.debug);
    }

    #[test]
    fn client_with_debug_sets_flag() {
        let client = ShieldClient::new("key").with_debug(true);
        assert!(client.debug);
    }

    #[test]
    fn auth_sets_access_key_header() {
        let client = ShieldClient::new("secret-key");
        let rb = client.auth(client.client.get("http://localhost"));
        let req = rb.build().unwrap();
        let access_key = req.headers().get("AccessKey").unwrap().to_str().unwrap();
        assert_eq!(access_key, "secret-key");
    }
}
