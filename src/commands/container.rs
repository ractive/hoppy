use crate::auth;
use crate::cli::{
    ContainerAction, ContainerAppAction, ContainerEndpointAction, ContainerLogForwardingAction,
    ContainerNodeAction, ContainerPodAction, ContainerRegionAction, ContainerRegistryAction,
    ContainerTemplateAction, ContainerVolumeAction, OutputFormat,
};
use crate::output;
use anyhow::{Context, Result, bail};
use bunny_api_containers::{
    AddApplicationRequest, AddContainerRequest, AnycastEndpointRequest, AnycastIpProtocolVersion,
    AutoscalingSettings, CdnEndpointRequest, ContainerConfigSuggestions, ContainerImage,
    ContainerImageTag, ContainerPortMappingRequest, ContainerRegistryRequest, ContainersClient,
    EndpointRequest, GetContainerConfigSuggestionsRequest, GetContainerImageDigestRequest,
    Granularity, ImageTagInfo, ListContainerImageTagsRequest, LogForwardingRequest,
    PatchApplicationRequest, PatchContainerRequest, PatchVolumeRequest, RegionSettings,
    RegistryCredentials, SearchPublicContainerImagesRequest, UpdateRegionSettingsRequest,
};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Helper: build the client
// ---------------------------------------------------------------------------

fn client(debug: bool, record: Option<&str>) -> Result<ContainersClient> {
    auth::containers_client(debug, record)
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

impl From<&bunny_api_containers::AppListItem> for AppRow {
    fn from(a: &bunny_api_containers::AppListItem) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            status: format!("{:?}", a.status),
        }
    }
}

#[derive(serde::Serialize, tabled::Tabled)]
struct AppDetail {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Runtime")]
    runtime_type: String,
    #[tabled(rename = "Min Instances")]
    min: i32,
    #[tabled(rename = "Max Instances")]
    max: i32,
}

