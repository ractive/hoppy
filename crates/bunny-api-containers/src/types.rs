use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

/// RFC 7807 problem details returned on 4xx/5xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type", default)]
    pub problem_type: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub instance: Option<String>,
}

impl std::fmt::Display for ProblemDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Magic Containers API error {}: {}",
            self.status.unwrap_or(0),
            self.title.as_deref().unwrap_or("unknown"),
        )?;
        if let Some(detail) = &self.detail {
            write!(f, " — {detail}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ProblemDetails {}

/// Structured error body with optional validation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<i32>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub instance: Option<String>,
    #[serde(default)]
    pub errors: Option<Vec<ValidationError>>,
}

impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Magic Containers API error {}: {}",
            self.status.unwrap_or(0),
            self.title.as_deref().unwrap_or("unknown"),
        )?;
        if let Some(detail) = &self.detail {
            write!(f, " — {detail}")?;
        }
        if let Some(errors) = &self.errors {
            for e in errors {
                write!(
                    f,
                    "\n  {}: {}",
                    e.field.as_deref().unwrap_or("?"),
                    e.message
                )?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ErrorDetails {}

/// A single field-level validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    #[serde(default)]
    pub field: Option<String>,
    #[serde(default)]
    pub message: String,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Application lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplicationStatus {
    Unknown,
    Active,
    Progressing,
    Inactive,
    Failing,
    Suspended,
}

/// Application runtime type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeType {
    Shared,
    Reserved,
}

/// Endpoint type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndpointType {
    CDN,
    Anycast,
    PublicIp,
}

/// Region provisioning type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionProvisioningType {
    Static,
    Dynamic,
}

/// Container image pull policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImagePullPolicy {
    Always,
    IfNotPresent,
}

/// Network protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Sctp,
}

/// Anycast IP protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnycastIpProtocolVersion {
    IPv4,
}

/// Overview status grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Grade {
    CouldBeBetter,
    NotBad,
    DoingGreat,
}

/// Deployment status for a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Unknown,
    Active,
    Progressing,
    Inactive,
    Failing,
}

/// Pod lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PodStatus {
    NotScheduled,
    Scheduled,
    Ready,
    Deleting,
}

/// Container status within a pod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    NotStarted,
    Started,
    Ready,
}

/// Statistics granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Granularity {
    Daily,
    Hourly,
    Minute,
}

/// Volume instance status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeInstanceStatus {
    Unknown,
    Attached,
    Detached,
    Extending,
    Deleting,
    Creating,
    NotScheduled,
    Scheduled,
    Failed,
}

/// Container registry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryType {
    DockerHub,
    GitHub,
}

/// Result status after saving a container registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SavedContainerRegistryStatus {
    Saved,
    SecretsValidationFailed,
    UnknownErrorOccured,
    NotFound,
    InvalidInput,
}

/// Result status after removing a container registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoveContainerRegistryStatus {
    NotFound,
    InUse,
    Removed,
}

/// Log forwarding transport type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogForwardingType {
    SyslogUdp,
    SyslogTcp,
}

/// Syslog message format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyslogFormat {
    SyslogRfc3164,
    SyslogRfc5424,
}

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

/// Cursor-based list metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMeta {
    pub total_items: i64,
}

/// Generic cursor-paginated list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: for<'a> serde::Deserialize<'a>"))]
pub struct CursorList<T> {
    #[serde(default = "Vec::new")]
    pub items: Vec<T>,
    #[serde(default)]
    pub meta: Option<ListMeta>,
    #[serde(default)]
    pub cursor: Option<String>,
}

// ---------------------------------------------------------------------------
// Applications
// ---------------------------------------------------------------------------

/// Full application model returned by GET /apps/{appId}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub id: String,
    pub name: String,
    pub status: ApplicationStatus,
    pub runtime_type: RuntimeType,
    pub region_settings: RegionSettings,
    #[serde(default)]
    pub container_templates: Vec<ContainerTemplate>,
    #[serde(default)]
    pub container_instances: Vec<ContainerInstance>,
    #[serde(default)]
    pub volumes: Vec<VolumeTemplate>,
    #[serde(default)]
    pub display_endpoint: Option<DisplayEndpoint>,
    #[serde(default)]
    pub auto_scaling: Option<AutoscalingSettings>,
    #[serde(default)]
    pub network_settings: Option<NetworkLimits>,
    #[serde(default)]
    pub repository_settings: Option<RepositorySettings>,
}

