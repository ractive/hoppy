use crate::auth;
use crate::cli::{EdgeRuleAction, OutputFormat, PullZoneAction, PullZoneHostnameAction};
use crate::output::{self, PaginatedListJson};
use anyhow::{Context, Result, bail};
use bunny_api_core::CoreClient;
use bunny_api_core::types::{
    AddOrUpdateEdgeRule, CreatePullZone, EdgeRule, EdgeRuleActionType, EdgeRuleTrigger,
    MatchingType, PullZone, PurgeCache, TriggerType, UpdatePullZone,
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

/// Detailed single-item view for get/create/update output.
#[derive(serde::Serialize, tabled::Tabled)]
struct PullZoneDetail {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Origin URL")]
    origin_url: String,
    #[tabled(rename = "CNAME")]
    cname_domain: String,
    #[tabled(rename = "Type")]
    zone_type: String,
    #[tabled(rename = "Enabled")]
    enabled: bool,
    #[tabled(rename = "Suspended")]
    suspended: bool,
    #[tabled(rename = "Bandwidth Used")]
    monthly_bandwidth_used: i64,
    #[tabled(rename = "Bandwidth Limit")]
    monthly_bandwidth_limit: i64,
    #[tabled(rename = "Hostnames")]
    hostnames: String,
}

impl From<&PullZone> for PullZoneDetail {
    fn from(pz: &PullZone) -> Self {
        let zone_type = pz
            .zone_type
            .map(|t| t.to_string())
            .unwrap_or_else(|| "-".to_owned());
        let hostnames = if pz.hostnames.is_empty() {
            "-".to_owned()
        } else {
            pz.hostnames
                .iter()
                .map(|h| h.value.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        Self {
            id: pz.id,
            name: pz.name.clone(),
            origin_url: pz.origin_url.clone(),
            cname_domain: pz.cname_domain.clone(),
            zone_type,
            enabled: pz.enabled,
            suspended: pz.suspended,
            monthly_bandwidth_used: pz.monthly_bandwidth_used,
            monthly_bandwidth_limit: pz.monthly_bandwidth_limit,
            hostnames,
        }
    }
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
) -> Result<()> {
    let client = auth::core_client(debug, record)?;

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
            }
        }
        PullZoneAction::Get { id } => {
            let pz = client.get_pull_zone(*id).await?;
            print_pull_zone(&pz, format);
        }
        PullZoneAction::Create { name, origin_url } => {
            let body = CreatePullZone::new(name, origin_url);
            let pz = client.create_pull_zone(&body).await?;
            print_pull_zone(&pz, format);
        }
        PullZoneAction::Update {
            id,
            origin_url,
            monthly_bandwidth_limit,
            cache_expiration_time,
            zone_security_enabled,
            enable_geo_zone_us,
            enable_geo_zone_eu,
            enable_geo_zone_asia,
            enable_geo_zone_sa,
            enable_geo_zone_af,
        } => {
            let mut body = UpdatePullZone::new();
            if let Some(url) = origin_url {
                body = body.origin_url(url);
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
            let pz = client.update_pull_zone(*id, &body).await?;
            print_pull_zone(&pz, format);
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
        PullZoneAction::Statistics {
            id,
            r#type,
            date_from,
            date_to,
            hourly,
        } => {
            let df = date_from.as_deref();
            let dt = date_to.as_deref();
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

/// Output a single PullZone: full JSON for JSON format, detail struct otherwise.
fn print_pull_zone(pz: &PullZone, format: OutputFormat) {
    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(pz).expect("failed to serialize to JSON");
        println!("{json}");
    } else {
        let detail = PullZoneDetail::from(pz);
        output::print_single(&detail, format);
    }
}

// ---------------------------------------------------------------------------
// Edge rule helpers
// ---------------------------------------------------------------------------

/// Compact table row for edge rule list output.
#[derive(serde::Serialize, tabled::Tabled)]
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
                output::print_data(&rows, format);
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
