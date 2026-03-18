use crate::auth;
use crate::cli::{OutputFormat, StorageAction};
use crate::output;
use anyhow::{Context, Result, bail};
use bunny_api_core::CoreClient;
use bunny_api_storage::StorageClient;
use bunny_api_storage::StorageObject;
use std::io::{self, BufRead, Write};
use tokio::fs;

// ---------------------------------------------------------------------------
// Display structs
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, tabled::Tabled)]
struct StorageObjectRow {
    #[tabled(rename = "Name")]
    object_name: String,
    #[tabled(rename = "Size")]
    length: i64,
    #[tabled(rename = "Directory")]
    is_directory: bool,
    #[tabled(rename = "Last Changed")]
    last_changed: String,
}

impl From<&StorageObject> for StorageObjectRow {
    fn from(obj: &StorageObject) -> Self {
        Self {
            object_name: obj.object_name.clone(),
            length: obj.length,
            is_directory: obj.is_directory,
            last_changed: obj.last_changed.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &StorageAction,
    format: OutputFormat,
    debug: bool,
    yes: bool,
) -> Result<()> {
    match action {
        StorageAction::Ls { zone, path, region } => {
            let client = build_storage_client(zone, region, debug).await?;
            let path = path.trim_matches('/');
            let objects = client.list_files(zone, path).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&objects).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<StorageObjectRow> =
                    objects.iter().map(StorageObjectRow::from).collect();
                output::print_data(&rows, format);
            }
        }
        StorageAction::Upload {
            zone,
            remote_path,
            file,
            region,
        } => {
            let client = build_storage_client(zone, region, debug).await?;
            let (dir, name) = split_remote_path(remote_path)?;
            let bytes =
                fs::read(file).await.with_context(|| format!("reading local file: {file}"))?;
            eprintln!("Uploading {file} → {zone}/{remote_path} ...");
            client.upload_file(zone, dir, name, bytes, None).await?;
            eprintln!("Done.");
        }
        StorageAction::Download {
            zone,
            remote_path,
            output,
            region,
        } => {
            let client = build_storage_client(zone, region, debug).await?;
            let (dir, name) = split_remote_path(remote_path)?;
            eprintln!("Downloading {zone}/{remote_path} ...");
            let bytes = client.download_file(zone, dir, name).await?;
            match output {
                Some(path) => {
                    fs::write(path, &bytes)
                        .await
                        .with_context(|| format!("writing output file: {path}"))?;
                    eprintln!("Saved to {path} ({} bytes)", bytes.len());
                }
                None => {
                    io::stdout()
                        .write_all(&bytes)
                        .context("writing to stdout")?;
                }
            }
        }
        StorageAction::Rm {
            zone,
            remote_path,
            region,
        } => {
            if !yes {
                eprint!("Delete {zone}/{remote_path}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let client = build_storage_client(zone, region, debug).await?;
            let (dir, name) = split_remote_path(remote_path)?;
            client.delete_file(zone, dir, name).await?;
            eprintln!("Deleted {zone}/{remote_path}");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Split a remote path like `"images/2024/photo.jpg"` into
/// `("images/2024", "photo.jpg")`.
///
/// A bare filename with no directory component returns `("", "photo.jpg")`.
fn split_remote_path(remote_path: &str) -> Result<(&str, &str)> {
    let path = remote_path.trim_matches('/');
    if path.is_empty() {
        bail!("remote_path must not be empty");
    }
    match path.rfind('/') {
        Some(idx) => Ok((&path[..idx], &path[idx + 1..])),
        None => Ok(("", path)),
    }
}

/// Resolve the storage access key and build a `StorageClient`.
///
/// Resolution order:
/// 1. `BUNNY_STORAGE_KEY` environment variable.
/// 2. Fetch the storage zone via the Core API and use its `Password` field.
async fn build_storage_client(zone_name: &str, region: &str, debug: bool) -> Result<StorageClient> {
    let access_key = if let Some(key) = auth::get_storage_key() {
        key
    } else {
        // Fall back to fetching the password from the Core API.
        let api_key = auth::get_api_key().context(
            "BUNNY_STORAGE_KEY is not set and BUNNY_API_KEY is needed to fetch the storage key",
        )?;
        let core = CoreClient::new(api_key).with_debug(debug);
        let result = core
            .list_storage_zones(None, None, Some(zone_name), None)
            .await
            .context("fetching storage zones to resolve access key")?;
        let zone = result
            .items
            .into_iter()
            .find(|z| z.name == zone_name)
            .with_context(|| format!("storage zone '{zone_name}' not found"))?;
        if zone.password.is_empty() {
            bail!(
                "storage zone '{zone_name}' password is empty; \
                 set BUNNY_STORAGE_KEY instead"
            );
        }
        zone.password
    };

    Ok(StorageClient::new(region, access_key).with_debug(debug))
}