/// Application list item (subset of fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppListItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub display_endpoint: Option<DisplayEndpoint>,
    pub status: ApplicationStatus,
}

/// Display endpoint summary shown on app listings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayEndpoint {
    pub id: String,
    pub address: String,
    #[serde(rename = "type")]
    pub endpoint_type: EndpointType,
}

/// Autoscaling settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoscalingSettings {
    pub min: i32,
    pub max: i32,
}

/// Region settings (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionSettings {
    #[serde(default)]
    pub allowed_region_ids: Vec<String>,
    #[serde(default)]
    pub required_region_ids: Vec<String>,
    #[serde(default)]
    pub max_allowed_regions: Option<i32>,
    #[serde(default)]
    pub provisioning_type: Option<RegionProvisioningType>,
}

/// Region settings update request.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRegionSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_region_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_region_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_allowed_regions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_selectors: Option<HashMap<String, String>>,
}

/// Network bandwidth limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkLimits {
    #[serde(default)]
    pub ingress_bandwidth_limit: Option<i64>,
    #[serde(default)]
    pub egress_bandwidth_limit: Option<i64>,
}

/// Repository settings for source-code integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySettings {
    #[serde(default)]
    pub template_repository: Option<String>,
    #[serde(default)]
    pub repository_name: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

/// Container instance (running container in a pod).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerInstance {
    pub id: String,
    pub template_id: String,
    pub pod_id: String,
    pub node_ip_address: String,
}

/// Request body for creating an application.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddApplicationRequest {
    pub name: String,
    pub runtime_type: RuntimeType,
    pub auto_scaling: AutoscalingSettings,
    pub region_settings: UpdateRegionSettingsRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_grace_period_seconds: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_settings: Option<RepositorySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_templates: Option<Vec<ContainerRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<VolumeRequest>>,
}

/// Request body for partially updating an application.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchApplicationRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_type: Option<RuntimeType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_scaling: Option<AutoscalingSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_settings: Option<UpdateRegionSettingsRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_templates: Option<Vec<ContainerRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<VolumeRequest>>,
}

/// Response from add/update application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddApplicationResponse {
    pub id: String,
}

// ---------------------------------------------------------------------------
// Container templates
// ---------------------------------------------------------------------------

/// Container template (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerTemplate {
    pub id: String,
    pub name: String,
    pub package_id: String,
    pub image: String,
    pub image_name: String,
    pub image_namespace: String,
    pub image_tag: String,
    pub image_registry_id: String,
    pub image_digest: String,
    pub image_pull_policy: ImagePullPolicy,
    pub entry_point: ContainerEntryPoint,
    pub probes: ContainerProbes,
    #[serde(default)]
    pub environment_variables: Vec<EnvironmentVariable>,
    #[serde(default)]
    pub endpoints: Vec<ContainerEndpoint>,
    #[serde(default)]
    pub volume_mounts: Vec<ContainerVolumeMount>,
}

/// Container request (for create/update).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub image_name: String,
    pub image_namespace: String,
    pub image_tag: String,
    pub image_registry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<ImagePullPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<ContainerEntryPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probes: Option<ContainerProbes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<EndpointRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_mounts: Option<Vec<VolumeMountRequest>>,
}

/// Request for adding a container template to an existing app.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddContainerRequest {
    pub name: String,
    pub image_name: String,
    pub image_namespace: String,
    pub image_tag: String,
    pub image_registry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<ImagePullPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<ContainerEntryPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probes: Option<ContainerProbes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<EndpointRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_mounts: Option<Vec<VolumeMountRequest>>,
}

/// Request for partially updating a container template.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchContainerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_registry_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_pull_policy: Option<ImagePullPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<ContainerEntryPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probes: Option<ContainerProbes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_variables: Option<Vec<EnvironmentVariable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<EndpointRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_mounts: Option<Vec<VolumeMountRequest>>,
}

/// Container entry point configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerEntryPoint {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub command_array: Option<Vec<String>>,
    #[serde(default)]
    pub arguments: Option<String>,
    #[serde(default)]
    pub arguments_array: Option<Vec<String>>,
    #[serde(default)]
    pub working_directory: Option<String>,
}

/// Container health probes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerProbes {
    #[serde(default)]
    pub startup: Option<ContainerProbe>,
    #[serde(default)]
    pub readiness: Option<ContainerProbe>,
    #[serde(default)]
    pub liveness: Option<ContainerProbe>,
}

