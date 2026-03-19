use crate::auth;
use crate::cli::{OutputFormat, StorageZoneAction};
use crate::output::{self, PaginatedListJson};
use anyhow::{Result, bail};
use bunny_api_core::types::{CreateStorageZone, StorageZone, UpdateStorageZone};
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
                let json =
                    serde_json::to_string_pretty(&envelope).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<StorageZoneRow> =
                    result.items.iter().map(StorageZoneRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        StorageZoneAction::Get { id } => {
            let sz = client.get_storage_zone(*id).await?;
            print_storage_zone(&sz, format);
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
                body = body.zone_tier(*tier);
            }
            let sz = client.create_storage_zone(&body).await?;
            print_storage_zone(&sz, format);
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
    }

    Ok(())
}

/// Output a single StorageZone: full JSON for JSON format, detail struct otherwise.
fn print_storage_zone(sz: &StorageZone, format: OutputFormat) {
    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(sz).expect("failed to serialize to JSON");
        println!("{json}");
    } else {
        let detail = StorageZoneDetail::from(sz);
        output::print_single(&detail, format);
    }
}
