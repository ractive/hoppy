use crate::auth;
use crate::cli::{
    ContainerAction, ContainerAppAction, ContainerEndpointAction, ContainerLogForwardingAction,
    ContainerNodeAction, ContainerPodAction, ContainerRegionAction, ContainerRegistryAction,
    ContainerTemplateAction, ContainerVolumeAction, OutputFormat,
};
use crate::output;
use crate::redact::{self, RedactConfig};
use anyhow::{Context, Result, bail};
use bunny_net_api::containers::{
    AddApplicationRequest, AddContainerRequest, AnycastEndpointRequest, AnycastIpProtocolVersion,
    AppListItem, AutoscalingSettings, CdnEndpointRequest, ContainerConfigSuggestions,
    ContainerEntryPoint, ContainerImage, ContainerImageTag, ContainerPortMappingRequest,
    ContainerProbes, ContainerRegistryRequest, ContainersClient, CursorList, EndpointListItem,
    EndpointRequest, ErrorDetails, GetContainerConfigSuggestionsRequest,
    GetContainerImageDigestRequest, Granularity, ImagePullPolicy, ImageTagInfo,
    ListContainerImageTagsRequest, ListContainerImagesRequest, LogForwardingConfiguration,
    LogForwardingRequest, LogForwardingType, PatchApplicationRequest, PatchContainerRequest,
    PatchVolumeRequest, ProblemDetails, Protocol, Region, RegionSettings, RegistryCredentials,
    SearchPublicContainerImagesRequest, StickySessionSettings, SyslogFormat,
    UpdateRegionSettingsRequest, VolumeMountRequest, VolumeRequest,
};
use bunny_syslog_receiver::{
    BoreTunnel, LocalListener, LogEvent, NoopTunnel, Severity, StaticTunnel, Tunnel, spawn_receiver,
};
use console::style;
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

// ---------------------------------------------------------------------------
// Helper: build the client
// ---------------------------------------------------------------------------

fn client(debug: bool, record: Option<&str>, reveal: bool) -> Result<ContainersClient> {
    auth::containers_client_with_reveal(debug, record, reveal)
}

// ---------------------------------------------------------------------------
// Display rows — Applications
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct AppRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
}

impl From<&bunny_net_api::containers::AppListItem> for AppRow {
    fn from(a: &bunny_net_api::containers::AppListItem) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            status: format!("{:?}", a.status),
        }
    }
}

/// Wider table row used after `app create` so operators (and LLMs) can chain
/// the template id and display-endpoint id without a follow-up `app get`.
#[derive(serde::Serialize, tabled::Tabled)]
struct AppDetailWithIds {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Template IDs")]
    template_ids: String,
    #[tabled(rename = "Endpoint ID")]
    endpoint_id: String,
}

