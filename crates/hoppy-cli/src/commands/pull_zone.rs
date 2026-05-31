use crate::auth;
use crate::cli::{
    EdgeRuleAction, OutputFormat, PullZoneAction, PullZoneHostnameAction, PullZoneIpAction,
    PullZoneReferrerAction,
};
use crate::date;
use crate::output::{self, PaginatedListJson};
use crate::redact::RedactConfig;
use anyhow::{Context, Result, bail};
use bunny_net_api::core::CoreClient;
use bunny_net_api::core::types::{
    AddOrUpdateEdgeRule, CreatePullZone, EdgeRule, EdgeRuleActionType, EdgeRuleTrigger,
    LogAnonymizationType, MatchingType, OptimizerWatermarkPosition, PermaCacheType, PullZone,
    PullZoneLogForwarderProtocolType, PullZoneType, PurgeCache, TriggerType, UpdatePullZone,
};
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Display structs
// ---------------------------------------------------------------------------

/// Compact table row for list output.
#[derive(serde::Serialize, tabled::Tabled)]
struct PullZoneRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Origin")]
    origin_url: String,
    #[tabled(rename = "Enabled")]
    enabled: bool,
    #[tabled(rename = "Suspended")]
    suspended: bool,
    #[tabled(rename = "Bandwidth Used")]
    monthly_bandwidth_used: i64,
}

impl From<&PullZone> for PullZoneRow {
    fn from(pz: &PullZone) -> Self {
        Self {
            id: pz.id,
            name: pz.name.clone(),
            origin_url: pz.origin_url.clone(),
            enabled: pz.enabled,
            suspended: pz.suspended,
            monthly_bandwidth_used: pz.monthly_bandwidth_used,
        }
    }
}

// ---------------------------------------------------------------------------
// Log-forwarding precheck helper
// ---------------------------------------------------------------------------