/// A single health probe configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerProbe {
    #[serde(default)]
    pub initial_delay_seconds: Option<i32>,
    #[serde(default)]
    pub period_seconds: Option<i32>,
    #[serde(default)]
    pub timeout_seconds: Option<i32>,
    #[serde(default)]
    pub failure_threshold: Option<i32>,
    #[serde(default)]
    pub success_threshold: Option<i32>,
    #[serde(default)]
    pub http_get: Option<HttpGetProbe>,
    #[serde(default)]
    pub tcp_socket: Option<TcpSocketProbe>,
    #[serde(default)]
    pub grpc: Option<GrpcProbe>,
}

/// HTTP GET probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpGetProbe {
    #[serde(default)]
    pub request: Option<HttpGetProbeRequest>,
    #[serde(default)]
    pub response: Option<HttpGetProbeResponse>,
}

/// HTTP GET probe request details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpGetProbeRequest {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub port_number: Option<i32>,
}

/// HTTP GET probe expected response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpGetProbeResponse {
    #[serde(default)]
    pub expected_status_code: Option<String>,
}

/// TCP socket probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpSocketProbe {
    #[serde(default)]
    pub request: Option<TcpSocketProbeRequest>,
}

/// TCP socket probe request details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TcpSocketProbeRequest {
    #[serde(default)]
    pub port_number: Option<i32>,
}

/// gRPC probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpcProbe {
    #[serde(default)]
    pub request: Option<GrpcProbeRequest>,
}

/// gRPC probe request details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpcProbeRequest {
    #[serde(default)]
    pub port_number: Option<i32>,
    #[serde(default)]
    pub service_name: Option<String>,
}

/// An environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentVariable {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
}

// ---------------------------------------------------------------------------
// Endpoints (networking)
// ---------------------------------------------------------------------------

/// Container endpoint (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerEndpoint {
    pub display_name: String,
    pub public_host: String,
    #[serde(rename = "type")]
    pub endpoint_type: EndpointType,
    pub is_ssl_enabled: bool,
    pub pull_zone_id: String,
    #[serde(default)]
    pub port_mappings: Vec<EndpointPortMapping>,
    #[serde(default)]
    pub sticky_sessions: Option<EndpointStickySession>,
    #[serde(default)]
    pub internal_ip_addresses: Option<Vec<EndpointInternalIp>>,
    #[serde(default)]
    pub public_ip_addresses: Option<Vec<EndpointInternalIp>>,
}

/// Endpoint list item (includes container info).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointListItem {
    pub id: String,
    pub display_name: String,
    pub public_host: String,
    #[serde(rename = "type")]
    pub endpoint_type: EndpointType,
    pub is_ssl_enabled: bool,
    pub pull_zone_id: String,
    #[serde(default)]
    pub port_mappings: Vec<EndpointPortMapping>,
    #[serde(default)]
    pub container_name: Option<String>,
    #[serde(default)]
    pub container_id: Option<String>,
    #[serde(default)]
    pub sticky_sessions: Option<EndpointStickySession>,
    #[serde(default)]
    pub internal_ip_addresses: Option<Vec<EndpointInternalIp>>,
    #[serde(default)]
    pub public_ip_addresses: Option<Vec<EndpointInternalIp>>,
}

/// Port mapping (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointPortMapping {
    pub container_port: i32,
    pub exposed_port: i32,
    #[serde(default)]
    pub protocols: Vec<Protocol>,
}

/// Sticky session settings (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointStickySession {
    pub enabled: bool,
    #[serde(default)]
    pub session_headers: Vec<String>,
    #[serde(default)]
    pub cookie_name: Option<String>,
}

/// Internal/public IP address for an endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointInternalIp {
    pub address: String,
    pub region: String,
}

/// Endpoint request (for create/update).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn: Option<CdnEndpointRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anycast: Option<AnycastEndpointRequest>,
}

/// CDN endpoint request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CdnEndpointRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ssl_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticky_sessions: Option<StickySessionSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_zone_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_mappings: Option<Vec<ContainerPortMappingRequest>>,
}

/// Anycast endpoint request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnycastEndpointRequest {
    #[serde(rename = "type")]
    pub protocol_version: AnycastIpProtocolVersion,
    pub port_mappings: Vec<ContainerPortMappingRequest>,
}

/// Sticky session settings (request).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StickySessionSettings {
    pub session_headers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookie_name: Option<String>,
}

/// Port mapping request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerPortMappingRequest {
    pub container_port: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposed_port: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<Protocol>>,
}

