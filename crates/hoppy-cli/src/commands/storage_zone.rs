use crate::auth;
use crate::cli::{OutputFormat, StorageZoneAction};
use crate::date;
use crate::output::{self, PaginatedListJson};
use crate::redact::{RedactConfig, redact_secrets_in_json};
use anyhow::{Context, Result, bail};
use bunny_net_api::core::types::{CreateStorageZone, StorageZone, UpdateStorageZone};
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
    let client = auth::core_client_with_reveal(debug, record, redact_cfg.reveal_all)?;

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
            let created = client.create_storage_zone(&body).await?;
            // The create response returns a literal "string" placeholder for Password/
            // ReadOnlyPassword. Fetch the zone immediately to get the real credentials.
            let sz = client.get_storage_zone(created.id).await.with_context(|| {
                format!(
                    "storage zone {} was created but credential fetch failed — \
                     run `hoppy storage-zone get --id {}` to retrieve them",
                    created.id, created.id
                )
            })?;
            // On create, reveal the password so scripts can capture it — the user
            // explicitly requested this zone and needs the credential right now.
            let reveal_cfg = RedactConfig::new(true, vec![]);
            print_storage_zone(&sz, format, &reveal_cfg);
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

/// Output a single StorageZone as a vertical Field/Value table (or JSON).
///
/// Password / ReadOnlyPassword fields are redacted by default; the caller
/// passes `--reveal` to bypass.
fn print_storage_zone(sz: &StorageZone, format: OutputFormat, redact_cfg: &RedactConfig) {
    output::print_single_vertical(sz, format, redact_cfg);
}
