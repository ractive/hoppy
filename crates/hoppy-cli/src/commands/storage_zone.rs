use crate::auth;
use crate::cli::{OutputFormat, StorageZoneAction};
use crate::date;
use crate::output::{self, PaginatedListJson};
use crate::redact::{RedactConfig, redact_secrets_in_json};
use anyhow::{Context, Result, bail};
use bunny_api::core::types::{CreateStorageZone, StorageZone, UpdateStorageZone};
use std::io::{self, BufRead, Write};

// ---------------------------------------------------------------------------
// Display structs
// ---------------------------------------------------------------------------

/// Compact table row for list output.
#[derive(serde::Serialize, tabled::Tabled)]
struct StorageZoneRow {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Region")]
    region: String,
    #[tabled(rename = "Storage Used")]
    storage_used: i64,
    #[tabled(rename = "Files Stored")]
    files_stored: i64,
    #[tabled(rename = "Zone Tier")]
    zone_tier: i64,
}

impl From<&StorageZone> for StorageZoneRow {
    fn from(sz: &StorageZone) -> Self {
        Self {
            id: sz.id,
            name: sz.name.clone(),
            region: sz.region.clone(),
            storage_used: sz.storage_used,
            files_stored: sz.files_stored,
            zone_tier: sz.zone_tier,
        }
    }
}

/// Detailed single-item view for get/create output.
#[derive(serde::Serialize, tabled::Tabled)]
struct StorageZoneDetail {
    #[tabled(rename = "ID")]
    id: i64,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Region")]
    region: String,
    #[tabled(rename = "Hostname")]
    storage_hostname: String,
    #[tabled(rename = "Storage Used")]
    storage_used: i64,
    #[tabled(rename = "Files Stored")]
    files_stored: i64,
    #[tabled(rename = "Zone Tier")]
    zone_tier: i64,
    #[tabled(rename = "Replication Regions")]
    replication_regions: String,
    #[tabled(rename = "Rewrite 404→200")]
    rewrite_404_to_200: bool,
    #[tabled(rename = "Custom 404 Path")]
    custom_404_file_path: String,
    #[tabled(rename = "Date Modified")]
    date_modified: String,
    #[tabled(rename = "Deleted")]
    deleted: bool,
}

impl From<&StorageZone> for StorageZoneDetail {
    fn from(sz: &StorageZone) -> Self {
        let replication_regions = if sz.replication_regions.is_empty() {
            "-".to_owned()
        } else {
            sz.replication_regions.join(", ")
        };
        Self {
            id: sz.id,
            name: sz.name.clone(),
            region: sz.region.clone(),
            storage_hostname: sz.storage_hostname.clone(),
            storage_used: sz.storage_used,
            files_stored: sz.files_stored,
            zone_tier: sz.zone_tier,
            replication_regions,
            rewrite_404_to_200: sz.rewrite_404_to_200,
            custom_404_file_path: sz
                .custom_404_file_path
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("-")
                .to_owned(),
            date_modified: sz.date_modified.clone(),
            deleted: sz.deleted,
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &StorageZoneAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
    record: Option<&str>,
    redact_cfg: &RedactConfig,
) -> Result<()> {
    let client = auth::core_client(debug, record)?;

    match action {
        StorageZoneAction::List {
            search,
            page,
            per_page,
        } => {
            let result = client
                .list_storage_zones(*page, *per_page, search.as_deref(), None)
                .await?;
            if let OutputFormat::Json = format {
                let envelope = PaginatedListJson {
                    items: &result.items,
                    current_page: result.current_page,
                    total_items: result.total_items,
                    has_more_items: result.has_more_items,
                };
                let mut value = serde_json::to_value(&envelope)
                    .context("failed to serialize storage zone list to JSON")?;
                redact_secrets_in_json(&mut value, redact_cfg);
                let json =
                    serde_json::to_string_pretty(&value).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let rows: Vec<StorageZoneRow> =
                    result.items.iter().map(StorageZoneRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        StorageZoneAction::Get { id } => {
            let sz = client.get_storage_zone(*id).await?;
            print_storage_zone(&sz, format, redact_cfg);
        }
        StorageZoneAction::Create {
            name,
            region,
            replication_regions,
            zone_tier,
        } => {
            let mut body = CreateStorageZone::new(name, region);
            if !replication_regions.is_empty() {
                body = body.replication_regions(replication_regions.clone());
            }
            if let Some(tier) = zone_tier {
                body = body.zone_tier(i64::from(*tier));
            }
            let sz = client.create_storage_zone(&body).await?;
            print_storage_zone(&sz, format, redact_cfg);
        }
        StorageZoneAction::Update {
            id,
            rewrite_404_to_200,
            custom_404_file_path,
            origin_url,
        } => {
            if rewrite_404_to_200.is_none()
                && custom_404_file_path.is_none()
                && origin_url.is_none()
            {
                bail!(
                    "at least one update flag is required (--rewrite-404-to-200, --custom-404-file-path, or --origin-url)"
                );
            }
            let mut body = UpdateStorageZone::new();
            if let Some(rewrite) = rewrite_404_to_200 {
                body = body.rewrite_404_to_200(*rewrite);
            }
            if let Some(path) = custom_404_file_path {
                body = body.custom_404_file_path(path);
            }
            if let Some(url) = origin_url {
                body = body.origin_url(url);
            }
            client.update_storage_zone(*id, &body).await?;
            eprintln!("Updated storage zone {id}");
        }
        StorageZoneAction::Delete { id } => {
            if !yes {
                eprint!("Delete storage zone {id}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            client.delete_storage_zone(*id).await?;
            eprintln!("Deleted storage zone {id}");
        }
        StorageZoneAction::Statistics {
            id,
            date_from,
            date_to,
        } => {
            let date_from = date::normalise_datetime_opt(date_from.as_deref())?;
            let date_to = date::normalise_datetime_opt(date_to.as_deref())?;
            let stats = client
                .get_storage_zone_statistics(*id, date_from.as_deref(), date_to.as_deref())
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                eprintln!("Storage zone {id} statistics (use --format json for chart data)");
                if let Some(chart) = &stats.storage_used_chart
                    && let Some(latest) = chart.values().max()
                {
                    eprintln!("  Peak storage used: {latest} bytes");
                }
                if let Some(chart) = &stats.file_count_chart
                    && let Some(latest) = chart.values().max()
                {
                    eprintln!("  Peak file count: {latest}");
                }
            }
        }
    }

    Ok(())
}

/// Output a single StorageZone: full JSON for JSON format, detail struct otherwise.
///
/// Password / ReadOnlyPassword fields are redacted by default; the caller
/// passes `--reveal` to bypass.
fn print_storage_zone(sz: &StorageZone, format: OutputFormat, redact_cfg: &RedactConfig) {
    if let OutputFormat::Json = format {
        let mut value = serde_json::to_value(sz).expect("failed to serialize StorageZone to JSON");
        redact_secrets_in_json(&mut value, redact_cfg);
        let json = serde_json::to_string_pretty(&value).expect("failed to serialize to JSON");
        println!("{json}");
    } else {
        let detail = StorageZoneDetail::from(sz);
        output::print_single(&detail, format);
    }
}
