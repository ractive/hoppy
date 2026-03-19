use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

/// Script type: DNS, CDN, or Middleware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ScriptType {
    Dns = 0,
    Cdn = 1,
    Middleware = 2,
}

/// Release lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ReleaseStatus {
    Archived = 0,
    Live = 1,
}

/// A pull zone linked to an edge script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LinkedPullZone {
    pub id: i64,
    pub pull_zone_name: Option<String>,
    pub default_hostname: Option<String>,
}

/// Deploy configuration for a source-code integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeployConfiguration {
    pub branch: Option<String>,
    pub install_command: Option<String>,
    pub build_command: Option<String>,
    pub entry_file: Option<String>,
    pub create_workflow: bool,
}

/// Repository settings for a source-code integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SourceCodeRepositorySettings {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub private: bool,
    pub template_url: Option<String>,
}

/// A source-code integration (e.g. GitHub) attached to a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SourceCodeIntegration {
    pub integration_id: i64,
    pub repository_settings: Option<SourceCodeRepositorySettings>,
    pub deploy_configuration: Option<DeployConfiguration>,
}

/// An environment variable defined on a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeScriptVariable {
    pub id: i64,
    pub name: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

/// The main edge script model returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeScript {
    pub id: i64,
    pub name: Option<String>,
    pub last_modified: String,
    pub script_type: ScriptType,
    pub current_release_id: i64,
    pub edge_script_variables: Option<Vec<EdgeScriptVariable>>,
    pub deleted: bool,
    pub linked_pull_zones: Option<Vec<LinkedPullZone>>,
    pub integration: Option<SourceCodeIntegration>,
    pub default_hostname: Option<String>,
    pub system_hostname: Option<String>,
    #[serde(skip_serializing)]
    pub deployment_key: Option<String>,
    pub repository_id: Option<i64>,
    pub integration_id: Option<i64>,
    pub monthly_cost: f64,
    pub monthly_request_count: i64,
    pub monthly_cpu_time: i64,
}

/// Request body for creating a new edge script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateEdgeScript {
    pub name: Option<String>,
    pub code: Option<String>,
    pub script_type: ScriptType,
    pub create_linked_pull_zone: bool,
    pub linked_pull_zone_name: Option<String>,
    pub integration: Option<SourceCodeIntegration>,
}

/// Request body for updating an existing edge script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateEdgeScript {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_type: Option<ScriptType>,
}

/// The source code of a script, as returned by GET /compute/script/{id}/code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeScriptCode {
    pub code: Option<String>,
    pub last_modified: String,
}

/// Request body for updating a script's source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateEdgeScriptCode {
    pub code: Option<String>,
}

/// A published release of an edge script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeScriptRelease {
    pub id: i64,
    pub deleted: bool,
    pub code: Option<String>,
    pub uuid: Option<String>,
    pub note: Option<String>,
    pub author: Option<String>,
    pub author_email: Option<String>,
    pub commit_sha: Option<String>,
    pub status: ReleaseStatus,
    pub date_released: String,
    pub date_published: String,
}

/// Request body for publishing a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublishScript {
    pub note: Option<String>,
}

/// A secret attached to an edge script (value is never returned by the API).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeScriptSecret {
    pub id: i64,
    pub name: Option<String>,
    pub last_modified: String,
}

/// Response containing the full list of secrets for a script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SecretList {
    pub secrets: Option<Vec<EdgeScriptSecret>>,
}

/// Request body for adding a secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddSecret {
    pub name: String,
    pub secret: Option<String>,
}

/// Request body for updating an existing secret's value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateSecret {
    pub secret: Option<String>,
}

/// Request body for upserting a secret (create or update by name).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpsertSecret {
    pub name: Option<String>,
    pub secret: Option<String>,
}

/// Request body for adding a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AddVariable {
    pub name: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// Request body for updating a variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateVariable {
    pub default_value: Option<String>,
    pub required: Option<bool>,
}

/// Request body for upserting a variable (create or update by name).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpsertVariable {
    pub name: String,
    pub required: Option<bool>,
    pub default_value: Option<String>,
}