/// Returns `true` when a GET precheck is required before sending the update.
///
/// A precheck is needed when:
/// - at least one log-forwarding sub-field (`hostname`, `port`, `token`,
///   `protocol`) is `Some`, AND
/// - `log_forwarding_enabled` is NOT `Some(true)` (i.e. the caller is not
///   atomically enabling + configuring in the same call).
fn lf_precheck_required(
    hostname: &Option<String>,
    port: &Option<u16>,
    token: &Option<String>,
    protocol: &Option<crate::cli::PullZoneLogForwardingProtocolArg>,
    enabled: &Option<bool>,
) -> bool {
    let has_sub_field =
        hostname.is_some() || port.is_some() || token.is_some() || protocol.is_some();
    has_sub_field && enabled != &Some(true)
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &PullZoneAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    let client = auth::core_client_with_reveal(debug, record, redact_cfg.reveal_all)?;

    match action {
        PullZoneAction::List {
            search,
            page,
            per_page,
        } => {
            let result = client
                .list_pull_zones(*page, *per_page, search.as_deref())
                .await?;
            if let OutputFormat::Json = format {
                let envelope = PaginatedListJson {
                    items: &result.items,
                    current_page: result.current_page,
                    total_items: result.total_items,
                    has_more_items: result.has_more_items,
                };
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<PullZoneRow> = result.items.iter().map(PullZoneRow::from).collect();
                output::print_data(&rows, format);
                if let Some(first) = result.items.first() {
                    let id = first.id;
                    output::hints::tips(&[
                        &format!("hoppy pull-zone get --id {id}"),
                        &format!("hoppy pull-zone statistics --id {id}"),
                    ]);
                }
            }
        }
        PullZoneAction::Get { id } => {
            let pz = client.get_pull_zone(*id).await?;
            print_pull_zone(&pz, format, redact_cfg);
        }
        PullZoneAction::Create {
            name,
            origin_url,
            storage_zone_id,
            zone_tier,
        } => {
            // Clap's ArgGroup guarantees exactly one of these is Some.
            let mut body = match (origin_url, storage_zone_id) {
                (Some(url), None) => CreatePullZone::new(name, url),
                (None, Some(id)) => CreatePullZone::for_storage_zone(name, *id),
                _ => unreachable!("clap ArgGroup enforces exactly one"),
            };
            body = body.zone_type(PullZoneType::from(*zone_tier));
            let pz = client.create_pull_zone(&body).await?;
            let new_id = pz.id;
            print_pull_zone(&pz, format, redact_cfg);
            output::hints::tips(&[
                &format!("hoppy pull-zone hostname add --id {new_id} --hostname <fqdn>"),
                &format!("hoppy pull-zone edge-rule add --id {new_id} ..."),
            ]);
        }
        PullZoneAction::Update {
            id,
            origin_url,
            storage_zone_id,
            monthly_bandwidth_limit,
            cache_expiration_time,
            zone_security_enabled,
            enable_geo_zone_us,
            enable_geo_zone_eu,
            enable_geo_zone_asia,
            enable_geo_zone_sa,
            enable_geo_zone_af,
            log_forwarding_enabled,
            log_forwarding_hostname,
            log_forwarding_port,
            log_forwarding_token,
            log_forwarding_protocol,
            logging_save_to_storage,
            logging_storage_zone_id,
            optimizer_enabled,
            optimizer_automatic_optimization,
            optimizer_desktop_max_width,
            optimizer_mobile_max_width,
            optimizer_image_quality,
            optimizer_mobile_image_quality,
            optimizer_webp,
            optimizer_upscaling,
            optimizer_minify_css,
            optimizer_minify_js,
            optimizer_manipulation_engine,
            optimizer_classes,
            optimizer_force_classes,
            optimizer_watermark,
            optimizer_watermark_url,
            optimizer_watermark_position,
            optimizer_watermark_offset,
            optimizer_watermark_min_image_size,
            optimizer_static_html,
            optimizer_static_html_wp_path,
            optimizer_static_html_wp_bypass_cookie,
            optimizer_prerender_html,
            optimizer_tunnel,
            enable_tls1,
            enable_tls1_1,
            enable_auto_ssl,
            disable_lets_encrypt,
            verify_origin_ssl,
            enable_access_control_origin_header,
            access_control_origin_header_extensions,
            zone_security_include_hash_remote_ip,
            aws_signing_enabled,
            aws_signing_key,
            aws_signing_secret,
            aws_signing_region_name,
            logging_ip_anonymization_enabled,
            log_anonymization_type,
            enable_webp_vary,
            enable_avif_vary,
            enable_cookie_vary,
            enable_country_code_vary,
            enable_country_state_code_vary,
            enable_hostname_vary,
            enable_mobile_vary,
            enable_cache_slice,
            enable_smart_cache,
            enable_safe_hop,
            ignore_query_strings,
            enable_query_string_ordering,
            query_string_vary_parameters,
            cookie_vary_parameters,
            use_stale_while_updating,
            use_stale_while_offline,
            use_background_update,
            cache_control_max_age_override,
            cache_control_public_max_age_override,
            cache_control_browser_max_age_override,
            cache_error_responses,
            perma_cache_storage_zone_id,
            perma_cache_type,
        } => {
            // Guard: if any log-forwarding sub-field is being set without also
            // enabling log forwarding in this same call, verify the zone has
            // log_forwarding_enabled=true; if not, bail with a clear message.
            if lf_precheck_required(
                log_forwarding_hostname,
                log_forwarding_port,
                log_forwarding_token,
                log_forwarding_protocol,
                log_forwarding_enabled,
            ) {
                let current = client.get_pull_zone(*id).await?;
                if !current.log_forwarding_enabled.unwrap_or(false) {
                    bail!(
                        "log-forwarding fields cannot be updated while disabled\n\
                         hint: pass --log-forwarding-enabled true to enable and update in one call"
                    );
                }
            }

            let mut body = UpdatePullZone::new();
            if let Some(url) = origin_url {
                body = body.origin_url(url);
            }
            if let Some(sz_id) = storage_zone_id {
                body = body.storage_zone_id(*sz_id);
            }
            if let Some(limit) = monthly_bandwidth_limit {
                body = body.monthly_bandwidth_limit(*limit);
            }
            if let Some(secs) = cache_expiration_time {
                body = body.cache_expiration_time(*secs);
            }
            if let Some(enabled) = zone_security_enabled {
                body = body.zone_security_enabled(*enabled);
            }
            body.enable_geo_zone_us = *enable_geo_zone_us;
            body.enable_geo_zone_eu = *enable_geo_zone_eu;
            body.enable_geo_zone_asia = *enable_geo_zone_asia;
            body.enable_geo_zone_sa = *enable_geo_zone_sa;
            body.enable_geo_zone_af = *enable_geo_zone_af;
            // Log forwarding fields
            if let Some(v) = log_forwarding_enabled {
                body = body.log_forwarding_enabled(*v);
            }
            if let Some(v) = log_forwarding_hostname {
                body = body.log_forwarding_hostname(v.as_str());
            }
            if let Some(v) = log_forwarding_port {
                body = body.log_forwarding_port(i32::from(*v));
            }
            if let Some(v) = log_forwarding_token {
                body = body.log_forwarding_token(v.as_str());
            }
            if let Some(p) = log_forwarding_protocol {
                body = body.log_forwarding_protocol(PullZoneLogForwarderProtocolType::from(*p));
            }
            if let Some(v) = logging_save_to_storage {
                body = body.logging_save_to_storage(*v);
            }
            if let Some(v) = logging_storage_zone_id {
                body = body.logging_storage_zone_id(*v);
            }
            // Optimizer fields
            body.optimizer_enabled = *optimizer_enabled;
            body.optimizer_automatic_optimization_enabled = *optimizer_automatic_optimization;
            body.optimizer_desktop_max_width = *optimizer_desktop_max_width;
            body.optimizer_mobile_max_width = *optimizer_mobile_max_width;
            body.optimizer_image_quality = *optimizer_image_quality;
            body.optimizer_mobile_image_quality = *optimizer_mobile_image_quality;
            body.optimizer_enable_web_p = *optimizer_webp;
            body.optimizer_enable_upscaling = *optimizer_upscaling;
            body.optimizer_minify_css = *optimizer_minify_css;
            body.optimizer_minify_java_script = *optimizer_minify_js;
            body.optimizer_enable_manipulation_engine = *optimizer_manipulation_engine;
            if let Some(cls) = optimizer_classes {
                body = body.optimizer_classes(cls.as_str());
            }
            body.optimizer_force_classes = *optimizer_force_classes;
            body.optimizer_watermark_enabled = *optimizer_watermark;
            if let Some(url) = optimizer_watermark_url {
                body = body.optimizer_watermark_url(url.as_str());
            }
            if let Some(pos) = optimizer_watermark_position {
                body = body.optimizer_watermark_position(OptimizerWatermarkPosition::from(*pos));
            }
            body.optimizer_watermark_offset = *optimizer_watermark_offset;
            body.optimizer_watermark_min_image_size = *optimizer_watermark_min_image_size;
            body.optimizer_static_html_enabled = *optimizer_static_html;
            if let Some(path) = optimizer_static_html_wp_path {
                body = body.optimizer_static_html_word_press_path(path.as_str());
            }
            if let Some(cookie) = optimizer_static_html_wp_bypass_cookie {
                body = body.optimizer_static_html_word_press_bypass_cookie(cookie.as_str());
            }
            body.optimizer_prerender_html = *optimizer_prerender_html;
            body.optimizer_tunnel_enabled = *optimizer_tunnel;
            // Security / compliance fields (iter-44)
            body.enable_tls1 = *enable_tls1;
            body.enable_tls1_1 = *enable_tls1_1;
            body.enable_auto_ssl = *enable_auto_ssl;
            body.disable_lets_encrypt = *disable_lets_encrypt;
            body.verify_origin_ssl = *verify_origin_ssl;
            body.enable_access_control_origin_header = *enable_access_control_origin_header;
            if let Some(exts) = access_control_origin_header_extensions.as_ref() {
                let normalized: Vec<String> = exts
                    .iter()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
                body.access_control_origin_header_extensions = Some(normalized);
            }
            body.zone_security_include_hash_remote_ip = *zone_security_include_hash_remote_ip;
            body.aws_signing_enabled = *aws_signing_enabled;
            if let Some(v) = aws_signing_key {
                body = body.aws_signing_key(v.as_str());
            }
            if let Some(v) = aws_signing_secret {
                body = body.aws_signing_secret(v.as_str());
            }
            if let Some(v) = aws_signing_region_name {
                body = body.aws_signing_region_name(v.as_str());
            }
            body.logging_ip_anonymization_enabled = *logging_ip_anonymization_enabled;
            if let Some(t) = log_anonymization_type {
                body = body.log_anonymization_type(LogAnonymizationType::from(*t));
            }
            // Vary headers (iter-45)
            body.enable_webp_vary = *enable_webp_vary;
            body.enable_avif_vary = *enable_avif_vary;
            body.enable_cookie_vary = *enable_cookie_vary;
            body.enable_country_code_vary = *enable_country_code_vary;
            body.enable_country_state_code_vary = *enable_country_state_code_vary;
            body.enable_hostname_vary = *enable_hostname_vary;
            body.enable_mobile_vary = *enable_mobile_vary;
            // Performance / caching (iter-45)
            body.enable_cache_slice = *enable_cache_slice;
            body.enable_smart_cache = *enable_smart_cache;
            body.enable_safe_hop = *enable_safe_hop;
            body.ignore_query_strings = *ignore_query_strings;
            body.enable_query_string_ordering = *enable_query_string_ordering;
            body.query_string_vary_parameters = query_string_vary_parameters.clone();
            body.cookie_vary_parameters = cookie_vary_parameters.clone();
            body.use_stale_while_updating = *use_stale_while_updating;
            body.use_stale_while_offline = *use_stale_while_offline;
            body.use_background_update = *use_background_update;
            body.cache_control_max_age_override = *cache_control_max_age_override;
            body.cache_control_public_max_age_override = *cache_control_public_max_age_override;
            body.cache_control_browser_max_age_override = *cache_control_browser_max_age_override;
            body.cache_error_responses = *cache_error_responses;
            body.perma_cache_storage_zone_id = *perma_cache_storage_zone_id;
            if let Some(t) = perma_cache_type {
                body = body.perma_cache_type(PermaCacheType::from(*t));
            }
            let pz = client.update_pull_zone(*id, &body).await?;
            print_pull_zone(&pz, format, redact_cfg);
        }
        PullZoneAction::Delete { id } => {
            if !yes {
                eprint!("Delete pull zone {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_pull_zone(*id).await?;
            eprintln!("Deleted pull zone {id}");
        }
        PullZoneAction::Purge { id, cache_tag } => {
            let body = match cache_tag {
                Some(tag) => PurgeCache::by_tag(tag),
                None => PurgeCache::all(),
            };
            client.purge_pull_zone_cache(*id, &body).await?;
            eprintln!("Purged cache for pull zone {id}");
        }
        PullZoneAction::Hostname { action } => {
            handle_hostname(&client, action).await?;
        }
        PullZoneAction::EdgeRule { action } => {
            handle_edge_rule(&client, action, format, yes).await?;
        }
        PullZoneAction::Referrer { action } => {
            handle_referrer(&client, action, format).await?;
        }
        PullZoneAction::Ip { action } => {
            handle_ip(&client, action, format).await?;
        }
        PullZoneAction::Statistics {
            id,
            r#type,
            date_from,
            date_to,
            hourly,
        } => {
            let df = date::normalise_datetime_opt(date_from.as_deref())?;
            let dt = date::normalise_datetime_opt(date_to.as_deref())?;
            let df = df.as_deref();
            let dt = dt.as_deref();
            match r#type.as_str() {
                "optimizer" => {
                    let stats = client
                        .get_pull_zone_optimizer_statistics(*id, df, dt, *hourly)
                        .await?;
                    if let OutputFormat::Json = format {
                        let json = serde_json::to_string_pretty(&stats)
                            .context("failed to serialize to JSON")?;
                        println!("{json}");
                    } else {
                        #[derive(serde::Serialize, tabled::Tabled)]
                        struct Row {
                            #[tabled(rename = "Metric")]
                            metric: String,
                            #[tabled(rename = "Value")]
                            value: String,
                        }
                        let rows = vec![
                            Row {
                                metric: "Total Requests Optimized".to_string(),
                                value: format!("{:.0}", stats.total_requests_optimized),
                            },
                            Row {
                                metric: "Total Traffic Saved".to_string(),
                                value: format!("{:.0}", stats.total_traffic_saved),
                            },
                            Row {
                                metric: "Avg Processing Time".to_string(),
                                value: format!("{:.2} ms", stats.average_processing_time),
                            },
                            Row {
                                metric: "Avg Compression Ratio".to_string(),
                                value: format!("{:.2}%", stats.average_compression_ratio),
                            },
                        ];
                        output::print_data(&rows, format);
                    }
                }
                "origin-shield" => {
                    let stats = client
                        .get_pull_zone_origin_shield_statistics(*id, df, dt, *hourly)
                        .await?;
                    if let OutputFormat::Json = format {
                        let json = serde_json::to_string_pretty(&stats)
                            .context("failed to serialize to JSON")?;
                        println!("{json}");
                    } else {
                        eprintln!(
                            "Origin shield queue statistics for pull zone {id} (use --format json for chart data)"
                        );
                    }
                }
                "safehop" => {
                    let stats = client
                        .get_pull_zone_safehop_statistics(*id, df, dt, *hourly)
                        .await?;
                    if let OutputFormat::Json = format {
                        let json = serde_json::to_string_pretty(&stats)
                            .context("failed to serialize to JSON")?;
                        println!("{json}");
                    } else {
                        #[derive(serde::Serialize, tabled::Tabled)]
                        struct Row {
                            #[tabled(rename = "Metric")]
                            metric: String,
                            #[tabled(rename = "Value")]
                            value: String,
                        }
                        let rows = vec![
                            Row {
                                metric: "Total Requests Retried".to_string(),
                                value: format!("{:.0}", stats.total_requests_retried),
                            },
                            Row {
                                metric: "Total Requests Saved".to_string(),
                                value: format!("{:.0}", stats.total_requests_saved),
                            },
                        ];
                        output::print_data(&rows, format);
                    }
                }
                other => {
                    bail!(
                        "unknown statistics type '{other}', expected: optimizer, origin-shield, safehop"
                    );
                }
            }
        }
    }

    Ok(())
}

async fn handle_hostname(client: &CoreClient, action: &PullZoneHostnameAction) -> Result<()> {
    match action {
        PullZoneHostnameAction::Add { id, hostname } => {
            client.add_hostname(*id, hostname).await?;
            eprintln!("Added hostname {hostname} to pull zone {id}");
        }
        PullZoneHostnameAction::Remove { id, hostname } => {
            client.remove_hostname(*id, hostname).await?;
            eprintln!("Removed hostname {hostname} from pull zone {id}");
        }
        PullZoneHostnameAction::LoadFreeCert { hostname } => {
            client.load_free_certificate(hostname).await?;
            eprintln!("Loaded free certificate for {hostname}");
        }
        PullZoneHostnameAction::ForceSsl {
            id,
            hostname,
            enabled,
        } => {
            client.set_force_ssl(*id, hostname, *enabled).await?;
            let status = if *enabled { "enabled" } else { "disabled" };
            eprintln!("Force SSL {status} for {hostname} on pull zone {id}");
        }
        PullZoneHostnameAction::AddCert {
            id,
            hostname,
            certificate,
            key,
        } => {
            client
                .add_certificate(*id, hostname, certificate, key)
                .await?;
            eprintln!("Added certificate for {hostname} on pull zone {id}");
        }
        PullZoneHostnameAction::RemoveCert { id, hostname } => {
            client.remove_certificate(*id, hostname).await?;
            eprintln!("Removed certificate for {hostname} on pull zone {id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Access-control helpers
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct ReferrerRow {
    #[tabled(rename = "Type")]
    kind: String,
    #[tabled(rename = "Hostname")]
    hostname: String,
}

#[derive(serde::Serialize, tabled::Tabled)]
struct IpRow {
    #[tabled(rename = "IP")]
    ip: String,
}

async fn handle_referrer(
    client: &CoreClient,
    action: &PullZoneReferrerAction,
    format: OutputFormat,
) -> Result<()> {
    match action {
        PullZoneReferrerAction::List { id } => {
            let pz = client.get_pull_zone(*id).await?;
            if let OutputFormat::Json = format {
                let payload = serde_json::json!({
                    "AllowedReferrers": pz.allowed_referrers,
                    "BlockedReferrers": pz.blocked_referrers,
                });
                let json = serde_json::to_string_pretty(&payload)
                    .context("failed to serialize referrers to JSON")?;
                println!("{json}");
            } else {
                let mut rows: Vec<ReferrerRow> = Vec::new();
                for h in &pz.allowed_referrers {
                    rows.push(ReferrerRow {
                        kind: "allowed".to_owned(),
                        hostname: h.clone(),
                    });
                }
                for h in &pz.blocked_referrers {
                    rows.push(ReferrerRow {
                        kind: "blocked".to_owned(),
                        hostname: h.clone(),
                    });
                }
                output::print_data(&rows, format);
            }
        }
        PullZoneReferrerAction::Allow { id, value } => {
            client.add_allowed_referrer(*id, value).await?;
            eprintln!("Allowed referrer {value} on pull zone {id}");
        }
        PullZoneReferrerAction::RemoveAllowed { id, value } => {
            client.remove_allowed_referrer(*id, value).await?;
            eprintln!("Removed allowed referrer {value} from pull zone {id}");
        }
        PullZoneReferrerAction::Block { id, value } => {
            client.add_blocked_referrer(*id, value).await?;
            eprintln!("Blocked referrer {value} on pull zone {id}");
        }
        PullZoneReferrerAction::RemoveBlocked { id, value } => {
            client.remove_blocked_referrer(*id, value).await?;
            eprintln!("Removed blocked referrer {value} from pull zone {id}");
        }
    }
    Ok(())
}

async fn handle_ip(
    client: &CoreClient,
    action: &PullZoneIpAction,
    format: OutputFormat,
) -> Result<()> {
    match action {
        PullZoneIpAction::List { id } => {
            let pz = client.get_pull_zone(*id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&pz.blocked_ips)
                    .context("failed to serialize blocked IPs to JSON")?;
                println!("{json}");
            } else {
                let rows: Vec<IpRow> = pz
                    .blocked_ips
                    .iter()
                    .map(|ip| IpRow { ip: ip.clone() })
                    .collect();
                output::print_data(&rows, format);
            }
        }
        PullZoneIpAction::Block { id, value } => {
            client.add_blocked_ip(*id, value).await?;
            eprintln!("Blocked IP {value} on pull zone {id}");
        }
        PullZoneIpAction::Unblock { id, value } => {
            client.remove_blocked_ip(*id, value).await?;
            eprintln!("Unblocked IP {value} on pull zone {id}");
        }
    }
    Ok(())
}

/// Output a single PullZone as a vertical Field/Value table (or JSON).
/// Redacts secret-bearing fields (e.g. `LogForwardingToken`) unless
/// `redact_cfg` has `reveal_all` set.
fn print_pull_zone(pz: &PullZone, format: OutputFormat, redact_cfg: &RedactConfig) {
    let cmd = format!("pull-zone get --id {}", pz.id);
    output::print_single_vertical_with_cmd(pz, format, redact_cfg, Some(&cmd));
}

// ---------------------------------------------------------------------------
// Edge rule helpers
// ---------------------------------------------------------------------------

/// Compact table row for edge rule list output.
#[derive(Clone, serde::Serialize, tabled::Tabled)]
struct EdgeRuleRow {
    #[tabled(rename = "GUID")]
    guid: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Action")]
    action: String,
    #[tabled(rename = "Triggers")]
    triggers: String,
    #[tabled(rename = "Enabled")]
    enabled: bool,
}

impl From<&EdgeRule> for EdgeRuleRow {
    fn from(r: &EdgeRule) -> Self {
        let action = r.action_type.map(|a| a.to_string()).unwrap_or_default();
        let trigger_summary = if r.triggers.is_empty() {
            "none".to_string()
        } else {
            format!("{} trigger(s)", r.triggers.len())
        };
        Self {
            guid: r.guid.clone().unwrap_or_default(),
            description: r.description.clone().unwrap_or_default(),
            action,
            triggers: trigger_summary,
            enabled: r.enabled,
        }
    }
}

/// Parse a `--trigger` flag value like `url:*.jpg,*.png` into an `EdgeRuleTrigger`.
fn parse_trigger(raw: &str) -> Result<EdgeRuleTrigger> {
    let (type_str, patterns_str) = raw
        .split_once(':')
        .context("trigger must be in type:pattern1,pattern2 format")?;
    let trigger_type: TriggerType = type_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;
    let patterns: Vec<String> = patterns_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(EdgeRuleTrigger {
        trigger_type: Some(trigger_type),
        pattern_matches: patterns,
        pattern_matching_type: Some(MatchingType::MatchAny),
        parameter1: None,
    })
}

/// Build an `AddOrUpdateEdgeRule` from CLI flags.
fn build_edge_rule_body(
    guid: Option<&str>,
    action_type: &str,
    action_param1: Option<&str>,
    action_param2: Option<&str>,
    trigger_matching_type: &str,
    triggers: &[String],
    description: Option<&str>,
) -> Result<AddOrUpdateEdgeRule> {
    let action: EdgeRuleActionType = action_type
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;
    let matching: MatchingType = trigger_matching_type
        .parse()
        .map_err(|e: String| anyhow::anyhow!("{e}"))?;

    let mut body = AddOrUpdateEdgeRule::new(action).trigger_matching_type(matching);

    if let Some(g) = guid {
        body = body.guid(g);
    }
    if let Some(p1) = action_param1 {
        body = body.action_parameter1(p1);
    }
    if let Some(p2) = action_param2 {
        body = body.action_parameter2(p2);
    }
    if let Some(desc) = description {
        body = body.description(desc);
    }
    for raw in triggers {
        body = body.trigger(parse_trigger(raw)?);
    }
    Ok(body)
}

async fn handle_edge_rule(
    client: &CoreClient,
    action: &EdgeRuleAction,
    format: OutputFormat,
    yes: bool,
) -> Result<()> {
    match action {
        EdgeRuleAction::List { id } => {
            let pz = client.get_pull_zone(*id).await?;
            if let OutputFormat::Json = format {
                let json = serde_json::to_string_pretty(&pz.edge_rules)
                    .context("failed to serialize edge rules to JSON")?;
                println!("{json}");
            } else {
                let rows: Vec<EdgeRuleRow> = pz.edge_rules.iter().map(EdgeRuleRow::from).collect();
                if let OutputFormat::Table = format {
                    let mut truncated_rows = rows.clone();
                    let mut any_truncated = false;
                    for row in &mut truncated_rows {
                        let (v, t) = crate::output::truncate_for_table(
                            &row.description,
                            crate::output::TABLE_CELL_MAX,
                        );
                        row.description = v;
                        any_truncated |= t;
                    }
                    output::print_data(&truncated_rows, format);
                    if any_truncated {
                        output::hints::tip(
                            "some Description values were truncated — use --format json for full values",
                        );
                    }
                } else {
                    output::print_data(&rows, format);
                }
            }
        }
        EdgeRuleAction::Add {
            id,
            description,
            action_type,
            action_param1,
            action_param2,
            trigger_matching_type,
            triggers,
        } => {
            let body = build_edge_rule_body(
                None,
                action_type,
                action_param1.as_deref(),
                action_param2.as_deref(),
                trigger_matching_type,
                triggers,
                description.as_deref(),
            )?;
            client.add_or_update_edge_rule(*id, &body).await?;
            eprintln!("Added edge rule to pull zone {id}");
        }
        EdgeRuleAction::Update {
            id,
            rule_id,
            description,
            action_type,
            action_param1,
            action_param2,
            trigger_matching_type,
            triggers,
        } => {
            let body = build_edge_rule_body(
                Some(rule_id),
                action_type,
                action_param1.as_deref(),
                action_param2.as_deref(),
                trigger_matching_type,
                triggers,
                description.as_deref(),
            )?;
            client.add_or_update_edge_rule(*id, &body).await?;
            eprintln!("Updated edge rule {rule_id} on pull zone {id}");
        }
        EdgeRuleAction::Delete { id, rule_id } => {
            if !yes {
                eprint!("Delete edge rule {rule_id} from pull zone {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_edge_rule(*id, rule_id).await?;
            eprintln!("Deleted edge rule {rule_id} from pull zone {id}");
        }
        EdgeRuleAction::Enable {
            id,
            rule_id,
            enabled,
        } => {
            client.set_edge_rule_enabled(*id, rule_id, *enabled).await?;
            let status = if *enabled { "Enabled" } else { "Disabled" };
            eprintln!("{status} edge rule {rule_id} on pull zone {id}");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lf_precheck_hostname_only_no_enabled_requires_check() {
        assert!(lf_precheck_required(
            &Some("logs.example.com".to_owned()),
            &None,
            &None,
            &None,
            &None,
        ));
    }

    #[test]
    fn lf_precheck_hostname_with_enabled_true_skips_check() {
        assert!(!lf_precheck_required(
            &Some("logs.example.com".to_owned()),
            &None,
            &None,
            &None,
            &Some(true),
        ));
    }

    #[test]
    fn lf_precheck_hostname_with_enabled_false_requires_check() {
        assert!(lf_precheck_required(
            &Some("logs.example.com".to_owned()),
            &None,
            &None,
            &None,
            &Some(false),
        ));
    }

    #[test]
    fn lf_precheck_no_sub_fields_no_check() {
        assert!(!lf_precheck_required(&None, &None, &None, &None, &None));
    }

    #[test]
    fn lf_precheck_port_only_requires_check() {
        assert!(lf_precheck_required(&None, &Some(514), &None, &None, &None));
    }

    #[test]
    fn lf_precheck_token_only_requires_check() {
        assert!(lf_precheck_required(
            &None,
            &None,
            &Some("tok".to_owned()),
            &None,
            &None,
        ));
    }

    #[test]
    fn lf_precheck_protocol_only_requires_check() {
        use crate::cli::PullZoneLogForwardingProtocolArg;
        assert!(lf_precheck_required(
            &None,
            &None,
            &None,
            &Some(PullZoneLogForwardingProtocolArg::Udp),
            &None,
        ));
    }
}