/// Response from add endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveEndpointResponse {
    pub id: String,
}

// ---------------------------------------------------------------------------
// Volumes
// ---------------------------------------------------------------------------

/// Volume template (response, within Application).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeTemplate {
    pub name: String,
    pub size: f64,
}

/// Volume request (for create/update app).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeRequest {
    pub name: String,
    pub size: i32,
}

/// Volume mount request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMountRequest {
    pub name: String,
    pub mount_path: String,
}

/// Volume mount (response).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerVolumeMount {
    pub name: String,
    pub mount_path: String,
}

/// Volume in list response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInList {
    pub name: String,
    pub id: String,
    pub size: f64,
    #[serde(default)]
    pub total_usage: f64,
    #[serde(default)]
    pub total_instances_count: i32,
    #[serde(default)]
    pub attached_instances_count: i32,
    #[serde(default)]
    pub containers_count: i32,
    #[serde(default)]
    pub volume_instances: Vec<VolumeInstance>,
}

/// A single volume instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInstance {
    pub id: String,
    #[serde(default)]
    pub attached_pods: Vec<String>,
    #[serde(default)]
    pub attached_containers: Vec<String>,
    pub region: String,
    pub status: VolumeInstanceStatus,
    pub size: f64,
    #[serde(default)]
    pub usage: f64,
}

/// Volume list summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVolumesSummary {
    #[serde(default)]
    pub total_pods: i32,
    #[serde(default)]
    pub total_containers: i32,
    #[serde(default)]
    pub total_storage: f64,
}

/// Volume list response with summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListVolumesResponse {
    #[serde(default)]
    pub items: Vec<VolumeInList>,
    #[serde(default)]
    pub meta: Option<ListMeta>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub summary: Option<ListVolumesSummary>,
}

/// Patch volume request.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchVolumeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
}

/// Update volume response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVolumeResponse {
    pub name: String,
    pub size: f64,
}

/// Detach volume response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachVolumeResponse {
    pub name: String,
}

/// Delete all volume instances response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAllVolumeInstancesResponse {
    #[serde(default)]
    pub ids: Vec<String>,
}

/// Delete single volume instance response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteVolumeInstanceResponse {
    pub id: String,
}

// ---------------------------------------------------------------------------
// Container registries
// ---------------------------------------------------------------------------

/// Container registry model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerRegistry {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    pub namespace_id: String,
    pub display_name: String,
    pub host_name: String,
    #[serde(default)]
    pub user_name: Option<String>,
    #[serde(default)]
    pub first_password_symbols: Option<String>,
    #[serde(default)]
    pub last_password_symbols: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub last_updated_at: Option<String>,
}

/// Request to create or update a container registry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerRegistryRequest {
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub registry_type: Option<RegistryType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_credentials: Option<RegistryCredentials>,
}

/// Credentials for a container registry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryCredentials {
    pub user_name: String,
    pub password: String,
}

/// Result of saving a container registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveContainerRegistryResult {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub error: Option<String>,
    pub status: SavedContainerRegistryStatus,
}

/// Result of removing a container registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveContainerRegistryResult {
    pub status: RemoveContainerRegistryStatus,
    #[serde(default)]
    pub applications: Option<Vec<String>>,
}

/// Container image (from list/search).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerImage {
    pub id: String,
    pub namespace: String,
}

/// Container image tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerImageTag {
    pub name: String,
}

/// Image tag digest info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageTagInfo {
    #[serde(default)]
    pub image_namespace: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub digest: Option<String>,
}

/// Request for listing container images.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContainerImagesRequest {
    pub registry_id: String,
}

/// Request for listing container image tags.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListContainerImageTagsRequest {
    pub registry_id: String,
    pub image_name: String,
    pub image_namespace: String,
}

/// Request for getting an image digest by tag.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContainerImageDigestRequest {
    pub registry_id: String,
    pub image_name: String,
    pub image_namespace: String,
    pub tag: String,
}

/// Request for config suggestions.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetContainerConfigSuggestionsRequest {
    pub registry_id: String,
    pub image_name: String,
    pub image_namespace: String,
    pub tag: String,
}

/// Container config suggestions response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerConfigSuggestions {
    #[serde(default)]
    pub endpoint_suggestions: Vec<serde_json::Value>,
    #[serde(default)]
    pub environment_variables_suggestions: Vec<EnvironmentVariableSuggestion>,
    #[serde(default)]
    pub app_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub registry_url: Option<String>,
}

