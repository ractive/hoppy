use crate::auth;
use crate::cli::{OutputFormat, StorageAction};
use crate::output;
use crate::progress;
use anyhow::{Context, Result, bail};
use bunny_net_api::storage::StorageClient;
use bunny_net_api::storage::StorageObject;
use std::io::{self, BufRead, Write};
use tokio::fs;
use tokio_util::io::ReaderStream;

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
    quiet: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        StorageAction::Ls {
            zone,
            remote_path,
            region,
        } => {
            let client = build_storage_client(zone, region, debug, record).await?;
            let path = remote_path.trim_matches('/');
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
            let client = build_storage_client(zone, region, debug, record).await?;
            let (dir, name) = split_remote_path(remote_path)?;

            // Open the file and get its size for the progress bar.
            let fh = fs::File::open(file)
                .await
                .with_context(|| format!("opening local file: {file}"))?;
            let file_size = fh
                .metadata()
                .await
                .with_context(|| format!("reading metadata for: {file}"))?
                .len();

            let pb = progress::file_progress(file_size, quiet);

            // Wrap the file in a streaming body so the progress bar can track
            // bytes as they are sent, without loading the whole file into memory.
            // `wrap_async_read` increments the bar as bytes pass through.
            // When quiet/no-TTY there is no bar, so we stream the file directly.
            let body: reqwest::Body = if let Some(bar) = &pb {
                reqwest::Body::wrap_stream(ReaderStream::new(bar.wrap_async_read(fh)))
            } else {
                reqwest::Body::wrap_stream(ReaderStream::new(fh))
            };

            client.upload_file(zone, dir, name, body, None).await?;

            progress::finish_with_message(pb.as_ref(), format!("Uploaded {file}"));

            let display_path = remote_path.trim_start_matches('/');
            if quiet {
                // nothing
            } else if pb.is_none() {
                // Not a TTY — emit result (JSON envelope or prose to stderr).
                output::print_mutation_result(
                    format,
                    "upload",
                    "storage-object",
                    serde_json::json!({ "Path": format!("{zone}/{display_path}") }),
                    &format!("Uploaded {file} → {zone}/{display_path}"),
                );
            }
        }
        StorageAction::Download {
            zone,
            remote_path,
            file,
            region,
        } => {
            let client = build_storage_client(zone, region, debug, record).await?;
            let (dir, name) = split_remote_path(remote_path)?;
            let display_path = remote_path.trim_start_matches('/');

            let pb = progress::spinner(format!("Downloading {zone}/{display_path}..."), quiet);

            // Stream the body straight to the sink instead of buffering the
            // whole file in memory.
            match file {
                Some(path) => {
                    let mut out = std::fs::File::create(path)
                        .with_context(|| format!("creating output file: {path}"))?;
                    let written = client
                        .download_file_streaming(zone, dir, name, &mut out)
                        .await?;
                    progress::finish_with_message(
                        pb.as_ref(),
                        format!("Downloaded {zone}/{display_path} ({written} bytes)"),
                    );
                    if pb.is_none() && !quiet {
                        eprintln!("Saved to {path} ({written} bytes)");
                    }
                }
                None => {
                    let stdout = io::stdout();
                    let mut handle = stdout.lock();
                    let written = client
                        .download_file_streaming(zone, dir, name, &mut handle)
                        .await?;
                    progress::finish_with_message(
                        pb.as_ref(),
                        format!("Downloaded {zone}/{display_path} ({written} bytes)"),
                    );
                }
            }
        }
        StorageAction::Rm {
            zone,
            remote_path,
            region,
        } => {
            let display_path = remote_path.trim_start_matches('/');
            if !yes {
                eprint!("Delete {zone}/{display_path}? [y/N] ");
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let client = build_storage_client(zone, region, debug, record).await?;
            let (dir, name) = split_remote_path(remote_path)?;
            client.delete_file(zone, dir, name).await?;
            output::print_mutation_result(
                format,
                "delete",
                "storage-object",
                serde_json::json!({ "Path": format!("{zone}/{display_path}") }),
                &format!("Deleted {zone}/{display_path}"),
            );
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
async fn build_storage_client(
    zone_name: &str,
    region: &str,
    debug: bool,
    record: Option<&str>,
) -> Result<StorageClient> {
    let access_key = if let Some(key) = auth::get_storage_key() {
        key
    } else {
        // Fall back to fetching the password from the Core API.
        let core = auth::core_client(debug, record).context(
            "BUNNY_STORAGE_KEY is not set and BUNNY_API_KEY is needed to fetch the storage key",
        )?;
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

    let mut client = if let Some(url) = auth::get_storage_url() {
        StorageClient::with_base_url(access_key, url)
    } else {
        StorageClient::new(region, access_key)?
    };
    client = client.with_debug(debug);
    if let Some(dir) = auth::get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}