/// Usage and cost statistics for an edge script.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct EdgeScriptStatistics {
    pub total_requests_served: i64,
    pub total_cpu_used: f64,
    pub total_monthly_cost: f64,
    pub average_cpu_time_per_execution: f64,
    pub requests_served_chart: Option<HashMap<String, f64>>,
    pub average_cpu_time_chart: Option<HashMap<String, f64>>,
    pub total_cpu_time_chart: Option<HashMap<String, f64>>,
}

pub(crate) fn deserialize_null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Generic pagination wrapper used by list endpoints.
///
/// `items` defaults to an empty `Vec` when the API returns `null` or omits the key.
// NOTE: This `PaginatedList` is intentionally separate from the one in
// `bunny-api-core`. The Compute API can return `null` for the `Items` key,
// which requires a custom deserializer (`deserialize_null_as_empty_vec`).
// Core's version assumes `Items` is always an array. These differences are
// meaningful and the types are kept separate to avoid a shared-crate dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[serde(bound(deserialize = "T: for<'a> serde::Deserialize<'a>"))]
pub struct PaginatedList<T> {
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub items: Vec<T>,
    pub current_page: i32,
    pub total_items: i32,
    pub has_more_items: bool,
}

/// Error body returned by the API on 4xx responses.
///
/// NOTE: Intentionally separate from `bunny-api-core::ApiError` — the Compute
/// API returns `Option` fields whereas the Core API returns non-optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiError {
    pub error_key: Option<String>,
    pub field: Option<String>,
    pub message: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.message, &self.error_key) {
            (Some(msg), _) => write!(f, "{msg}"),
            (None, Some(key)) => write!(f, "API error: {key}"),
            (None, None) => write!(f, "unknown API error"),
        }
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_type_roundtrip() {
        for variant in [ScriptType::Dns, ScriptType::Cdn, ScriptType::Middleware] {
            let json = serde_json::to_string(&variant).unwrap();
            let decoded: ScriptType = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, variant);
        }
        // Dns = 0, Cdn = 1, Middleware = 2
        assert_eq!(serde_json::to_string(&ScriptType::Dns).unwrap(), "0");
        assert_eq!(serde_json::to_string(&ScriptType::Cdn).unwrap(), "1");
        assert_eq!(serde_json::to_string(&ScriptType::Middleware).unwrap(), "2");
    }

    #[test]
    fn create_edge_script_serializes() {
        let body = CreateEdgeScript {
            name: Some("my-script".to_owned()),
            code: None,
            script_type: ScriptType::Cdn,
            create_linked_pull_zone: false,
            linked_pull_zone_name: None,
            integration: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"Name\":\"my-script\""));
        assert!(json.contains("\"ScriptType\":1"));
        assert!(json.contains("\"CreateLinkedPullZone\":false"));
    }

    #[test]
    fn paginated_list_handles_null_items() {
        // When Items is null in the JSON, it should deserialize to an empty Vec.
        let json = r#"{"Items":null,"CurrentPage":1,"TotalItems":0,"HasMoreItems":false}"#;
        let list: PaginatedList<EdgeScript> = serde_json::from_str(json).unwrap();
        assert!(list.items.is_empty());
        assert_eq!(list.current_page, 1);
        assert_eq!(list.total_items, 0);
    }

    #[test]
    fn paginated_list_missing_items_key() {
        // When Items key is absent entirely, it should also default to empty Vec.
        let json = r#"{"CurrentPage":2,"TotalItems":5,"HasMoreItems":true}"#;
        let list: PaginatedList<EdgeScript> = serde_json::from_str(json).unwrap();
        assert!(list.items.is_empty());
        assert_eq!(list.current_page, 2);
    }

    #[test]
    fn api_error_implements_std_error() {
        let err = ApiError {
            error_key: Some("auth.invalid".to_owned()),
            field: None,
            message: Some("Unauthorized".to_owned()),
        };
        // Verify it satisfies std::error::Error (compile-time check via trait object).
        let _: &dyn std::error::Error = &err;
        assert_eq!(err.to_string(), "Unauthorized");
    }
}