impl From<&bunny_net_api::containers::Application> for AppDetailWithIds {
    fn from(a: &bunny_net_api::containers::Application) -> Self {
        let template_ids = if a.container_templates.is_empty() {
            "-".to_owned()
        } else {
            a.container_templates
                .iter()
                .map(|t| t.id.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let endpoint_id = a
            .display_endpoint
            .as_ref()
            .map(|e| e.id.clone())
            .unwrap_or_else(|| "-".to_owned());
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            status: format!("{:?}", a.status),
            template_ids,
            endpoint_id,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Container templates
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ContainerTemplateRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Image")]
    image: String,
    #[tabled(rename = "Tag")]
    image_tag: String,
}

impl From<&bunny_net_api::containers::ContainerTemplate> for ContainerTemplateRow {
    fn from(t: &bunny_net_api::containers::ContainerTemplate) -> Self {
        Self {
            id: t.id.clone(),
            name: t.name.clone(),
            image: t.image_name.clone(),
            image_tag: t.image_tag.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Endpoints
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct EndpointRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    display_name: String,
    #[tabled(rename = "Type")]
    endpoint_type: String,
    #[tabled(rename = "Host")]
    public_host: String,
    #[tabled(rename = "SSL")]
    is_ssl_enabled: bool,
}

impl From<&bunny_net_api::containers::EndpointListItem> for EndpointRow {
    fn from(e: &bunny_net_api::containers::EndpointListItem) -> Self {
        Self {
            id: e.id.clone(),
            display_name: e.display_name.clone(),
            endpoint_type: format!("{:?}", e.endpoint_type),
            public_host: e.public_host.clone(),
            is_ssl_enabled: e.is_ssl_enabled,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Volumes
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct VolumeRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Size (GB)")]
    size: String,
    #[tabled(rename = "Instances")]
    total_instances_count: i32,
    #[tabled(rename = "Attached")]
    attached_instances_count: i32,
}

impl From<&bunny_net_api::containers::VolumeInList> for VolumeRow {
    fn from(v: &bunny_net_api::containers::VolumeInList) -> Self {
        Self {
            id: v.id.clone(),
            name: v.name.clone(),
            size: format!("{:.1}", v.size),
            total_instances_count: v.total_instances_count,
            attached_instances_count: v.attached_instances_count,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Registries
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct RegistryRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    display_name: String,
    #[tabled(rename = "Host")]
    host_name: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

impl From<&bunny_net_api::containers::ContainerRegistry> for RegistryRow {
    fn from(r: &bunny_net_api::containers::ContainerRegistry) -> Self {
        Self {
            id: r
                .id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            display_name: r.display_name.clone(),
            host_name: r.host_name.clone(),
            created_at: r.created_at.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Regions
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
#[serde(rename_all = "PascalCase")]
struct RegionRow {
    #[tabled(rename = "ID")]
    #[serde(rename = "Id")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Group")]
    group: String,
    #[tabled(rename = "Anycast")]
    has_anycast_support: bool,
    #[tabled(rename = "Capacity")]
    has_capacity: bool,
}

impl From<&bunny_net_api::containers::Region> for RegionRow {
    fn from(r: &bunny_net_api::containers::Region) -> Self {
        Self {
            id: r.id.clone(),
            name: r.name.clone(),
            group: r.group.clone(),
            has_anycast_support: r.has_anycast_support,
            has_capacity: r.has_capacity,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — User Limits
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct UserLimitsRow {
    #[tabled(rename = "Max Apps")]
    max_number_of_applications: i32,
    #[tabled(rename = "Existing Apps")]
    existing_number_of_applications: i32,
    #[tabled(rename = "Max Instances/Region")]
    max_number_of_instances_per_region: i32,
    #[tabled(rename = "Max Volumes/App")]
    max_number_of_volumes_per_application: i32,
}

impl From<&bunny_net_api::containers::UserLimits> for UserLimitsRow {
    fn from(l: &bunny_net_api::containers::UserLimits) -> Self {
        Self {
            max_number_of_applications: l.max_number_of_applications,
            existing_number_of_applications: l.existing_number_of_applications,
            max_number_of_instances_per_region: l.max_number_of_instances_per_region,
            max_number_of_volumes_per_application: l.max_number_of_volumes_per_application,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Log Forwarding
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct LogForwardingRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "App")]
    app: String,
    #[tabled(rename = "Type")]
    forwarding_type: String,
    #[tabled(rename = "Endpoint")]
    endpoint: String,
    #[tabled(rename = "Port")]
    port: i32,
    #[tabled(rename = "Enabled")]
    enabled: bool,
}

impl From<&bunny_net_api::containers::LogForwardingConfiguration> for LogForwardingRow {
    fn from(l: &bunny_net_api::containers::LogForwardingConfiguration) -> Self {
        Self {
            id: l.id.clone(),
            app: l.app.clone(),
            forwarding_type: format!("{:?}", l.forwarding_type),
            endpoint: l.endpoint.clone(),
            port: l.port,
            enabled: l.enabled,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Autoscaling
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct AutoscalingRow {
    #[tabled(rename = "Min Instances")]
    min: i32,
    #[tabled(rename = "Max Instances")]
    max: i32,
}

impl From<&AutoscalingSettings> for AutoscalingRow {
    fn from(s: &AutoscalingSettings) -> Self {
        Self {
            min: s.min,
            max: s.max,
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Region Settings
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct RegionSettingsRow {
    #[tabled(rename = "Allowed Regions")]
    allowed_region_ids: String,
    #[tabled(rename = "Required Regions")]
    required_region_ids: String,
    #[tabled(rename = "Max Allowed")]
    max_allowed_regions: String,
    #[tabled(rename = "Provisioning Type")]
    provisioning_type: String,
}

impl From<&RegionSettings> for RegionSettingsRow {
    fn from(s: &RegionSettings) -> Self {
        Self {
            allowed_region_ids: if s.allowed_region_ids.is_empty() {
                "-".to_owned()
            } else {
                s.allowed_region_ids.join(", ")
            },
            required_region_ids: if s.required_region_ids.is_empty() {
                "-".to_owned()
            } else {
                s.required_region_ids.join(", ")
            },
            max_allowed_regions: s
                .max_allowed_regions
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            provisioning_type: s
                .provisioning_type
                .map(|p| format!("{p:?}"))
                .unwrap_or_else(|| "-".to_owned()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Container image tags
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ImageTagRow {
    #[tabled(rename = "Tag")]
    name: String,
}

impl From<&ContainerImageTag> for ImageTagRow {
    fn from(t: &ContainerImageTag) -> Self {
        Self {
            name: t.name.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Image digest
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ImageDigestRow {
    #[tabled(rename = "Namespace")]
    image_namespace: String,
    #[tabled(rename = "Image")]
    image: String,
    #[tabled(rename = "Tag")]
    tag: String,
    #[tabled(rename = "Digest")]
    digest: String,
}

impl From<&ImageTagInfo> for ImageDigestRow {
    fn from(i: &ImageTagInfo) -> Self {
        Self {
            image_namespace: i.image_namespace.as_deref().unwrap_or("-").to_owned(),
            image: i.image.as_deref().unwrap_or("-").to_owned(),
            tag: i.tag.as_deref().unwrap_or("-").to_owned(),
            digest: i.digest.as_deref().unwrap_or("-").to_owned(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Config suggestions
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ConfigSuggestionsRow {
    #[tabled(rename = "App Name")]
    app_name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Registry URL")]
    registry_url: String,
    #[tabled(rename = "Env Suggestions")]
    env_suggestions_count: usize,
}

impl From<&ContainerConfigSuggestions> for ConfigSuggestionsRow {
    fn from(s: &ContainerConfigSuggestions) -> Self {
        Self {
            app_name: s.app_name.as_deref().unwrap_or("-").to_owned(),
            description: s.description.as_deref().unwrap_or("-").to_owned(),
            registry_url: s.registry_url.as_deref().unwrap_or("-").to_owned(),
            env_suggestions_count: s.environment_variables_suggestions.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// Display rows — Public container images
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct PublicImageRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Namespace")]
    namespace: String,
}

impl From<&ContainerImage> for PublicImageRow {
    fn from(i: &ContainerImage) -> Self {
        Self {
            id: i.id.clone(),
            namespace: i.namespace.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Enum parsing helpers
// ---------------------------------------------------------------------------

fn parse_env_pairs(pairs: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for pair in pairs {
        let (k, v) = pair
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("env '{pair}' is not in KEY=VALUE format"))?;
        map.insert(k.to_owned(), v.to_owned());
    }
    Ok(map)
}

/// Parse `NAME:SIZE_GB` volume specs into request bodies. Size must be a
/// positive integer number of gigabytes.
fn parse_volumes(specs: &[String]) -> Result<Vec<VolumeRequest>> {
    specs
        .iter()
        .map(|spec| {
            let (name, size_str) = spec.split_once(':').ok_or_else(|| {
                anyhow::anyhow!("volume '{spec}' is not in NAME:SIZE_GB format (e.g. data:10)")
            })?;
            if name.is_empty() {
                bail!("volume '{spec}' has an empty name");
            }
            let size: i32 = size_str
                .parse()
                .map_err(|_| anyhow::anyhow!("volume '{spec}': size must be an integer (GB)"))?;
            if size <= 0 {
                bail!("volume '{spec}': size must be a positive integer (GB)");
            }
            Ok(VolumeRequest {
                name: name.to_owned(),
                size,
            })
        })
        .collect()
}

/// Parse `NAME:MOUNT_PATH` volume-mount specs. Splits on the first colon so
/// absolute mount paths (which contain no colon) are preserved verbatim.
fn parse_volume_mounts(specs: &[String]) -> Result<Vec<VolumeMountRequest>> {
    specs
        .iter()
        .map(|spec| {
            let (name, path) = spec.split_once(':').ok_or_else(|| {
                anyhow::anyhow!(
                    "volume mount '{spec}' is not in NAME:MOUNT_PATH format (e.g. data:/var/lib/data)"
                )
            })?;
            if name.is_empty() || path.is_empty() {
                bail!("volume mount '{spec}' needs both a name and a mount path");
            }
            Ok(VolumeMountRequest {
                name: name.to_owned(),
                mount_path: path.to_owned(),
            })
        })
        .collect()
}

/// Parse an image pull policy string (`Always` / `IfNotPresent`,
/// case-insensitive).
fn parse_pull_policy(s: &str) -> Result<ImagePullPolicy> {
    match s.to_lowercase().as_str() {
        "always" => Ok(ImagePullPolicy::Always),
        "ifnotpresent" => Ok(ImagePullPolicy::IfNotPresent),
        other => bail!("invalid pull-policy '{other}': must be Always or IfNotPresent"),
    }
}

/// Parse a network protocol string (`Tcp` / `Udp` / `Sctp`, case-insensitive).
fn parse_protocol(s: &str) -> Result<Protocol> {
    match s.to_lowercase().as_str() {
        "tcp" => Ok(Protocol::Tcp),
        "udp" => Ok(Protocol::Udp),
        "sctp" => Ok(Protocol::Sctp),
        other => bail!("invalid protocol '{other}': must be Tcp, Udp, or Sctp"),
    }
}

/// Parse a `CONTAINER[:EXPOSED[:PROTO,...]]` port-mapping spec.
///
/// - `8080`            → container 8080, no exposed port, no protocols
/// - `8080:80`         → container 8080, exposed 80
/// - `8080:80:Tcp,Udp` → container 8080, exposed 80, protocols [Tcp, Udp]
/// - `53::Udp`         → container 53, no exposed port, protocol [Udp]
fn parse_port_mapping(spec: &str) -> Result<ContainerPortMappingRequest> {
    let mut parts = spec.splitn(3, ':');
    let container_str = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("port mapping '{spec}' is missing a container port"))?;
    let container_port: i32 = container_str
        .parse()
        .map_err(|_| anyhow::anyhow!("port mapping '{spec}': container port must be an integer"))?;
    let exposed_port = match parts.next() {
        Some(s) if !s.is_empty() => Some(s.parse::<i32>().map_err(|_| {
            anyhow::anyhow!("port mapping '{spec}': exposed port must be an integer")
        })?),
        _ => None,
    };
    let protocols = match parts.next() {
        Some(s) if !s.is_empty() => {
            let protos = s
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(parse_protocol)
                .collect::<Result<Vec<_>>>()?;
            if protos.is_empty() {
                None
            } else {
                Some(protos)
            }
        }
        _ => None,
    };
    Ok(ContainerPortMappingRequest {
        container_port,
        exposed_port,
        protocols,
    })
}

/// Build the list of port mappings for an endpoint from either the granular
/// `--port-mapping` specs or the legacy `--container-port`/`--exposed-port`
/// pair. Exactly one source must be present (clap enforces at least one via
/// `required_unless_present`).
fn resolve_port_mappings(
    container_port: Option<i32>,
    exposed_port: Option<i32>,
    port_mappings: &[String],
) -> Result<Vec<ContainerPortMappingRequest>> {
    if !port_mappings.is_empty() {
        if container_port.is_some() {
            bail!("use either --container-port or --port-mapping, not both");
        }
        port_mappings
            .iter()
            .map(|s| parse_port_mapping(s))
            .collect()
    } else {
        let cp = container_port
            .ok_or_else(|| anyhow::anyhow!("--container-port or --port-mapping is required"))?;
        Ok(vec![ContainerPortMappingRequest {
            container_port: cp,
            exposed_port,
            protocols: None,
        }])
    }
}

/// Build a `ContainerEntryPoint` from the CLI flags, returning `None` when no
/// entrypoint flag was supplied. Array forms take precedence over the string
/// forms when both are given for the same field.
fn build_entry_point(
    command: &Option<String>,
    command_array: &[String],
    arguments: &Option<String>,
    arguments_array: &[String],
    working_directory: &Option<String>,
) -> Option<ContainerEntryPoint> {
    if command.is_none()
        && command_array.is_empty()
        && arguments.is_none()
        && arguments_array.is_empty()
        && working_directory.is_none()
    {
        return None;
    }
    Some(ContainerEntryPoint {
        command: if command_array.is_empty() {
            command.clone()
        } else {
            None
        },
        command_array: if command_array.is_empty() {
            None
        } else {
            Some(command_array.to_vec())
        },
        arguments: if arguments_array.is_empty() {
            arguments.clone()
        } else {
            None
        },
        arguments_array: if arguments_array.is_empty() {
            None
        } else {
            Some(arguments_array.to_vec())
        },
        working_directory: working_directory.clone(),
    })
}

/// Load and parse a `--probes-json` file into a `ContainerProbes`.
fn load_probes(path: &str) -> Result<ContainerProbes> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read probes JSON file '{path}'"))?;
    serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse probes JSON file '{path}'"))
}

/// Build the optional sticky-session settings block for a CDN endpoint.
fn build_sticky_sessions(
    sticky: bool,
    sticky_headers: &[String],
    sticky_cookie: &Option<String>,
) -> Result<Option<StickySessionSettings>> {
    if !sticky && sticky_headers.is_empty() && sticky_cookie.is_none() {
        return Ok(None);
    }
    if sticky && sticky_headers.is_empty() {
        bail!("--sticky requires at least one --sticky-header (1-3 allowed)");
    }
    if sticky_headers.len() > 3 {
        bail!("--sticky-header accepts at most 3 headers");
    }
    Ok(Some(StickySessionSettings {
        session_headers: sticky_headers.to_vec(),
        enabled: Some(sticky),
        cookie_name: sticky_cookie.clone(),
    }))
}

/// Walk the error chain looking for a structured 404 from the containers client.
/// Falls back to checking the rendered message if no typed error matches.
fn is_not_found_error(e: &anyhow::Error) -> bool {
    for cause in e.chain() {
        if let Some(err) = cause.downcast_ref::<ErrorDetails>()
            && err.status == Some(404)
        {
            return true;
        }
        if let Some(p) = cause.downcast_ref::<ProblemDetails>()
            && p.status == Some(404)
        {
            return true;
        }
    }
    let msg = format!("{e:#}");
    msg.contains("HTTP 404") || msg.to_lowercase().contains("not found")
}

fn is_confirmed(input: &str) -> bool {
    let answer = input.trim().to_lowercase();
    answer == "y" || answer == "yes"
}

async fn confirm(prompt: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || {
        eprint!("{prompt} [y/N] ");
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        Ok(is_confirmed(&line))
    })
    .await
    .context("confirm task panicked")?
}

/// Require an exact phrase typed at the prompt. Used for destructive actions
/// where a `[y/N]` accept is too easy to fat-finger.
async fn confirm_phrase(prompt: String, phrase: &'static str) -> Result<bool> {
    tokio::task::spawn_blocking(move || {
        eprint!("{prompt} ");
        io::stderr().flush()?;
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line)?;
        Ok(line.trim() == phrase)
    })
    .await
    .context("confirm task panicked")?
}

/// Print a value as JSON, applying env-var redaction first.
fn print_json_with_redaction<T: serde::Serialize>(value: &T, redact: &RedactConfig) -> Result<()> {
    let mut json = serde_json::to_value(value).context("failed to serialize to JSON")?;
    redact::redact_env_in_json(&mut json, redact);
    println!(
        "{}",
        serde_json::to_string_pretty(&json).context("failed to serialize to JSON")?
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Top-level handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &ContainerAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    redact: &RedactConfig,
) -> Result<()> {
    match action {
        ContainerAction::App { action } => {
            handle_app(action, format, debug, yes, record, redact).await
        }
        ContainerAction::Template { action } => {
            handle_template(action, format, debug, yes, record, redact).await
        }
        ContainerAction::Endpoint { action } => {
            handle_endpoint(action, format, debug, yes, record, redact.reveal_all).await
        }
        ContainerAction::Volume { action } => {
            handle_volume(action, format, debug, yes, record, redact.reveal_all).await
        }
        ContainerAction::Registry { action } => {
            handle_registry(action, format, debug, yes, record, redact.reveal_all).await
        }
        ContainerAction::Region { action } => {
            handle_region(action, format, debug, record, redact.reveal_all).await
        }
        ContainerAction::Node { action } => {
            handle_node(action, format, debug, record, redact.reveal_all).await
        }
        ContainerAction::Pod { action } => {
            handle_pod(action, debug, record, redact.reveal_all).await
        }
        ContainerAction::Limits => handle_limits(format, debug, record, redact.reveal_all).await,
        ContainerAction::LogForwarding { action } => {
            handle_log_forwarding(action, format, debug, yes, record, redact.reveal_all).await
        }
        ContainerAction::List { cursor, limit, all } => {
            handle_app(
                &ContainerAppAction::List {
                    cursor: cursor.clone(),
                    limit: *limit,
                    all: *all,
                },
                format,
                debug,
                yes,
                record,
                redact,
            )
            .await
        }
        ContainerAction::Get { id } => {
            handle_app(
                &ContainerAppAction::Get { id: id.clone() },
                format,
                debug,
                yes,
                record,
                redact,
            )
            .await
        }
        ContainerAction::Delete {
            id,
            cascade,
            no_cascade,
        } => {
            handle_app(
                &ContainerAppAction::Delete {
                    id: id.clone(),
                    cascade: *cascade,
                    no_cascade: *no_cascade,
                },
                format,
                debug,
                yes,
                record,
                redact,
            )
            .await
        }
        ContainerAction::Logs {
            app_id,
            tunnel,
            tunnel_host,
            local_port,
            replace_existing,
            follow: _,
            bore_server,
        } => {
            handle_logs(
                app_id,
                tunnel,
                tunnel_host.as_deref(),
                *local_port,
                *replace_existing,
                format,
                bore_server.as_deref(),
                debug,
                record,
                redact.reveal_all,
            )
            .await
        }
    }
}

// ---------------------------------------------------------------------------
// Application sub-handlers
// ---------------------------------------------------------------------------

async fn handle_app(
    action: &ContainerAppAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    redact: &RedactConfig,
) -> Result<()> {
    let c = client(debug, record, redact.reveal_all)?;
    match action {
        ContainerAppAction::List { cursor, limit, all } => {
            if *all {
                let mut next_cursor: Option<String> = None;
                let mut accumulated: Vec<AppListItem> = Vec::new();
                loop {
                    let result = c.list_applications(next_cursor.as_deref(), None).await?;
                    let more_cursor = result.cursor.clone();
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        let rows: Vec<AppRow> = result.items.iter().map(AppRow::from).collect();
                        output::print_data(&rows, format);
                    }
                    match more_cursor {
                        Some(c) if !c.is_empty() => next_cursor = Some(c),
                        _ => break,
                    }
                }
                if let OutputFormat::Json = format {
                    let combined: CursorList<AppListItem> = CursorList {
                        items: accumulated,
                        meta: None,
                        cursor: None,
                    };
                    print_json_with_redaction(&combined, redact)?;
                }
            } else {
                let result = c
                    .list_applications(cursor.as_deref(), limit.as_ref().copied())
                    .await?;
                if let OutputFormat::Json = format {
                    print_json_with_redaction(&result, redact)?;
                } else {
                    let rows: Vec<AppRow> = result.items.iter().map(AppRow::from).collect();
                    output::print_data(&rows, format);
                }
            }
        }
        ContainerAppAction::Get { id } => {
            let app = c.get_application(id).await?;
            if let OutputFormat::Json = format {
                print_json_with_redaction(&app, redact)?;
            } else {
                let cmd = format!("container app get --id {id}");
                output::print_single_vertical_with_cmd(&app, format, redact, Some(&cmd));
            }
        }
        ContainerAppAction::Summary { id } => {
            // Schema-less upstream operation — emit the raw JSON verbatim.
            let summary = c.get_application_summary(id).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).context("failed to serialize to JSON")?
            );
        }
        ContainerAppAction::Create {
            name,
            runtime_type,
            min,
            max,
            regions,
            image_name,
            image_namespace,
            image_tag,
            registry_id,
            env,
            volumes,
            minimal,
        } => {
            let has_image = image_name.is_some()
                || image_namespace.is_some()
                || image_tag.is_some()
                || registry_id.is_some();
            if !env.is_empty() && !has_image {
                bail!(
                    "--env requires --image-name / --image-namespace / --image-tag / \
                     --registry-id (env vars belong to a container template)"
                );
            }
            let container_templates = match (image_name, image_namespace, image_tag, registry_id) {
                (Some(img), Some(ns), Some(tag), Some(reg)) => {
                    use bunny_net_api::containers::{ContainerRequest, ImagePullPolicy};
                    Some(vec![ContainerRequest {
                        id: None,
                        name: img.clone(),
                        image_name: img.clone(),
                        image_namespace: ns.clone(),
                        image_tag: tag.clone(),
                        image_registry_id: reg.clone(),
                        image: None,
                        image_digest: None,
                        image_pull_policy: Some(ImagePullPolicy::IfNotPresent),
                        entry_point: None,
                        probes: None,
                        environment_variables: None,
                        endpoints: None,
                        volume_mounts: None,
                    }])
                }
                (None, None, None, None) => None,
                _ => bail!(
                    "--image-name, --image-namespace, --image-tag, and --registry-id \
                         must all be provided together"
                ),
            };
            let volume_reqs = if volumes.is_empty() {
                None
            } else {
                Some(parse_volumes(volumes)?)
            };
            let body = AddApplicationRequest {
                name: name.clone(),
                runtime_type: runtime_type.parse().map_err(anyhow::Error::msg)?,
                auto_scaling: AutoscalingSettings {
                    min: *min,
                    max: *max,
                },
                region_settings: UpdateRegionSettingsRequest {
                    allowed_region_ids: Some(regions.clone()),
                    ..Default::default()
                },
                termination_grace_period_seconds: None,
                repository_settings: None,
                container_templates,
                volumes: volume_reqs,
            };
            let resp = c.add_application(&body).await?;

            // Best-effort: apply --env via a follow-up template env --replace-all
            // call so env vars survive the next pod start. Failures here are
            // surfaced — the app exists but env did not land.
            if !env.is_empty() {
                let env_map = parse_env_pairs(env)?;
                let app_full = c.get_application(&resp.id).await?;
                let template_id = app_full
                    .container_templates
                    .first()
                    .map(|t| t.id.clone())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "app created but has no container templates; cannot apply --env"
                        )
                    })?;
                c.set_container_env(&resp.id, &template_id, &env_map)
                    .await
                    .context("created app but failed to apply --env")?;
            }

            if *minimal {
                if let OutputFormat::Json = format {
                    print_json_with_redaction(&resp, redact)?;
                } else {
                    output::print_mutation_result(
                        format,
                        "create",
                        "container-app",
                        serde_json::json!({ "Id": resp.id }),
                        &format!("Created application: {}", resp.id),
                    );
                }
            } else {
                // Default: return the full document so downstream tooling
                // doesn't need a follow-up `app get` to chain template /
                // endpoint ids.
                let app = c.get_application(&resp.id).await?;
                if let OutputFormat::Json = format {
                    print_json_with_redaction(&app, redact)?;
                } else {
                    output::print_mutation_result(
                        format,
                        "create",
                        "container-app",
                        serde_json::json!({ "Id": resp.id }),
                        &format!("Created application: {}", resp.id),
                    );
                    let row = AppDetailWithIds::from(&app);
                    output::print_single(&row, format);
                }
            }
            output::hints::tips(&[
                &format!(
                    "hoppy container template add --app-id {} --image-name <img> \
                     --image-namespace <ns> --image-tag <tag> --registry-id <reg>",
                    resp.id
                ),
                &format!("hoppy container endpoint add --app-id {} ...", resp.id),
            ]);
        }
        ContainerAppAction::Update {
            id,
            name,
            runtime_type,
            min,
            max,
            volumes,
        } => {
            if name.is_none()
                && runtime_type.is_none()
                && min.is_none()
                && max.is_none()
                && volumes.is_empty()
            {
                bail!(
                    "at least one update flag is required \
                     (--name, --runtime-type, --min, --max, --volume)"
                );
            }
            let volume_reqs = if volumes.is_empty() {
                None
            } else {
                Some(parse_volumes(volumes)?)
            };
            let auto_scaling = if min.is_some() || max.is_some() {
                // Fetch current values for the fields not being changed
                let current = c.get_application(id).await?;
                let current_scaling = current.auto_scaling.as_ref().map(|s| (s.min, s.max));
                let (resolved_min, resolved_max) = match current_scaling {
                    Some((cur_min, cur_max)) => (min.unwrap_or(cur_min), max.unwrap_or(cur_max)),
                    None => match (min, max) {
                        (Some(lo), Some(hi)) => (*lo, *hi),
                        _ => bail!(
                            "application has no autoscaling configured; \
                             provide both --min and --max to set it"
                        ),
                    },
                };
                Some(AutoscalingSettings {
                    min: resolved_min,
                    max: resolved_max,
                })
            } else {
                None
            };
            let body = PatchApplicationRequest {
                name: name.clone(),
                runtime_type: runtime_type
                    .as_deref()
                    .map(|s| s.parse().map_err(anyhow::Error::msg))
                    .transpose()?,
                auto_scaling,
                volumes: volume_reqs,
                ..Default::default()
            };
            let resp = c.patch_application(id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                output::print_mutation_result(
                    format,
                    "update",
                    "container-app",
                    serde_json::json!({ "Id": resp.id }),
                    &format!("Updated application: {}", resp.id),
                );
            }
        }
        ContainerAppAction::Deploy { id } => {
            c.deploy_application(id).await?;
            output::print_mutation_result(
                format,
                "deploy",
                "container-app",
                serde_json::json!({}),
                &format!("Deployed application {id}"),
            );
        }
        ContainerAppAction::Undeploy { id } => {
            if !yes && !confirm(format!("Undeploy application {id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            c.undeploy_application(id).await?;
            output::print_mutation_result(
                format,
                "undeploy",
                "container-app",
                serde_json::json!({}),
                &format!("Undeployed application {id}"),
            );
        }
        ContainerAppAction::Restart { id } => {
            c.restart_application(id).await?;
            output::print_mutation_result(
                format,
                "restart",
                "container-app",
                serde_json::json!({}),
                &format!("Restarted application {id}"),
            );
        }
        ContainerAppAction::Delete {
            id,
            cascade,
            no_cascade,
        } => {
            handle_app_delete(&c, id, *cascade, *no_cascade, yes, format, debug, record).await?;
        }
        ContainerAppAction::Overview { id } => {
            let overview = c.get_application_overview(id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&overview)
                        .context("failed to serialize to JSON")?
                );
            } else {
                // Overview is complex — show as JSON-style key/value in text mode
                let status = overview
                    .status
                    .as_ref()
                    .map(|s| format!("{s:?}"))
                    .unwrap_or_else(|| "-".to_owned());
                let active_regions = overview
                    .active_regions
                    .as_ref()
                    .map(|i| i.indicator.to_string())
                    .unwrap_or_else(|| "-".to_owned());
                let active_instances = overview
                    .active_instances
                    .as_ref()
                    .map(|i| i.indicator.to_string())
                    .unwrap_or_else(|| "-".to_owned());
                let monthly_cost = overview
                    .monthly_cost
                    .map(|c| format!("{c:.4}"))
                    .unwrap_or_else(|| "-".to_owned());
                eprintln!("Status: {status}");
                eprintln!("Active regions: {active_regions}");
                eprintln!("Active instances: {active_instances}");
                eprintln!("Monthly cost: {monthly_cost}");
            }
        }
        ContainerAppAction::Statistics {
            id,
            from,
            to,
            granularity,
        } => {
            let gran = granularity
                .as_deref()
                .map(|s| s.parse().map_err(anyhow::Error::msg))
                .transpose()?
                .unwrap_or(Granularity::Daily);
            let stats = c
                .get_application_statistics(id, from, gran, to.as_deref())
                .await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?
                );
            } else {
                eprintln!(
                    "Statistics data contains time-series charts — use --format json for full output"
                );
            }
        }
        ContainerAppAction::AutoscalingGet { app_id } => {
            let settings = c.get_autoscaling(app_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&settings)
                        .context("failed to serialize to JSON")?
                );
            } else {
                let row = AutoscalingRow::from(&settings);
                output::print_single(&row, format);
            }
        }
        ContainerAppAction::AutoscalingUpdate { app_id, min, max } => {
            let body = AutoscalingSettings {
                min: *min,
                max: *max,
            };
            c.update_autoscaling(app_id, &body).await?;
            output::print_mutation_result(
                format,
                "set-autoscaling",
                "container-app",
                serde_json::json!({}),
                &format!("Updated autoscaling for application {app_id}"),
            );
        }
        ContainerAppAction::RegionSettingsGet { app_id } => {
            let settings = c.get_region_settings(app_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&settings)
                        .context("failed to serialize to JSON")?
                );
            } else {
                let row = RegionSettingsRow::from(&settings);
                output::print_single(&row, format);
            }
        }
        ContainerAppAction::RegionSettingsUpdate {
            app_id,
            allowed_region_ids,
            required_region_ids,
            max_allowed_regions,
        } => {
            if allowed_region_ids.is_none()
                && required_region_ids.is_none()
                && max_allowed_regions.is_none()
            {
                bail!(
                    "at least one update flag is required \
                     (--allowed-region, --required-region, --max-allowed-regions)"
                );
            }
            let body = UpdateRegionSettingsRequest {
                allowed_region_ids: allowed_region_ids.clone(),
                required_region_ids: required_region_ids.clone(),
                max_allowed_regions: *max_allowed_regions,
                node_selectors: None,
            };
            c.update_region_settings(app_id, &body).await?;
            output::print_mutation_result(
                format,
                "set-region-settings",
                "container-app",
                serde_json::json!({}),
                &format!("Updated region settings for application {app_id}"),
            );
        }
    }
    Ok(())
}

/// Discover auto-managed Pull Zone IDs owned by a Magic Containers app.
///
/// Bunny creates a Pull Zone for every CDN endpoint and tracks its id in
/// `endpoint.pull_zone_id`; deleting the app does NOT cascade to those zones,
/// leaving them live and billable. We collect them up front so the operator
/// can see what will be orphaned before confirming.
async fn discover_auto_pull_zones(
    c: &ContainersClient,
    app_id: &str,
) -> Result<Vec<(String, i64)>> {
    let endpoints = c.list_endpoints(app_id).await?;
    let mut out: Vec<(String, i64)> = Vec::new();
    for ep in &endpoints.items {
        if let Some(pz_id) = parse_pull_zone_id(ep) {
            out.push((ep.id.clone(), pz_id));
        }
    }
    Ok(out)
}

/// Bunny stores `pullZoneId` as a string on the endpoint. Treat "0" / empty as
/// "no auto-PZ" (Anycast / public-IP endpoints don't have one).
fn parse_pull_zone_id(ep: &EndpointListItem) -> Option<i64> {
    let raw = ep.pull_zone_id.trim();
    if raw.is_empty() {
        return None;
    }
    let parsed: i64 = raw.parse().ok()?;
    if parsed == 0 { None } else { Some(parsed) }
}

#[allow(clippy::too_many_arguments)]
async fn handle_app_delete(
    c: &ContainersClient,
    id: &str,
    cascade: bool,
    no_cascade: bool,
    yes: bool,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
) -> Result<()> {
    let auto_pzs = match discover_auto_pull_zones(c, id).await {
        Ok(v) => v,
        Err(e) => {
            // Endpoint discovery is best-effort — if it fails, fall back to
            // the legacy single-step delete and warn the operator.
            eprintln!(
                "warning: failed to enumerate endpoints for app {id} ({e:#}); \
                 cannot detect orphan pull zones"
            );
            Vec::new()
        }
    };

    let has_auto_pzs = !auto_pzs.is_empty();
    if has_auto_pzs && !cascade && !no_cascade {
        eprintln!(
            "App {id} owns {n} auto-managed Pull Zone(s) that won't be deleted automatically:",
            n = auto_pzs.len()
        );
        for (ep, pz) in &auto_pzs {
            eprintln!("  - pull-zone {pz} (endpoint {ep})");
        }
        bail!(
            "refusing to delete: pass --cascade to also delete the Pull Zones, \
             or --no-cascade to delete only the app and leave them as orphans"
        );
    }

    if !yes {
        let prompt = if cascade && has_auto_pzs {
            format!(
                "Delete application {id} AND {n} auto-managed Pull Zone(s)?",
                n = auto_pzs.len()
            )
        } else {
            format!("Delete application {id}?")
        };
        if !confirm(prompt).await? {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    c.delete_application(id).await?;
    output::print_mutation_result(
        format,
        "delete",
        "container-app",
        serde_json::json!({}),
        &format!("Deleted application {id}"),
    );

    if has_auto_pzs && cascade {
        let core = auth::core_client(debug, record)?;
        let mut failures: Vec<(i64, String)> = Vec::new();
        for (_ep, pz) in &auto_pzs {
            match core.delete_pull_zone(*pz).await {
                Ok(()) => eprintln!("Deleted auto-managed pull zone {pz}"),
                Err(e) => {
                    eprintln!("warning: failed to delete pull zone {pz}: {e:#}");
                    failures.push((*pz, format!("{e:#}")));
                }
            }
        }
        if !failures.is_empty() {
            bail!(
                "deleted app {id} but {n} pull-zone cleanup(s) failed; \
                 retry with: hoppy pull-zone delete --id <id> --yes",
                n = failures.len()
            );
        }
    } else if has_auto_pzs && no_cascade {
        eprintln!(
            "Note: {n} auto-managed Pull Zone(s) were NOT deleted. Remove with:",
            n = auto_pzs.len()
        );
        for (_ep, pz) in &auto_pzs {
            eprintln!("  hoppy pull-zone delete --id {pz} --yes");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Template sub-handlers
// ---------------------------------------------------------------------------

async fn handle_template(
    action: &ContainerTemplateAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    redact: &RedactConfig,
) -> Result<()> {
    let c = client(debug, record, redact.reveal_all)?;
    match action {
        ContainerTemplateAction::Get {
            app_id,
            container_id,
        } => {
            let tmpl = c.get_container(app_id, container_id).await?;
            if let OutputFormat::Json = format {
                print_json_with_redaction(&tmpl, redact)?;
            } else {
                let row = ContainerTemplateRow::from(&tmpl);
                output::print_single(&row, format);
            }
        }
        ContainerTemplateAction::Add {
            app_id,
            name,
            image_name,
            image_namespace,
            image_tag,
            registry_id,
            image_digest,
            pull_policy,
            command,
            command_array,
            arguments,
            arguments_array,
            working_directory,
            probes_json,
            volume_mounts,
        } => {
            let entry_point = build_entry_point(
                command,
                command_array,
                arguments,
                arguments_array,
                working_directory,
            );
            let probes = probes_json.as_deref().map(load_probes).transpose()?;
            let pull = pull_policy.as_deref().map(parse_pull_policy).transpose()?;
            let mounts = if volume_mounts.is_empty() {
                None
            } else {
                Some(parse_volume_mounts(volume_mounts)?)
            };
            let body = AddContainerRequest {
                name: name.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                image_tag: image_tag.clone(),
                image_registry_id: registry_id.clone(),
                image: None,
                image_digest: image_digest.clone(),
                image_pull_policy: pull,
                entry_point,
                probes,
                environment_variables: None,
                endpoints: None,
                volume_mounts: mounts,
            };
            let tmpl = c.add_container(app_id, &body).await?;
            if let OutputFormat::Json = format {
                print_json_with_redaction(&tmpl, redact)?;
            } else {
                let row = ContainerTemplateRow::from(&tmpl);
                output::print_single(&row, format);
            }
        }
        ContainerTemplateAction::Update {
            app_id,
            container_id,
            name,
            image_tag,
            image_name,
            image_namespace,
            registry_id,
            image_digest,
            pull_policy,
            command,
            command_array,
            arguments,
            arguments_array,
            working_directory,
            probes_json,
            volume_mounts,
        } => {
            let entry_point = build_entry_point(
                command,
                command_array,
                arguments,
                arguments_array,
                working_directory,
            );
            let probes = probes_json.as_deref().map(load_probes).transpose()?;
            let pull = pull_policy.as_deref().map(parse_pull_policy).transpose()?;
            let mounts = if volume_mounts.is_empty() {
                None
            } else {
                Some(parse_volume_mounts(volume_mounts)?)
            };
            if name.is_none()
                && image_tag.is_none()
                && image_name.is_none()
                && image_namespace.is_none()
                && registry_id.is_none()
                && image_digest.is_none()
                && pull.is_none()
                && entry_point.is_none()
                && probes.is_none()
                && mounts.is_none()
            {
                bail!("at least one update flag is required");
            }
            let body = PatchContainerRequest {
                name: name.clone(),
                image_tag: image_tag.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                image_registry_id: registry_id.clone(),
                image_digest: image_digest.clone(),
                image_pull_policy: pull,
                entry_point,
                probes,
                volume_mounts: mounts,
                ..Default::default()
            };
            let tmpl = c.patch_container(app_id, container_id, &body).await?;
            if let OutputFormat::Json = format {
                print_json_with_redaction(&tmpl, redact)?;
            } else {
                let row = ContainerTemplateRow::from(&tmpl);
                output::print_single(&row, format);
            }
        }
        ContainerTemplateAction::Delete {
            app_id,
            container_id,
        } => {
            if !yes
                && !confirm(format!(
                    "Delete container template {container_id} from app {app_id}?"
                ))
                .await?
            {
                eprintln!("Aborted.");
                return Ok(());
            }
            c.delete_container(app_id, container_id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "container-template",
                serde_json::json!({}),
                &format!("Deleted container template {container_id}"),
            );
        }
        ContainerTemplateAction::Env {
            app_id,
            container_id,
            add,
            update,
            remove,
            replace_all,
            clear,
            list,
            env,
        } => {
            handle_template_env(
                &c,
                app_id,
                container_id,
                add,
                update,
                remove,
                *replace_all,
                *clear,
                *list,
                env,
                format,
                redact,
            )
            .await?;
        }
    }
    Ok(())
}

/// Implements MC.1 + MC.5: granular env operations on a container template.
///
/// Behaviour matrix (mutually exclusive groups marked):
/// - `--list`                    → print current env (redacted unless --reveal)
/// - `--clear`                   → wipe to empty (require typed confirmation)
/// - `--replace-all` + `--env …` → replace whole set
/// - `--add` / `--update` / `--remove` → granular merge (default)
///
/// A bare invocation (no flags) is rejected — historically this was a silent
/// "wipe everything" footgun; now it errors out with a recipe.
#[allow(clippy::too_many_arguments)]
async fn handle_template_env(
    c: &ContainersClient,
    app_id: &str,
    container_id: &str,
    add: &[String],
    update: &[String],
    remove: &[String],
    replace_all: bool,
    clear: bool,
    list: bool,
    env: &[String],
    format: OutputFormat,
    redact: &RedactConfig,
) -> Result<()> {
    let granular = !add.is_empty() || !update.is_empty() || !remove.is_empty();

    // Mutual-exclusion checks. clap's `conflicts_with` would handle this but
    // we have several groups, so we enforce by hand for clearer errors.
    let mode_count = [list, clear, replace_all, granular]
        .iter()
        .filter(|x| **x)
        .count();
    if mode_count > 1 {
        bail!(
            "--list, --clear, --replace-all, and --add/--remove/--update are \
             mutually exclusive — pick one mode per invocation"
        );
    }
    if !replace_all && !env.is_empty() {
        bail!("--env is only valid with --replace-all (use --add KEY=VAL otherwise)");
    }
    if replace_all && env.is_empty() {
        bail!("--replace-all requires one or more --env KEY=VAL");
    }

    if list {
        let tmpl = c.get_container(app_id, container_id).await?;
        let mut json = serde_json::to_value(&tmpl).context("failed to serialize to JSON")?;
        redact::redact_env_in_json(&mut json, redact);
        match format {
            OutputFormat::Json => {
                let env_arr = json
                    .get("environmentVariables")
                    .cloned()
                    .unwrap_or_else(|| serde_json::Value::Array(Vec::new()));
                println!(
                    "{}",
                    serde_json::to_string_pretty(&env_arr)
                        .context("failed to serialize to JSON")?
                );
            }
            _ => {
                let arr = json
                    .get("environmentVariables")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if arr.is_empty() {
                    eprintln!("No environment variables set.");
                } else {
                    for item in arr {
                        let name = item
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_owned();
                        let value = item
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_owned();
                        println!("{name}\t{value}");
                    }
                }
            }
        }
        return Ok(());
    }

    if clear {
        let current = c.get_container(app_id, container_id).await?;
        let n = current.environment_variables.len();
        // Decision-log (iter-21): typed phrase is required for `--clear`
        // regardless of `--yes`. `--yes` alone is too easy to fat-finger
        // after a successful prior invocation, so it does not bypass this
        // gate. Automation that needs to wipe non-interactively can pipe
        // the phrase via stdin: `echo wipe | hoppy ... --clear`.
        let prompt = format!(
            "Clear ALL {n} environment variable(s) on container {container_id}? \
             Type \"wipe\" to confirm:"
        );
        if !confirm_phrase(prompt, "wipe").await? {
            eprintln!("Aborted.");
            return Ok(());
        }
        let empty: HashMap<String, String> = HashMap::new();
        let tmpl = c.set_container_env(app_id, container_id, &empty).await?;
        report_env_result(&tmpl, format, redact)?;
        return Ok(());
    }

    if replace_all {
        let map = parse_env_pairs(env)?;
        let current = c.get_container(app_id, container_id).await?;
        let cur_n = current.environment_variables.len();
        let new_n = map.len();
        // Decision-log (iter-21): a shrinking `--replace-all` (which drops
        // existing vars) requires the typed phrase regardless of `--yes`.
        // Pure additive replacements (cur_n <= new_n) skip the phrase since
        // nothing is lost; they still went through the explicit `--replace-all`
        // mode selection.
        if cur_n > 0 && cur_n > new_n {
            let prompt = format!(
                "Replace {cur_n} environment variable(s) with {new_n}? \
                 Type \"replace\" to confirm:"
            );
            if !confirm_phrase(prompt, "replace").await? {
                eprintln!("Aborted.");
                return Ok(());
            }
        }
        let tmpl = c.set_container_env(app_id, container_id, &map).await?;
        report_env_result(&tmpl, format, redact)?;
        return Ok(());
    }

    if granular {
        // Read current set, apply all `--add`/`--update` inserts, then all
        // `--remove` drops. Removal always wins over insertion of the same
        // key in a single invocation — clap groups flag values by name, so
        // we cannot recover the exact CLI argv order without a custom parser.
        let current = c.get_container(app_id, container_id).await?;
        let mut map: HashMap<String, String> = current
            .environment_variables
            .iter()
            .map(|e| (e.name.clone(), e.value.clone().unwrap_or_default()))
            .collect();
        for pair in add.iter().chain(update.iter()) {
            let (k, v) = pair
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("env '{pair}' is not in KEY=VALUE format"))?;
            map.insert(k.to_owned(), v.to_owned());
        }
        for key in remove {
            map.remove(key);
        }
        let tmpl = c.set_container_env(app_id, container_id, &map).await?;
        report_env_result(&tmpl, format, redact)?;
        return Ok(());
    }

    // No flags at all — the historical zero-arg wipe. Refuse loudly with a
    // recipe so operators know how to do what they actually meant.
    bail!(
        "no operation specified — at least one of --add / --remove / --update / \
         --replace-all / --clear / --list is required.\n\n\
         Examples:\n  \
         hoppy container template env --app-id {app_id} --container-id {container_id} \
         --add KEY=VAL\n  \
         hoppy container template env --app-id {app_id} --container-id {container_id} \
         --remove KEY\n  \
         hoppy container template env --app-id {app_id} --container-id {container_id} \
         --list\n  \
         hoppy container template env --app-id {app_id} --container-id {container_id} \
         --clear   # wipe all (must type \"wipe\" to confirm)"
    );
}

fn report_env_result(
    tmpl: &bunny_net_api::containers::ContainerTemplate,
    format: OutputFormat,
    redact: &RedactConfig,
) -> Result<()> {
    if let OutputFormat::Json = format {
        print_json_with_redaction(tmpl, redact)?;
    } else {
        let row = ContainerTemplateRow::from(tmpl);
        output::print_single(&row, format);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Endpoint sub-handlers
// ---------------------------------------------------------------------------

/// Options captured from the endpoint add/update flags. Grouped into a struct
/// to keep `build_endpoint_request` under the clippy argument-count limit.
struct EndpointOpts<'a> {
    name: &'a str,
    container_port: Option<i32>,
    exposed_port: Option<i32>,
    port_mappings: &'a [String],
    /// Explicit `--cdn` selector. CDN is also the default when neither `--cdn`
    /// nor `--anycast` is passed, so this only matters for readability / future
    /// divergence; the request builder treats "not anycast" as CDN.
    #[allow(dead_code)]
    cdn: bool,
    anycast: bool,
    ssl: bool,
    pull_zone_id: Option<i32>,
    sticky: bool,
    sticky_headers: &'a [String],
    sticky_cookie: &'a Option<String>,
}

fn build_endpoint_request(opts: &EndpointOpts<'_>) -> Result<EndpointRequest> {
    let mappings =
        resolve_port_mappings(opts.container_port, opts.exposed_port, opts.port_mappings)?;

    if opts.anycast {
        // Anycast endpoints ignore CDN-only options; reject them loudly so an
        // operator doesn't think SSL / sticky sessions took effect.
        if opts.ssl || opts.pull_zone_id.is_some() || opts.sticky || !opts.sticky_headers.is_empty()
        {
            bail!(
                "--ssl / --pull-zone-id / --sticky* are CDN-only options and \
                 cannot be combined with --anycast"
            );
        }
        return Ok(EndpointRequest {
            display_name: opts.name.to_owned(),
            cdn: None,
            anycast: Some(AnycastEndpointRequest {
                protocol_version: AnycastIpProtocolVersion::IPv4,
                port_mappings: mappings,
            }),
        });
    }

    // Default (and explicit --cdn): CDN endpoint. CDN accepts exactly one
    // port mapping.
    if mappings.len() > 1 {
        bail!(
            "CDN endpoints accept exactly one port mapping; pass a single \
             --port-mapping (or --container-port), or use --anycast for several"
        );
    }
    let sticky_sessions =
        build_sticky_sessions(opts.sticky, opts.sticky_headers, opts.sticky_cookie)?;
    Ok(EndpointRequest {
        display_name: opts.name.to_owned(),
        cdn: Some(CdnEndpointRequest {
            is_ssl_enabled: if opts.ssl { Some(true) } else { None },
            sticky_sessions,
            pull_zone_id: opts.pull_zone_id,
            port_mappings: Some(mappings),
        }),
        anycast: None,
    })
}

async fn handle_endpoint(
    action: &ContainerEndpointAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerEndpointAction::List { app_id } => {
            let result = c.list_endpoints(app_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<EndpointRow> = result.items.iter().map(EndpointRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerEndpointAction::Add {
            app_id,
            container_id,
            name,
            container_port,
            exposed_port,
            port_mappings,
            cdn,
            anycast,
            ssl,
            pull_zone_id,
            sticky,
            sticky_headers,
            sticky_cookie,
        } => {
            let body = build_endpoint_request(&EndpointOpts {
                name,
                container_port: *container_port,
                exposed_port: *exposed_port,
                port_mappings,
                cdn: *cdn,
                anycast: *anycast,
                ssl: *ssl,
                pull_zone_id: *pull_zone_id,
                sticky: *sticky,
                sticky_headers,
                sticky_cookie,
            })?;
            let resp = c.add_endpoint(app_id, container_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                output::print_mutation_result(
                    format,
                    "create",
                    "container-endpoint",
                    serde_json::json!({ "Id": resp.id }),
                    &format!("Created endpoint: {}", resp.id),
                );
            }
        }
        ContainerEndpointAction::Update {
            app_id,
            endpoint_id,
            name,
            container_port,
            exposed_port,
            port_mappings,
            cdn,
            anycast,
            ssl,
            pull_zone_id,
            sticky,
            sticky_headers,
            sticky_cookie,
        } => {
            let body = build_endpoint_request(&EndpointOpts {
                name,
                container_port: *container_port,
                exposed_port: *exposed_port,
                port_mappings,
                cdn: *cdn,
                anycast: *anycast,
                ssl: *ssl,
                pull_zone_id: *pull_zone_id,
                sticky: *sticky,
                sticky_headers,
                sticky_cookie,
            })?;
            c.update_endpoint(app_id, endpoint_id, &body).await?;
            output::print_mutation_result(
                format,
                "update",
                "container-endpoint",
                serde_json::json!({}),
                &format!("Updated endpoint {endpoint_id}"),
            );
        }
        ContainerEndpointAction::Delete {
            app_id,
            endpoint_id,
        } => {
            if !yes && !confirm(format!("Delete endpoint {endpoint_id} from app {app_id}?")).await?
            {
                eprintln!("Aborted.");
                return Ok(());
            }
            c.delete_endpoint(app_id, endpoint_id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "container-endpoint",
                serde_json::json!({}),
                &format!("Deleted endpoint {endpoint_id}"),
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Volume sub-handlers
// ---------------------------------------------------------------------------

async fn handle_volume(
    action: &ContainerVolumeAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerVolumeAction::List { app_id } => {
            let result = c.list_volumes(app_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<VolumeRow> = result.items.iter().map(VolumeRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerVolumeAction::Update {
            app_id,
            volume_id,
            name,
            size,
        } => {
            if name.is_none() && size.is_none() {
                bail!("at least one update flag is required (--name, --size)");
            }
            let body = PatchVolumeRequest {
                name: name.clone(),
                size: *size,
            };
            let resp = c.update_volume(app_id, volume_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                output::print_mutation_result(
                    format,
                    "update",
                    "container-volume",
                    serde_json::json!({}),
                    &format!("Updated volume: {} ({:.1} GB)", resp.name, resp.size),
                );
            }
        }
        ContainerVolumeAction::Detach { app_id, volume_id } => {
            if !yes && !confirm(format!("Detach volume {volume_id} from app {app_id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            let resp = c.detach_volume(app_id, volume_id).await?;
            eprintln!("Detached volume: {}", resp.name);
        }
        ContainerVolumeAction::Delete { app_id, volume_id } => {
            if !yes
                && !confirm(format!(
                    "Delete all instances of volume {volume_id} from app {app_id}?"
                ))
                .await?
            {
                eprintln!("Aborted.");
                return Ok(());
            }
            let resp = c.delete_all_volume_instances(app_id, volume_id).await?;
            output::print_mutation_result(
                format,
                "delete-instances",
                "container-volume",
                serde_json::json!({}),
                &format!("Deleted {} volume instance(s)", resp.ids.len()),
            );
        }
        ContainerVolumeAction::DeleteInstance {
            app_id,
            volume_id,
            instance_id,
        } => {
            if !yes
                && !confirm(format!(
                    "Delete volume instance {instance_id} from volume {volume_id}?"
                ))
                .await?
            {
                eprintln!("Aborted.");
                return Ok(());
            }
            let resp = c
                .delete_volume_instance(app_id, volume_id, instance_id)
                .await?;
            output::print_mutation_result(
                format,
                "delete-instance",
                "container-volume",
                serde_json::json!({}),
                &format!("Deleted volume instance: {}", resp.id),
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Registry sub-handlers
// ---------------------------------------------------------------------------

async fn handle_registry(
    action: &ContainerRegistryAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerRegistryAction::List => {
            let result = c.list_registries().await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<RegistryRow> = result.items.iter().map(RegistryRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerRegistryAction::Get { id } => {
            let registry = c.get_registry(*id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&registry)
                        .context("failed to serialize to JSON")?
                );
            } else {
                let row = RegistryRow::from(&registry);
                output::print_single(&row, format);
            }
        }
        ContainerRegistryAction::Create {
            name,
            registry_type,
            username,
            password,
        } => {
            let credentials = match (username.as_deref(), password.as_deref()) {
                (Some(u), Some(p)) => Some(RegistryCredentials {
                    user_name: u.to_owned(),
                    password: p.to_owned(),
                }),
                (None, None) => None,
                _ => bail!("--username and --password must both be provided together"),
            };
            let body = ContainerRegistryRequest {
                display_name: name.clone(),
                registry_type: registry_type
                    .as_deref()
                    .map(|s| s.parse().map_err(anyhow::Error::msg))
                    .transpose()?,
                password_credentials: credentials,
            };
            let resp = c.add_registry(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                let id_str = resp
                    .id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "-".to_owned());
                output::print_mutation_result(
                    format,
                    "create",
                    "container-registry",
                    serde_json::json!({ "Id": id_str }),
                    &format!("Created registry: {} (status: {:?})", id_str, resp.status),
                );
            }
        }
        ContainerRegistryAction::Update {
            id,
            name,
            username,
            password,
        } => {
            let credentials = match (username.as_deref(), password.as_deref()) {
                (Some(u), Some(p)) => Some(RegistryCredentials {
                    user_name: u.to_owned(),
                    password: p.to_owned(),
                }),
                (None, None) => None,
                _ => bail!("--username and --password must both be provided together"),
            };
            let body = ContainerRegistryRequest {
                display_name: name.clone(),
                registry_type: None,
                password_credentials: credentials,
            };
            let resp = c.update_registry(*id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                output::print_mutation_result(
                    format,
                    "update",
                    "container-registry",
                    serde_json::json!({}),
                    &format!("Updated registry {} (status: {:?})", id, resp.status),
                );
            }
        }
        ContainerRegistryAction::Delete { id } => {
            if !yes && !confirm(format!("Delete registry {id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            let resp = c.delete_registry(*id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "container-registry",
                serde_json::json!({}),
                &format!("Deleted registry (status: {:?})", resp.status),
            );
        }
        ContainerRegistryAction::Images { registry_id } => {
            let body = ListContainerImagesRequest {
                registry_id: registry_id.clone(),
            };
            let images = c.list_container_images(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&images).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<PublicImageRow> = images.iter().map(PublicImageRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerRegistryAction::ImageConfig {
            registry_id,
            image_name,
            image_namespace,
            tag,
        } => {
            let body = GetContainerImageDigestRequest {
                registry_id: registry_id.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                tag: tag.clone(),
            };
            // Schema-less upstream operation — emit the raw JSON verbatim.
            let config = c.get_image_config(&body).await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&config).context("failed to serialize to JSON")?
            );
        }
        ContainerRegistryAction::ImageTags {
            registry_id,
            image_name,
            image_namespace,
        } => {
            let body = ListContainerImageTagsRequest {
                registry_id: registry_id.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
            };
            let tags = c.list_container_image_tags(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tags).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<ImageTagRow> = tags.iter().map(ImageTagRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerRegistryAction::ImageDigest {
            registry_id,
            image_name,
            image_namespace,
            tag,
        } => {
            let body = GetContainerImageDigestRequest {
                registry_id: registry_id.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                tag: tag.clone(),
            };
            let info = c.get_container_image_digest(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&info).context("failed to serialize to JSON")?
                );
            } else {
                let row = ImageDigestRow::from(&info);
                output::print_single(&row, format);
            }
        }
        ContainerRegistryAction::ConfigSuggestions {
            registry_id,
            image_name,
            image_namespace,
            tag,
        } => {
            let body = GetContainerConfigSuggestionsRequest {
                registry_id: registry_id.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                tag: tag.clone(),
            };
            let suggestions = c.get_config_suggestions(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&suggestions)
                        .context("failed to serialize to JSON")?
                );
            } else {
                let row = ConfigSuggestionsRow::from(&suggestions);
                output::print_single(&row, format);
            }
        }
        ContainerRegistryAction::SearchPublic {
            registry_id,
            query,
            size,
            page,
        } => {
            let body = SearchPublicContainerImagesRequest {
                registry_id: registry_id.clone(),
                prefix: query.clone(),
                size: *size,
                page: *page,
            };
            let images = c.search_public_images(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&images).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<PublicImageRow> = images.iter().map(PublicImageRow::from).collect();
                output::print_data(&rows, format);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Region sub-handlers
// ---------------------------------------------------------------------------

async fn handle_region(
    action: &ContainerRegionAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerRegionAction::List { cursor, limit, all } => {
            if *all {
                let mut next_cursor: Option<String> = None;
                let mut accumulated: Vec<Region> = Vec::new();
                loop {
                    let result = c.list_regions(next_cursor.as_deref(), None).await?;
                    let more_cursor = result.cursor.clone();
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        let rows: Vec<RegionRow> =
                            result.items.iter().map(RegionRow::from).collect();
                        output::print_data(&rows, format);
                    }
                    match more_cursor {
                        Some(c) if !c.is_empty() => next_cursor = Some(c),
                        _ => break,
                    }
                }
                if let OutputFormat::Json = format {
                    let combined: CursorList<Region> = CursorList {
                        items: accumulated,
                        meta: None,
                        cursor: None,
                    };
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&combined)
                            .context("failed to serialize to JSON")?
                    );
                }
            } else {
                let result = c
                    .list_regions(cursor.as_deref(), limit.as_ref().copied())
                    .await?;
                if let OutputFormat::Json = format {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&result)
                            .context("failed to serialize to JSON")?
                    );
                } else {
                    let rows: Vec<RegionRow> = result.items.iter().map(RegionRow::from).collect();
                    output::print_data(&rows, format);
                }
            }
        }
        ContainerRegionAction::Optimal => {
            let resp = c.get_optimal_region(None).await?;
            let row = RegionRow::from(&resp.region);
            output::print_single(&row, format);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Node sub-handlers
// ---------------------------------------------------------------------------

async fn handle_node(
    action: &ContainerNodeAction,
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerNodeAction::List { cursor, limit, all } => {
            if *all {
                let mut next_cursor: Option<String> = None;
                let mut accumulated: Vec<String> = Vec::new();
                loop {
                    let result = c.list_nodes(next_cursor.as_deref(), None).await?;
                    let more_cursor = result.cursor.clone();
                    if let OutputFormat::Json = format {
                        accumulated.extend(result.items);
                    } else {
                        for node in &result.items {
                            println!("{node}");
                        }
                    }
                    match more_cursor {
                        Some(c) if !c.is_empty() => next_cursor = Some(c),
                        _ => break,
                    }
                }
                if let OutputFormat::Json = format {
                    let combined: CursorList<String> = CursorList {
                        items: accumulated,
                        meta: None,
                        cursor: None,
                    };
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&combined)
                            .context("failed to serialize to JSON")?
                    );
                }
            } else {
                let result = c
                    .list_nodes(cursor.as_deref(), limit.as_ref().copied())
                    .await?;
                if let OutputFormat::Json = format {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&result)
                            .context("failed to serialize to JSON")?
                    );
                } else {
                    for node in &result.items {
                        println!("{node}");
                    }
                }
            }
        }
        ContainerNodeAction::Ips => {
            // Schema-less upstream operation — emit the raw JSON verbatim.
            let ips = c.list_nodes_plain().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&ips).context("failed to serialize to JSON")?
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pod sub-handlers
// ---------------------------------------------------------------------------

async fn handle_pod(
    action: &ContainerPodAction,
    debug: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerPodAction::Recreate { app_id, pod_id } => {
            c.recreate_pod(app_id, pod_id).await?;
            eprintln!("Recreated pod {pod_id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Limits handler
// ---------------------------------------------------------------------------

async fn handle_limits(
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    let limits = c.get_user_limits().await?;
    if let OutputFormat::Json = format {
        println!(
            "{}",
            serde_json::to_string_pretty(&limits).context("failed to serialize to JSON")?
        );
    } else {
        let row = UserLimitsRow::from(&limits);
        output::print_single(&row, format);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Log forwarding sub-handlers
// ---------------------------------------------------------------------------

async fn handle_log_forwarding(
    action: &ContainerLogForwardingAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    let c = client(debug, record, reveal)?;
    match action {
        ContainerLogForwardingAction::List => {
            let result = c.list_log_forwarding().await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result.items)
                        .context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<LogForwardingRow> =
                    result.items.iter().map(LogForwardingRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerLogForwardingAction::Get { app_id } => match c.get_log_forwarding(app_id).await {
            Ok(config) => {
                if let OutputFormat::Json = format {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&config)
                            .context("failed to serialize to JSON")?
                    );
                } else {
                    let row = LogForwardingRow::from(&config);
                    output::print_single(&row, format);
                }
            }
            Err(e) => {
                if is_not_found_error(&e) {
                    if let OutputFormat::Json = format {
                        println!("null");
                    } else {
                        eprintln!("No log forwarding configuration for app {app_id}");
                    }
                } else {
                    return Err(e);
                }
            }
        },
        ContainerLogForwardingAction::Create {
            app_id,
            forwarding_type,
            endpoint,
            port,
            syslog_format,
            token,
            enabled,
        } => {
            let body = LogForwardingRequest {
                app: app_id.clone(),
                forwarding_type: forwarding_type.parse().map_err(anyhow::Error::msg)?,
                endpoint: endpoint.clone(),
                port: *port,
                token: Some(token.clone()),
                format: syslog_format.parse().map_err(anyhow::Error::msg)?,
                enabled: *enabled,
            };
            let config = c.create_log_forwarding(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&config).context("failed to serialize to JSON")?
                );
            } else {
                let row = LogForwardingRow::from(&config);
                output::print_single(&row, format);
            }
        }
        ContainerLogForwardingAction::Update {
            app_id,
            forwarding_type,
            endpoint,
            port,
            syslog_format,
            token,
            enabled,
        } => {
            let body = LogForwardingRequest {
                app: app_id.clone(),
                forwarding_type: forwarding_type.parse().map_err(anyhow::Error::msg)?,
                endpoint: endpoint.clone(),
                port: *port,
                token: Some(token.clone()),
                format: syslog_format.parse().map_err(anyhow::Error::msg)?,
                enabled: *enabled,
            };
            let config = c.update_log_forwarding(app_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&config).context("failed to serialize to JSON")?
                );
            } else {
                let row = LogForwardingRow::from(&config);
                output::print_single(&row, format);
            }
        }
        ContainerLogForwardingAction::Delete { app_id } => {
            if !yes && !confirm(format!("Delete log forwarding config for app {app_id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            c.delete_log_forwarding(app_id).await?;
            output::print_mutation_result(
                format,
                "delete",
                "container-app-log-forwarding",
                serde_json::json!({}),
                &format!("Deleted log forwarding configuration for app {app_id}"),
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Logs — live syslog streaming
// ---------------------------------------------------------------------------

/// Local format selector for `handle_logs`. The global `OutputFormat` is not
/// used directly because "table" is invalid for streaming tail output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogsFormat {
    Text,
    Json,
}

/// Pretty-print a single [`LogEvent`] in human-readable form to stdout.
fn print_log_text(ev: &LogEvent) {
    // Extract HH:MM:SS from the RFC 3339 timestamp when available.
    let time_str = ev
        .timestamp
        .as_deref()
        .and_then(|ts| ts.get(11..19))
        .unwrap_or("??:??:??");

    let sev_label = ev.severity.map(Severity::label).unwrap_or("     ");

    let app_name = ev.app_name.as_deref().unwrap_or("-");

    let coloured_sev = match ev.severity {
        Some(Severity::Emergency | Severity::Alert | Severity::Critical | Severity::Error) => {
            style(sev_label).red().to_string()
        }
        Some(Severity::Warning) => style(sev_label).yellow().to_string(),
        Some(Severity::Notice) => style(sev_label).cyan().to_string(),
        Some(Severity::Informational) => style(sev_label).green().to_string(),
        Some(Severity::Debug) => style(sev_label).dim().to_string(),
        None => sev_label.to_owned(),
    };

    println!("{time_str} {coloured_sev} {app_name} | {}", ev.message);
}

/// Generate a per-session log-forwarding token.
///
/// The API rejects tokenless configurations with an empty 400 even though the
/// spec marks `token` optional (verified live 2026-08-09, see
/// `backlog/log-forwarding-create-empty-400.md`). hoppy's syslog listener
/// doesn't authenticate senders, so uniqueness is all that matters here.
fn generate_forwarding_token() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("hoppy-{:x}-{nanos:x}", std::process::id())
}

#[allow(clippy::too_many_arguments)]
async fn handle_logs(
    app_id: &str,
    tunnel: &str,
    tunnel_host: Option<&str>,
    local_port: u16,
    replace_existing: bool,
    format: OutputFormat,
    bore_server: Option<&str>,
    debug: bool,
    record: Option<&str>,
    reveal: bool,
) -> Result<()> {
    // --- 1. Validate format ---------------------------------------------------
    // Tail output is a live stream, not tabular data. The global `--format`
    // flag defaults to `table`, so silently fall back to `text` for that case
    // rather than failing every default invocation.
    let logs_format = match format {
        OutputFormat::Text | OutputFormat::Table => LogsFormat::Text,
        OutputFormat::Json => LogsFormat::Json,
    };

    // --- 2. Build client and verify app exists --------------------------------
    let c = client(debug, record, reveal)?;
    c.get_application(app_id)
        .await
        .with_context(|| format!("application '{app_id}' not found or inaccessible"))?;

    // --- 3. Idempotency check -------------------------------------------------
    // Store a prior config if --replace-existing is requested so we can
    // restore it on clean exit.
    let prior_config: Option<LogForwardingConfiguration> = match c.get_log_forwarding(app_id).await
    {
        Ok(existing) => {
            if !replace_existing {
                bail!(
                    "app {app_id} already has a log-forwarding config \
                         (endpoint={}). Run `hoppy container log-forwarding \
                         delete --app-id {app_id}` first, or use \
                         --replace-existing to take it over.",
                    existing.endpoint,
                );
            }
            // Delete the existing config; we'll restore it on exit.
            if debug {
                eprintln!(
                    "[debug] replacing existing log-forwarding config for {app_id} \
                         (endpoint={})",
                    existing.endpoint
                );
            }
            c.delete_log_forwarding(app_id)
                .await
                .context("failed to delete existing log-forwarding config")?;
            Some(existing)
        }
        Err(e) => {
            // A 404 / "not found" is the expected case — proceed normally.
            // Any other error is noted at debug verbosity but we carry on
            // (v1 best-effort; a typed error would be cleaner).
            if debug {
                eprintln!(
                    "[debug] get_log_forwarding returned an error (assuming no config exists): {e:#}"
                );
            }
            None
        }
    };

    // --- 4. Bind local listener -----------------------------------------------
    // For `--tunnel none` the user is providing their own ingress (VPN,
    // public IP, ssh -R from a different host, …) and that ingress will
    // not be able to reach a listener bound to 127.0.0.1. Bind all
    // interfaces in that case so the configured ingress can reach us.
    // For bore / a static tunnel-host the tunnel terminates on the same
    // machine so 127.0.0.1 is correct (and safer).
    let bind_all = tunnel == "none" && tunnel_host.is_none();
    let listener = if bind_all {
        LocalListener::bind_all_interfaces(local_port).await
    } else {
        LocalListener::bind(local_port).await
    }
    .context("failed to bind local syslog listener")?;
    let bound_port = listener.local_addr().port();

    // --- 5. Construct tunnel --------------------------------------------------
    let tunnel_impl: Box<dyn Tunnel> = if let Some(host_port) = tunnel_host {
        let (host, port_str) = host_port
            .rsplit_once(':')
            .with_context(|| format!("--tunnel-host '{host_port}' must be host:port"))?;
        let port: u16 = port_str
            .parse()
            .with_context(|| format!("invalid port in --tunnel-host '{host_port}'"))?;
        Box::new(StaticTunnel {
            host: host.to_owned(),
            port,
        })
    } else if tunnel == "none" {
        Box::new(NoopTunnel::default())
    } else {
        // Default: bore
        Box::new(BoreTunnel {
            server: bore_server.unwrap_or("bore.pub").to_owned(),
            ..Default::default()
        })
    };

    // --- 6. Start tunnel ------------------------------------------------------
    let mut tunnel_handle = tunnel_impl
        .start(bound_port)
        .await
        .context("failed to start tunnel")?;

    // --- 7. Register log forwarding with Bunny (skip for --tunnel none) -------
    let skip_forwarding = tunnel == "none" && tunnel_host.is_none();

    let created_config: Option<LogForwardingConfiguration> = if skip_forwarding {
        eprintln!(
            "Local syslog: 0.0.0.0:{bound_port} (all interfaces). \
             Configure forwarding manually."
        );
        None
    } else {
        let req = LogForwardingRequest {
            app: app_id.to_owned(),
            forwarding_type: LogForwardingType::SyslogTcp,
            endpoint: tunnel_handle.public_host.clone(),
            port: i32::from(tunnel_handle.public_port),
            format: SyslogFormat::SyslogRfc5424,
            enabled: true,
            token: Some(generate_forwarding_token()),
        };
        let cfg = c
            .create_log_forwarding(&req)
            .await
            .context("failed to create log-forwarding configuration")?;
        Some(cfg)
    };

    // --- 9. Print banner to stderr (stdout stays clean for --format json) -----
    if !skip_forwarding {
        eprintln!(
            "Listening on {}:{} → app {app_id}. \
             Logs may take 10–30s to start arriving (Bunny delivery delay). \
             Press Ctrl-C to stop.",
            tunnel_handle.public_host, tunnel_handle.public_port,
        );
    }

    // --- 10. Spawn receiver ---------------------------------------------------
    let (tx, mut rx) = mpsc::channel::<LogEvent>(1024);
    let cancel = CancellationToken::new();
    let (_addr, _server_handle) = spawn_receiver(listener, tx, cancel.clone());

    // --- 11. Stream loop ------------------------------------------------------
    // We await cleanup directly in all exit paths (normal Ctrl-C, bore child
    // exit, or channel close).  A panic in this async task will leak the
    // forwarding config — documented as a known limitation.
    let stream_result: Result<()> = async {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
                maybe_ev = rx.recv() => {
                    match maybe_ev {
                        Some(ev) => {
                            match logs_format {
                                LogsFormat::Text => print_log_text(&ev),
                                LogsFormat::Json => {
                                    let line = serde_json::to_string(&ev)
                                        .context("failed to serialise LogEvent to JSON")?;
                                    println!("{line}");
                                }
                            }
                        }
                        // Channel closed — receiver loop exited.
                        None => break,
                    }
                }
                // Race bore child exit: if bore dies unexpectedly surface it.
                Some(status) = async {
                    if let Some(child) = tunnel_handle.child_mut() {
                        child.wait().await.ok()
                    } else {
                        // Non-bore tunnel: never resolves, so this arm is
                        // disabled by the `Some(…)` pattern.
                        None
                    }
                } => {
                    let code = status.code().unwrap_or(-1);
                    bail!("bore tunnel exited unexpectedly (exit code {code})");
                }
            }
        }
        Ok(())
    }
    .await;

    // --- 12. Cleanup ----------------------------------------------------------
    cancel.cancel();

    // Delete the forwarding config we created. This is best-effort cleanup —
    // errors (including 404 if the config was already deleted) are logged as
    // warnings rather than propagating, so teardown always completes cleanly.
    if let Some(ref cfg) = created_config {
        if let Err(e) = c.delete_log_forwarding(&cfg.app).await {
            eprintln!(
                "Warning: failed to delete log-forwarding config for app {}: {e:#}",
                cfg.app,
            );
        } else {
            eprintln!("Log-forwarding config deleted for app {}.", cfg.app);
        }
    }

    // If --replace-existing was used, restore the prior config (best-effort).
    if let Some(prior) = prior_config {
        // The GET response is write-only for `token` (always null), and the
        // API rejects tokenless configs with an empty 400 — restoring with
        // `prior.token` verbatim would delete the user's config and then fail
        // to put it back. Substitute a fresh token and say so.
        let restore_token = match prior.token {
            Some(t) if !t.is_empty() => Some(t),
            _ => {
                eprintln!(
                    "Note: the previous config's token is not readable from the API; \
                     restoring with a newly generated token. If your syslog endpoint \
                     validates the token, update it via `container log-forwarding update`."
                );
                Some(generate_forwarding_token())
            }
        };
        let restore_req = LogForwardingRequest {
            app: prior.app.clone(),
            forwarding_type: prior.forwarding_type,
            endpoint: prior.endpoint.clone(),
            port: prior.port,
            format: prior.format,
            enabled: prior.enabled,
            token: restore_token,
        };
        if let Err(e) = c.create_log_forwarding(&restore_req).await {
            eprintln!(
                "Warning: failed to restore previous log-forwarding config for app {}: {e:#}",
                prior.app,
            );
        } else {
            eprintln!(
                "Restored previous log-forwarding config for app {}.",
                prior.app
            );
        }
    }

    // Stop the tunnel (best-effort).
    if let Err(e) = tunnel_handle.stop().await
        && debug
    {
        eprintln!("[debug] tunnel stop returned an error: {e:#}");
    }

    // Propagate any stream error after cleanup.
    stream_result
}

// ---------------------------------------------------------------------------
// Unit tests — pure parsing helpers (iter-76)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_volumes_ok() {
        let v = parse_volumes(&["data:10".to_owned(), "cache:5".to_owned()]).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].name, "data");
        assert_eq!(v[0].size, 10);
        assert_eq!(v[1].size, 5);
    }

    #[test]
    fn parse_volumes_rejects_bad_specs() {
        assert!(parse_volumes(&["data".to_owned()]).is_err());
        assert!(parse_volumes(&["data:0".to_owned()]).is_err());
        assert!(parse_volumes(&["data:-1".to_owned()]).is_err());
        assert!(parse_volumes(&["data:big".to_owned()]).is_err());
        assert!(parse_volumes(&[":10".to_owned()]).is_err());
    }

    #[test]
    fn parse_volume_mounts_preserves_paths() {
        let m = parse_volume_mounts(&["data:/var/lib/data".to_owned()]).unwrap();
        assert_eq!(m[0].name, "data");
        assert_eq!(m[0].mount_path, "/var/lib/data");
    }

    #[test]
    fn parse_volume_mounts_rejects_incomplete() {
        assert!(parse_volume_mounts(&["data".to_owned()]).is_err());
        assert!(parse_volume_mounts(&["data:".to_owned()]).is_err());
    }

    #[test]
    fn parse_pull_policy_case_insensitive() {
        assert_eq!(
            parse_pull_policy("always").unwrap(),
            ImagePullPolicy::Always
        );
        assert_eq!(
            parse_pull_policy("IfNotPresent").unwrap(),
            ImagePullPolicy::IfNotPresent
        );
        assert!(parse_pull_policy("never").is_err());
    }

    #[test]
    fn parse_protocol_case_insensitive() {
        assert_eq!(parse_protocol("tcp").unwrap(), Protocol::Tcp);
        assert_eq!(parse_protocol("UDP").unwrap(), Protocol::Udp);
        assert_eq!(parse_protocol("Sctp").unwrap(), Protocol::Sctp);
        assert!(parse_protocol("http").is_err());
    }

    #[test]
    fn parse_port_mapping_forms() {
        let m = parse_port_mapping("8080").unwrap();
        assert_eq!(m.container_port, 8080);
        assert!(m.exposed_port.is_none());
        assert!(m.protocols.is_none());

        let m = parse_port_mapping("8080:80").unwrap();
        assert_eq!(m.exposed_port, Some(80));

        let m = parse_port_mapping("8080:80:Tcp,Udp").unwrap();
        assert_eq!(m.protocols.as_ref().unwrap().len(), 2);

        let m = parse_port_mapping("53::Udp").unwrap();
        assert!(m.exposed_port.is_none());
        assert_eq!(m.protocols.as_ref().unwrap()[0], Protocol::Udp);
    }

    #[test]
    fn parse_port_mapping_rejects_bad() {
        assert!(parse_port_mapping("").is_err());
        assert!(parse_port_mapping("abc").is_err());
        assert!(parse_port_mapping("80:xx").is_err());
        assert!(parse_port_mapping("80:80:Http").is_err());
    }

    #[test]
    fn build_entry_point_none_when_empty() {
        assert!(build_entry_point(&None, &[], &None, &[], &None).is_none());
    }

    #[test]
    fn build_entry_point_array_wins_over_string() {
        let ep = build_entry_point(
            &Some("ignored".to_owned()),
            &["/bin/sh".to_owned()],
            &None,
            &[],
            &None,
        )
        .unwrap();
        assert!(ep.command.is_none());
        assert_eq!(ep.command_array.unwrap(), vec!["/bin/sh".to_owned()]);
    }

    #[test]
    fn build_sticky_sessions_rules() {
        // No flags → None.
        assert!(build_sticky_sessions(false, &[], &None).unwrap().is_none());
        // --sticky without header → error.
        assert!(build_sticky_sessions(true, &[], &None).is_err());
        // Too many headers → error.
        let four = vec![
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        ];
        assert!(build_sticky_sessions(true, &four, &None).is_err());
        // Valid.
        let s = build_sticky_sessions(true, &["X-User".to_owned()], &Some("sid".to_owned()))
            .unwrap()
            .unwrap();
        assert_eq!(s.enabled, Some(true));
        assert_eq!(s.cookie_name, Some("sid".to_owned()));
    }

    #[test]
    fn resolve_port_mappings_legacy_and_granular() {
        // Legacy single mapping.
        let m = resolve_port_mappings(Some(80), Some(8080), &[]).unwrap();
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].container_port, 80);
        assert_eq!(m[0].exposed_port, Some(8080));

        // Granular list.
        let m =
            resolve_port_mappings(None, None, &["80:80".to_owned(), "443:443".to_owned()]).unwrap();
        assert_eq!(m.len(), 2);

        // Both forms → error.
        assert!(resolve_port_mappings(Some(80), None, &["80".to_owned()]).is_err());
    }
}
