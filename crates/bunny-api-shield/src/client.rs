use anyhow::{Context, Result, bail};
use reqwest::{Client, RequestBuilder, Response, StatusCode};

use crate::types::{
    AccessListsDetailsResponse, BotDetectionConfigurationResponse, CreateCustomAccessList,
    CreateCustomWafRule, CreateRateLimitRule, CreateShieldZoneRequest, CustomAccessList,
    CustomAccessListResponse, CustomWafRule, GetCustomWafRulesResponse, GetRateLimitRulesResponse,
    GetShieldZoneResponse, GetShieldZonesResponse, ProblemDetails, RateLimitRule,
    ShieldZoneResponse, UpdateAccessListConfiguration, UpdateBotDetection,
    UpdateBotDetectionResponse, UpdateCustomAccessList, UpdateCustomWafRule, UpdateRateLimitRule,
    UpdateShieldZoneRequest, WafProfileMinimal,
};

const BASE_URL: &str = "https://api.bunny.net";

/// Client for the bunny.net Shield (WAF/Security) API.
///
/// All methods are `async` and return `anyhow::Result<T>`. HTTP errors are
/// surfaced either as [`ProblemDetails`] (when the server returns a
/// problem+json body) or as a plain anyhow error with the status code.
#[derive(Debug, Clone)]
pub struct ShieldClient {
    client: Client,
    api_key: String,
    base_url: String,
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
            base_url: base_url.into(),
        }
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

    async fn handle_response<T: serde::de::DeserializeOwned>(&self, resp: Response) -> Result<T> {
        let status = resp.status();
        if status.is_success() {
            let body = resp
                .json::<T>()
                .await
                .context("failed to decode success response")?;
            return Ok(body);
        }

        // Try to extract RFC 7807 problem details from the error body.
        if let Ok(problem) = resp.json::<ProblemDetails>().await {
            bail!(problem);
        }

        bail!("Shield API returned status {status}");
    }

    async fn handle_empty_response(&self, resp: Response) -> Result<()> {
        let status = resp.status();
        if status.is_success() || status == StatusCode::NO_CONTENT {
            return Ok(());
        }
        if let Ok(problem) = resp.json::<ProblemDetails>().await {
            bail!(problem);
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
            .auth(
                self.client
                    .get(self.url(&format!("/shield/shield-zone/{shield_zone_id}"))),
            )
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/get-by-pullzone/{pull_zone_id}"
            ))))
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.get(self.url("/shield/shield-zones")))
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.post(self.url("/shield/shield-zone")))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.patch(self.url("/shield/shield-zone")))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(
                self.client
                    .get(self.url(&format!("/shield/waf/custom-rules/{shield_zone_id}"))),
            )
            .send()
            .await
            .context("request failed")?;

        let response: GetCustomWafRulesResponse = self.handle_response(resp).await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Get a single custom WAF rule by its ID.
    ///
    /// `GET /shield/waf/custom-rule/{id}`
    pub async fn get_waf_rule(&self, id: i64) -> Result<CustomWafRule> {
        let resp = self
            .auth(
                self.client
                    .get(self.url(&format!("/shield/waf/custom-rule/{id}"))),
            )
            .send()
            .await
            .context("request failed")?;

        self.handle_response(resp).await
    }

    /// Create a new custom WAF rule.
    ///
    /// `POST /shield/waf/custom-rule`
    pub async fn create_waf_rule(&self, body: CreateCustomWafRule) -> Result<CustomWafRule> {
        let resp = self
            .auth(self.client.post(self.url("/shield/waf/custom-rule")))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(
                self.client
                    .patch(self.url(&format!("/shield/waf/custom-rule/{id}"))),
            )
            .json(&body)
            .send()
            .await
            .context("request failed")?;

        self.handle_response(resp).await
    }

    /// Delete a custom WAF rule.
    ///
    /// `DELETE /shield/waf/custom-rule/{id}`
    pub async fn delete_waf_rule(&self, id: i64) -> Result<()> {
        let resp = self
            .auth(
                self.client
                    .delete(self.url(&format!("/shield/waf/custom-rule/{id}"))),
            )
            .send()
            .await
            .context("request failed")?;

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
            .auth(
                self.client
                    .get(self.url(&format!("/shield/rate-limits/{shield_zone_id}"))),
            )
            .send()
            .await
            .context("request failed")?;

        let response: GetRateLimitRulesResponse = self.handle_response(resp).await?;
        Ok(response.data.unwrap_or_default())
    }

    /// Get a single rate limit rule by its ID.
    ///
    /// `GET /shield/rate-limit/{id}`
    pub async fn get_rate_limit_rule(&self, id: i64) -> Result<RateLimitRule> {
        let resp = self
            .auth(
                self.client
                    .get(self.url(&format!("/shield/rate-limit/{id}"))),
            )
            .send()
            .await
            .context("request failed")?;

        self.handle_response(resp).await
    }

    /// Create a new rate limit rule.
    ///
    /// `POST /shield/rate-limit`
    pub async fn create_rate_limit_rule(&self, body: CreateRateLimitRule) -> Result<RateLimitRule> {
        let resp = self
            .auth(self.client.post(self.url("/shield/rate-limit")))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(
                self.client
                    .patch(self.url(&format!("/shield/rate-limit/{id}"))),
            )
            .json(&body)
            .send()
            .await
            .context("request failed")?;

        self.handle_response(resp).await
    }

    /// Delete a rate limit rule.
    ///
    /// `DELETE /shield/rate-limit/{id}`
    pub async fn delete_rate_limit_rule(&self, id: i64) -> Result<()> {
        let resp = self
            .auth(
                self.client
                    .delete(self.url(&format!("/shield/rate-limit/{id}"))),
            )
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists"
            ))))
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/{id}"
            ))))
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.post(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists"
            ))))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.patch(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/{id}"
            ))))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.patch(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/configurations/{id}"
            ))))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

        self.handle_empty_response(resp).await
    }

    /// Delete a custom access list.
    ///
    /// `DELETE /shield/shield-zone/{shieldZoneId}/access-lists/{id}`
    pub async fn delete_access_list(&self, shield_zone_id: i64, id: i64) -> Result<()> {
        let resp = self
            .auth(self.client.delete(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/access-lists/{id}"
            ))))
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.get(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/bot-detection"
            ))))
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.patch(self.url(&format!(
                "/shield/shield-zone/{shield_zone_id}/bot-detection"
            ))))
            .json(&body)
            .send()
            .await
            .context("request failed")?;

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
            .auth(self.client.get(self.url("/shield/waf/profiles")))
            .send()
            .await
            .context("request failed")?;

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
    fn auth_sets_access_key_header() {
        let client = ShieldClient::new("secret-key");
        let rb = client.auth(client.client.get("http://localhost"));
        let req = rb.build().unwrap();
        let access_key = req.headers().get("AccessKey").unwrap().to_str().unwrap();
        assert_eq!(access_key, "secret-key");
    }
}