impl From<&bunny_api_containers::Application> for AppDetail {
    fn from(a: &bunny_api_containers::Application) -> Self {
        let (min, max) = a
            .auto_scaling
            .as_ref()
            .map(|s| (s.min, s.max))
            .unwrap_or((0, 0));
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            status: format!("{:?}", a.status),
            runtime_type: format!("{:?}", a.runtime_type),
            min,
            max,
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

impl From<&bunny_api_containers::ContainerTemplate> for ContainerTemplateRow {
    fn from(t: &bunny_api_containers::ContainerTemplate) -> Self {
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

impl From<&bunny_api_containers::EndpointListItem> for EndpointRow {
    fn from(e: &bunny_api_containers::EndpointListItem) -> Self {
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

impl From<&bunny_api_containers::VolumeInList> for VolumeRow {
    fn from(v: &bunny_api_containers::VolumeInList) -> Self {
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

impl From<&bunny_api_containers::ContainerRegistry> for RegistryRow {
    fn from(r: &bunny_api_containers::ContainerRegistry) -> Self {
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
struct RegionRow {
    #[tabled(rename = "ID")]
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

impl From<&bunny_api_containers::Region> for RegionRow {
    fn from(r: &bunny_api_containers::Region) -> Self {
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

impl From<&bunny_api_containers::UserLimits> for UserLimitsRow {
    fn from(l: &bunny_api_containers::UserLimits) -> Self {
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

impl From<&bunny_api_containers::LogForwardingConfiguration> for LogForwardingRow {
    fn from(l: &bunny_api_containers::LogForwardingConfiguration) -> Self {
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

// ---------------------------------------------------------------------------
// Top-level handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &ContainerAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        ContainerAction::App { action } => handle_app(action, format, debug, yes, record).await,
        ContainerAction::Template { action } => {
            handle_template(action, format, debug, yes, record).await
        }
        ContainerAction::Endpoint { action } => {
            handle_endpoint(action, format, debug, yes, record).await
        }
        ContainerAction::Volume { action } => {
            handle_volume(action, format, debug, yes, record).await
        }
        ContainerAction::Registry { action } => {
            handle_registry(action, format, debug, yes, record).await
        }
        ContainerAction::Region { action } => handle_region(action, format, debug, record).await,
        ContainerAction::Node { action } => handle_node(action, format, debug, record).await,
        ContainerAction::Pod { action } => handle_pod(action, debug, record).await,
        ContainerAction::Limits => handle_limits(format, debug, record).await,
        ContainerAction::LogForwarding { action } => {
            handle_log_forwarding(action, format, debug, yes, record).await
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
) -> Result<()> {
    let c = client(debug, record)?;
    match action {
        ContainerAppAction::List { cursor, limit } => {
            let result = c
                .list_applications(cursor.as_deref(), limit.as_ref().copied())
                .await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<AppRow> = result.items.iter().map(AppRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerAppAction::Get { id } => {
            let app = c.get_application(id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&app).context("failed to serialize to JSON")?
                );
            } else {
                let row = AppDetail::from(&app);
                output::print_single(&row, format);
            }
        }
        ContainerAppAction::Create {
            name,
            runtime_type,
            min,
            max,
            regions,
        } => {
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
                container_templates: None,
                volumes: None,
            };
            let resp = c.add_application(&body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                eprintln!("Created application: {}", resp.id);
            }
        }
        ContainerAppAction::Update {
            id,
            name,
            runtime_type,
            min,
            max,
        } => {
            if name.is_none() && runtime_type.is_none() && min.is_none() && max.is_none() {
                bail!(
                    "at least one update flag is required (--name, --runtime-type, --min, --max)"
                );
            }
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
                ..Default::default()
            };
            let resp = c.patch_application(id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                eprintln!("Updated application: {}", resp.id);
            }
        }
        ContainerAppAction::Deploy { id } => {
            c.deploy_application(id).await?;
            eprintln!("Deployed application {id}");
        }
        ContainerAppAction::Undeploy { id } => {
            if !yes && !confirm(format!("Undeploy application {id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            c.undeploy_application(id).await?;
            eprintln!("Undeployed application {id}");
        }
        ContainerAppAction::Restart { id } => {
            c.restart_application(id).await?;
            eprintln!("Restarted application {id}");
        }
        ContainerAppAction::Delete { id } => {
            if !yes && !confirm(format!("Delete application {id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            c.delete_application(id).await?;
            eprintln!("Deleted application {id}");
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
            eprintln!("Updated autoscaling for application {app_id}");
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
            eprintln!("Updated region settings for application {app_id}");
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
) -> Result<()> {
    let c = client(debug, record)?;
    match action {
        ContainerTemplateAction::Get {
            app_id,
            container_id,
        } => {
            let tmpl = c.get_container(app_id, container_id).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tmpl).context("failed to serialize to JSON")?
                );
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
        } => {
            let body = AddContainerRequest {
                name: name.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                image_tag: image_tag.clone(),
                image_registry_id: registry_id.clone(),
                image: None,
                image_digest: None,
                image_pull_policy: None,
                entry_point: None,
                probes: None,
                environment_variables: None,
                endpoints: None,
                volume_mounts: None,
            };
            let tmpl = c.add_container(app_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tmpl).context("failed to serialize to JSON")?
                );
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
        } => {
            if name.is_none()
                && image_tag.is_none()
                && image_name.is_none()
                && image_namespace.is_none()
                && registry_id.is_none()
            {
                bail!("at least one update flag is required");
            }
            let body = PatchContainerRequest {
                name: name.clone(),
                image_tag: image_tag.clone(),
                image_name: image_name.clone(),
                image_namespace: image_namespace.clone(),
                image_registry_id: registry_id.clone(),
                ..Default::default()
            };
            let tmpl = c.patch_container(app_id, container_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tmpl).context("failed to serialize to JSON")?
                );
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
            eprintln!("Deleted container template {container_id}");
        }
        ContainerTemplateAction::Env {
            app_id,
            container_id,
            env,
        } => {
            let map = parse_env_pairs(env)?;
            let tmpl = c.set_container_env(app_id, container_id, &map).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tmpl).context("failed to serialize to JSON")?
                );
            } else {
                let row = ContainerTemplateRow::from(&tmpl);
                output::print_single(&row, format);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Endpoint sub-handlers
// ---------------------------------------------------------------------------

fn build_endpoint_request(
    name: &str,
    container_port: i32,
    exposed_port: Option<i32>,
    cdn: bool,
    anycast: bool,
) -> Result<EndpointRequest> {
    let port_mapping = ContainerPortMappingRequest {
        container_port,
        exposed_port,
        protocols: None,
    };

    if cdn {
        Ok(EndpointRequest {
            display_name: name.to_owned(),
            cdn: Some(CdnEndpointRequest {
                is_ssl_enabled: None,
                sticky_sessions: None,
                pull_zone_id: None,
                port_mappings: Some(vec![port_mapping]),
            }),
            anycast: None,
        })
    } else if anycast {
        Ok(EndpointRequest {
            display_name: name.to_owned(),
            cdn: None,
            anycast: Some(AnycastEndpointRequest {
                protocol_version: AnycastIpProtocolVersion::IPv4,
                port_mappings: vec![port_mapping],
            }),
        })
    } else {
        // Default: CDN
        Ok(EndpointRequest {
            display_name: name.to_owned(),
            cdn: Some(CdnEndpointRequest {
                is_ssl_enabled: None,
                sticky_sessions: None,
                pull_zone_id: None,
                port_mappings: Some(vec![port_mapping]),
            }),
            anycast: None,
        })
    }
}

async fn handle_endpoint(
    action: &ContainerEndpointAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
) -> Result<()> {
    let c = client(debug, record)?;
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
            cdn,
            anycast,
        } => {
            let body =
                build_endpoint_request(name, *container_port, *exposed_port, *cdn, *anycast)?;
            let resp = c.add_endpoint(app_id, container_id, &body).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                eprintln!("Created endpoint: {}", resp.id);
            }
        }
        ContainerEndpointAction::Update {
            app_id,
            endpoint_id,
            name,
            container_port,
            exposed_port,
            cdn,
            anycast,
        } => {
            let body =
                build_endpoint_request(name, *container_port, *exposed_port, *cdn, *anycast)?;
            c.update_endpoint(app_id, endpoint_id, &body).await?;
            eprintln!("Updated endpoint {endpoint_id}");
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
            eprintln!("Deleted endpoint {endpoint_id}");
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
) -> Result<()> {
    let c = client(debug, record)?;
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
                eprintln!("Updated volume: {} ({:.1} GB)", resp.name, resp.size);
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
            eprintln!("Deleted {} volume instance(s)", resp.ids.len());
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
            eprintln!("Deleted volume instance: {}", resp.id);
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
) -> Result<()> {
    let c = client(debug, record)?;
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
                eprintln!("Created registry: {} (status: {:?})", id_str, resp.status);
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
                eprintln!("Updated registry {} (status: {:?})", id, resp.status);
            }
        }
        ContainerRegistryAction::Delete { id } => {
            if !yes && !confirm(format!("Delete registry {id}?")).await? {
                eprintln!("Aborted.");
                return Ok(());
            }
            let resp = c.delete_registry(*id).await?;
            eprintln!("Deleted registry (status: {:?})", resp.status);
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
) -> Result<()> {
    let c = client(debug, record)?;
    match action {
        ContainerRegionAction::List { cursor, limit } => {
            let result = c
                .list_regions(cursor.as_deref(), limit.as_ref().copied())
                .await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?
                );
            } else {
                let rows: Vec<RegionRow> = result.items.iter().map(RegionRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        ContainerRegionAction::Optimal => {
            let resp = c.get_optimal_region(None).await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resp).context("failed to serialize to JSON")?
                );
            } else {
                let row = RegionRow::from(&resp.region);
                output::print_single(&row, format);
            }
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
) -> Result<()> {
    let c = client(debug, record)?;
    match action {
        ContainerNodeAction::List { cursor, limit } => {
            let result = c
                .list_nodes(cursor.as_deref(), limit.as_ref().copied())
                .await?;
            if let OutputFormat::Json = format {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).context("failed to serialize to JSON")?
                );
            } else {
                // Nodes are strings — print one per line
                for node in &result.items {
                    println!("{node}");
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Pod sub-handlers
// ---------------------------------------------------------------------------

async fn handle_pod(action: &ContainerPodAction, debug: bool, record: Option<&str>) -> Result<()> {
    let c = client(debug, record)?;
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

async fn handle_limits(format: OutputFormat, debug: bool, record: Option<&str>) -> Result<()> {
    let c = client(debug, record)?;
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
) -> Result<()> {
    let c = client(debug, record)?;
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
        ContainerLogForwardingAction::Get { app_id } => {
            let config = c.get_log_forwarding(app_id).await?;
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
        ContainerLogForwardingAction::Create {
            app_id,
            forwarding_type,
            endpoint,
            port,
            format: fmt_str,
            token,
            enabled,
        } => {
            let body = LogForwardingRequest {
                app: app_id.clone(),
                forwarding_type: forwarding_type.parse().map_err(anyhow::Error::msg)?,
                endpoint: endpoint.clone(),
                port: *port,
                token: token.clone(),
                format: fmt_str.parse().map_err(anyhow::Error::msg)?,
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
            format: fmt_str,
            token,
            enabled,
        } => {
            let body = LogForwardingRequest {
                app: app_id.clone(),
                forwarding_type: forwarding_type.parse().map_err(anyhow::Error::msg)?,
                endpoint: endpoint.clone(),
                port: *port,
                token: token.clone(),
                format: fmt_str.parse().map_err(anyhow::Error::msg)?,
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
            eprintln!("Deleted log forwarding configuration for app {app_id}");
        }
    }
    Ok(())
}
