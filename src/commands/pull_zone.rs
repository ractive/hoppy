use crate::auth;
use crate::cli::{OutputFormat, PullZoneAction};
use crate::output::{self, PaginatedListJson};
use anyhow::Result;
use bunny_api_core::types::{CreatePullZone, PullZone, PurgeCache, UpdatePullZone};
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
) -> Result<()> {
    let client = auth::core_client(debug)?;

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