/// Environment variable suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentVariableSuggestion {
    pub name: String,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

/// Request for searching public container images.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPublicContainerImagesRequest {
    pub registry_id: String,
    pub prefix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
}

// ---------------------------------------------------------------------------
// Overview
// ---------------------------------------------------------------------------

/// Application overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationOverview {
    #[serde(default)]
    pub target_latency: Option<DoubleStatusIndicator>,
    #[serde(default)]
    pub current_latency: Option<DoubleStatusIndicator>,
    #[serde(default)]
    pub active_regions: Option<Int32StatusIndicator>,
    #[serde(default)]
    pub active_instances: Option<Int32StatusIndicator>,
    #[serde(default)]
    pub desired_instances: Option<i32>,
    #[serde(default)]
    pub status: Option<ApplicationStatus>,
    #[serde(default)]
    pub average_cpu: Option<DoubleStatusIndicator>,
    #[serde(default)]
    pub average_ram: Option<DoubleStatusIndicator>,
    #[serde(default)]
    pub average_volumes_usage: Option<DoubleStatusIndicator>,
    #[serde(default)]
    pub regions: Vec<OverviewRegion>,
    #[serde(default)]
    pub average_latency: Option<f64>,
    #[serde(default)]
    pub total_volume_size_in_gb: Option<f64>,
    #[serde(default)]
    pub monthly_cost: Option<f64>,
    #[serde(default)]
    pub latency_chart: Option<HashMap<String, f64>>,
}

/// Status indicator with a double value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoubleStatusIndicator {
    #[serde(default)]
    pub indicator: f64,
    #[serde(default)]
    pub status_grade: Option<Grade>,
}

/// Status indicator with an integer value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Int32StatusIndicator {
    #[serde(default)]
    pub indicator: i32,
    #[serde(default)]
    pub status_grade: Option<Grade>,
}

/// Region overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewRegion {
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub instances: i32,
    #[serde(default)]
    pub status: Option<DeploymentStatus>,
    #[serde(default)]
    pub average_cpu: f64,
    #[serde(default)]
    pub average_ram: f64,
    #[serde(default)]
    pub average_volumes_usage_percentage: f64,
    #[serde(default)]
    pub requests: f64,
    #[serde(default)]
    pub anycast_traffic: f64,
    #[serde(default)]
    pub pods: Vec<OverviewPod>,
}

/// Pod overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewPod {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<PodStatus>,
    #[serde(default)]
    pub last_heart_beat: Option<String>,
    #[serde(default)]
    pub outbound_traffic_chart: Option<HashMap<String, i64>>,
    #[serde(default)]
    pub cpu_usage: f64,
    #[serde(default)]
    pub ram_usage: f64,
    #[serde(default)]
    pub containers: Vec<OverviewContainer>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub volumes_utilization_percentage: Option<HashMap<String, f64>>,
}

/// Container overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewContainer {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cpu_usage: f64,
    #[serde(default)]
    pub ram_usage: f64,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub status: Option<ContainerStatus>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub image_display: Option<String>,
    #[serde(default)]
    pub number_of_restarts: Option<i32>,
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

/// Application statistics response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationStatistics {
    #[serde(default)]
    pub target_latency_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub active_regions_chart: Option<HashMap<String, i64>>,
    #[serde(default)]
    pub latency_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub instances_chart: Option<HashMap<String, i64>>,
    #[serde(default)]
    pub cpu_usage_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub ram_usage_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub traffic_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub volumes_usage_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub volumes_capacity_chart: Option<HashMap<String, f64>>,
    #[serde(default)]
    pub volumes_split_usage_chart: Option<HashMap<String, HashMap<String, f64>>>,
    #[serde(default)]
    pub volumes_split_capacity_chart: Option<HashMap<String, HashMap<String, f64>>>,
}

// ---------------------------------------------------------------------------
// Regions & nodes
// ---------------------------------------------------------------------------

/// A region.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Region {
    pub id: String,
    pub name: String,
    pub group: String,
    #[serde(default)]
    pub has_anycast_support: bool,
    #[serde(default)]
    pub has_capacity: bool,
}

/// Optimal base region response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimalBaseRegionResponse {
    pub region: Region,
}

// ---------------------------------------------------------------------------
// User limits
// ---------------------------------------------------------------------------

