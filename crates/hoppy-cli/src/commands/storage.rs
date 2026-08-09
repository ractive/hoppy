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
    reveal: bool,
) -> Result<()> {
    match action {
        StorageAction::Ls {
            zone,
            remote_path,
            region,
        } => {
            let client = build_storage_client(zone, region, debug, record, reveal).await?;
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
            checksum,
        } => {
            let client = build_storage_client(zone, region, debug, record, reveal).await?;
            let (dir, name) = split_remote_path(remote_path)?;

            // Resolve the optional integrity checksum. `--checksum <hex>` supplies
            // a pre-computed digest (uppercased before sending). Bare `--checksum`
            // (empty string via default_missing_value) computes the SHA-256 by
            // streaming over the file so we never buffer the whole payload.
            let checksum_hex: Option<String> = match checksum {
                None => None,
                Some(hex) if hex.is_empty() => {
                    let digest = sha256_file_hex(file)
                        .await
                        .with_context(|| format!("computing SHA-256 checksum for: {file}"))?;
                    Some(digest)
                }
                Some(hex) => Some(normalise_checksum(hex)?),
            };

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

            client
                .upload_file(zone, dir, name, body, checksum_hex.as_deref())
                .await?;

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
            let client = build_storage_client(zone, region, debug, record, reveal).await?;
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
            // A trailing slash means "delete this directory recursively". We must
            // detect it BEFORE trimming, because the directory-vs-file distinction
            // is encoded in that slash.
            let is_directory = remote_path.trim_start_matches('/').ends_with('/');
            let display_path = remote_path.trim_start_matches('/');
            if !yes {
                if is_directory {
                    eprint!(
                        "Recursively delete directory {zone}/{display_path} and ALL its contents? [y/N] "
                    );
                } else {
                    eprint!("Delete {zone}/{display_path}? [y/N] ");
                }
                io::stderr().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                let answer = line.trim().to_lowercase();
                if answer != "y" && answer != "yes" {
                    eprintln!("Aborted.");
                    return Ok(());
                }
            }
            let client = build_storage_client(zone, region, debug, record, reveal).await?;
            if is_directory {
                let dir = remote_path.trim_matches('/');
                if dir.is_empty() {
                    bail!("remote_path must name a directory (refusing to delete the zone root)");
                }
                client.delete_directory(zone, dir).await?;
                output::print_mutation_result(
                    format,
                    "delete",
                    "storage-directory",
                    serde_json::json!({ "Path": format!("{zone}/{display_path}") }),
                    &format!("Recursively deleted directory {zone}/{display_path}"),
                );
            } else {
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

/// Validate a user-supplied SHA-256 checksum and normalise it to uppercase hex.
///
/// The bunny.net Storage API requires the `Checksum` header value to be
/// uppercase hex. A SHA-256 digest is exactly 64 hex characters.
fn normalise_checksum(raw: &str) -> Result<String> {
    let trimmed = raw.trim();
    if trimmed.len() != 64 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!(
            "invalid --checksum: expected a 64-character hex SHA-256 digest, got {} characters",
            trimmed.len()
        );
    }
    Ok(trimmed.to_ascii_uppercase())
}

/// Compute the SHA-256 of a local file as uppercase hex, streaming over the
/// file in fixed-size chunks so the whole payload is never buffered in memory.
async fn sha256_file_hex(path: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    use tokio::io::AsyncReadExt;

    let mut file = fs::File::open(path)
        .await
        .with_context(|| format!("opening local file: {path}"))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buf)
            .await
            .with_context(|| format!("reading file for checksum: {path}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    let digest = hasher.finalize();
    // Uppercase hex as required by the Storage API `Checksum` header.
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut out, "{byte:02X}").expect("writing to String never fails");
    }
    Ok(out)
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
    reveal: bool,
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
    client = client.with_debug(debug).with_debug_reveal_secrets(reveal);
    if let Some(dir) = auth::get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_checksum_uppercases_valid_digest() {
        let lower = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        let out = normalise_checksum(lower).unwrap();
        assert_eq!(out, lower.to_ascii_uppercase());
        assert_eq!(out.len(), 64);
    }

    #[test]
    fn normalise_checksum_trims_whitespace() {
        let padded = "  9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08  ";
        assert!(normalise_checksum(padded).is_ok());
    }

    #[test]
    fn normalise_checksum_rejects_wrong_length() {
        assert!(normalise_checksum("abc123").is_err());
    }

    #[test]
    fn normalise_checksum_rejects_non_hex() {
        // 64 chars but contains 'z'.
        let bad = "z".repeat(64);
        assert!(normalise_checksum(&bad).is_err());
    }

    #[tokio::test]
    async fn sha256_file_hex_matches_known_vector() {
        // SHA-256 of the ASCII string "abc".
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("abc.txt");
        std::fs::write(&path, b"abc").unwrap();
        let digest = sha256_file_hex(path.to_str().unwrap()).await.unwrap();
        assert_eq!(
            digest,
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        );
    }
}