/// Account limits for Magic Containers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLimits {
    pub max_number_of_applications: i32,
    pub existing_number_of_applications: i32,
    #[serde(default)]
    pub max_number_of_regions_per_application: Option<i32>,
    pub max_number_of_instances_per_region: i32,
    #[serde(default)]
    pub max_number_of_instances_per_application: Option<i32>,
    pub max_number_of_volumes_per_application: i32,
    #[serde(default)]
    pub max_volume_size: Option<i32>,
}

// ---------------------------------------------------------------------------
// Log forwarding
// ---------------------------------------------------------------------------

/// Log forwarding configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogForwardingConfiguration {
    pub id: String,
    pub app: String,
    #[serde(default)]
    pub product_id: Option<String>,
    #[serde(rename = "type")]
    pub forwarding_type: LogForwardingType,
    pub endpoint: String,
    pub port: i32,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    pub format: SyslogFormat,
    pub enabled: bool,
}

/// Request for creating/updating a log forwarding configuration.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogForwardingRequest {
    pub app: String,
    #[serde(rename = "type")]
    pub forwarding_type: LogForwardingType,
    pub endpoint: String,
    pub port: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub format: SyslogFormat,
    pub enabled: bool,
}

/// List log forwarding configurations response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLogForwardingResponse {
    #[serde(default)]
    pub items: Vec<LogForwardingConfiguration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_status_roundtrip() {
        for variant in [
            ApplicationStatus::Unknown,
            ApplicationStatus::Active,
            ApplicationStatus::Progressing,
            ApplicationStatus::Inactive,
            ApplicationStatus::Failing,
            ApplicationStatus::Suspended,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let decoded: ApplicationStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, variant);
        }
    }

    #[test]
    fn runtime_type_roundtrip() {
        let json = serde_json::to_string(&RuntimeType::Shared).unwrap();
        assert_eq!(json, "\"Shared\"");
        let decoded: RuntimeType = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, RuntimeType::Shared);
    }

    #[test]
    fn problem_details_display() {
        let p = ProblemDetails {
            problem_type: None,
            title: Some("Not Found".to_string()),
            status: Some(404),
            detail: Some("Application not found".to_string()),
            instance: None,
        };
        let s = p.to_string();
        assert!(s.contains("404"));
        assert!(s.contains("Not Found"));
        assert!(s.contains("Application not found"));
    }

    #[test]
    fn error_details_display_with_validation() {
        let e = ErrorDetails {
            title: Some("Bad Request".to_string()),
            status: Some(400),
            detail: None,
            instance: None,
            errors: Some(vec![ValidationError {
                field: Some("name".to_string()),
                message: "Name is required".to_string(),
            }]),
        };
        let s = e.to_string();
        assert!(s.contains("400"));
        assert!(s.contains("name: Name is required"));
    }

    #[test]
    fn add_application_request_serializes() {
        let req = AddApplicationRequest {
            name: "my-app".to_string(),
            runtime_type: RuntimeType::Shared,
            auto_scaling: AutoscalingSettings { min: 1, max: 3 },
            region_settings: UpdateRegionSettingsRequest {
                allowed_region_ids: Some(vec!["DE".to_string()]),
                ..Default::default()
            },
            termination_grace_period_seconds: None,
            repository_settings: None,
            container_templates: None,
            volumes: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"name\":\"my-app\""));
        assert!(json.contains("\"runtimeType\":\"Shared\""));
        assert!(json.contains("\"min\":1"));
    }

    #[test]
    fn cursor_list_deserializes() {
        let json = r#"{"items":[{"id":"a","name":"test","status":"Active"}],"meta":{"totalItems":1},"cursor":null}"#;
        let list: CursorList<AppListItem> = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].name, "test");
        assert!(list.cursor.is_none());
    }

    #[test]
    fn volume_instance_status_roundtrip() {
        for variant in [
            VolumeInstanceStatus::Unknown,
            VolumeInstanceStatus::Attached,
            VolumeInstanceStatus::Detached,
            VolumeInstanceStatus::Failed,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let decoded: VolumeInstanceStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, variant);
        }
    }

    #[test]
    fn log_forwarding_request_serializes() {
        let req = LogForwardingRequest {
            app: "app-123".to_string(),
            forwarding_type: LogForwardingType::SyslogTcp,
            endpoint: "logs.example.com".to_string(),
            port: 514,
            token: None,
            format: SyslogFormat::SyslogRfc5424,
            enabled: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"type\":\"SyslogTcp\""));
        assert!(json.contains("\"format\":\"SyslogRfc5424\""));
        assert!(!json.contains("token"));
    }
}
