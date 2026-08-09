use anyhow::{Result, bail};
use bunny_net_api::compute::ComputeClient;
use bunny_net_api::containers::ContainersClient;
use bunny_net_api::core::CoreClient;
use bunny_net_api::database::DatabaseClient;
use bunny_net_api::logging::LoggingClient;
use bunny_net_api::origin_errors::OriginErrorsClient;
use bunny_net_api::shield::ShieldClient;
use std::env;

/// Resolve the effective record directory.
///
/// Returns the explicit `--record <DIR>` flag value if set, else the
/// `HOPPY_RECORD_DIR` environment variable when non-empty, else `None`.
/// Lets the `live-api` E2E suite refresh fixtures without threading the
/// flag through every test command.
pub fn get_record_dir(explicit: Option<&str>) -> Option<String> {
    if let Some(dir) = explicit
        && !dir.is_empty()
    {
        return Some(dir.to_string());
    }
    match env::var("HOPPY_RECORD_DIR") {
        Ok(dir) if !dir.is_empty() => Some(dir),
        _ => None,
    }
}

/// Read the bunny.net API key from the BUNNY_API_KEY environment variable.
pub fn get_api_key() -> Result<String> {
    match env::var("BUNNY_API_KEY") {
        Ok(key) if !key.is_empty() => Ok(key),
        _ => bail!(
            "BUNNY_API_KEY environment variable is not set.\n\
             Set it with: export BUNNY_API_KEY=your-api-key"
        ),
    }
}

fn get_env_url(var: &str) -> Option<String> {
    match env::var(var) {
        Ok(url) if !url.is_empty() => Some(url),
        _ => None,
    }
}

/// Read a custom base URL for the bunny.net Core/Compute/Shield API.
pub fn get_api_url() -> Option<String> {
    get_env_url("BUNNY_API_URL")
}

/// Read a custom base URL for the bunny.net Containers API.
pub fn get_containers_url() -> Option<String> {
    get_env_url("BUNNY_CONTAINERS_URL")
}

/// Read a custom base URL for the bunny.net Stream API.
pub fn get_stream_url() -> Option<String> {
    get_env_url("BUNNY_STREAM_URL")
}

/// Read a custom base URL for the bunny.net Storage API.
pub fn get_storage_url() -> Option<String> {
    get_env_url("BUNNY_STORAGE_URL")
}

/// Read a custom base URL for the bunny.net Database API.
pub fn get_database_url() -> Option<String> {
    get_env_url("BUNNY_DATABASE_URL")
}

/// Read a custom base URL for the bunny.net CDN Logging API.
pub fn get_logging_url() -> Option<String> {
    get_env_url("BUNNY_LOGGING_URL")
}

/// Read a custom base URL for the bunny.net Origin Errors API.
pub fn get_origin_errors_url() -> Option<String> {
    get_env_url("BUNNY_ORIGIN_ERRORS_URL")
}

/// Build a `LoggingClient` with optional base URL override.
///
/// Read-only surface (no mutating endpoints) — no `debug_reveal_secrets` to wire.
pub fn logging_client(debug: bool, record: Option<&str>) -> Result<LoggingClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_logging_url() {
        LoggingClient::with_base_url(api_key, url)
    } else {
        LoggingClient::new(api_key)
    }
    .with_debug(debug);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build an `OriginErrorsClient` with optional base URL override.
///
/// Read-only surface (no mutating endpoints) — no `debug_reveal_secrets` to wire.
pub fn origin_errors_client(debug: bool, record: Option<&str>) -> Result<OriginErrorsClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_origin_errors_url() {
        OriginErrorsClient::with_base_url(api_key, url)
    } else {
        OriginErrorsClient::new(api_key)
    }
    .with_debug(debug);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `DatabaseClient` with optional base URL override and reveal-secrets flag.
pub fn database_client_with_reveal(
    debug: bool,
    record: Option<&str>,
    reveal_secrets: bool,
) -> Result<DatabaseClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_database_url() {
        DatabaseClient::new(api_key).with_base_url(url)
    } else {
        DatabaseClient::new(api_key)
    }
    .with_debug(debug)
    .with_debug_reveal_secrets(reveal_secrets);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Read the bunny.net Storage Zone access key from the BUNNY_STORAGE_KEY
/// environment variable.
///
/// Returns `Some(key)` if set and non-empty, `None` otherwise.
/// The caller is responsible for falling back to fetching the password
/// from the Core API if `None` is returned.
pub fn get_storage_key() -> Option<String> {
    match env::var("BUNNY_STORAGE_KEY") {
        Ok(key) if !key.is_empty() => Some(key),
        _ => None,
    }
}

/// Build a `CoreClient` with optional base URL override.
pub fn core_client(debug: bool, record: Option<&str>) -> Result<CoreClient> {
    core_client_with_reveal(debug, record, false)
}

/// Build a `CoreClient` with optional base URL override and reveal-secrets flag.
pub fn core_client_with_reveal(
    debug: bool,
    record: Option<&str>,
    reveal_secrets: bool,
) -> Result<CoreClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_api_url() {
        CoreClient::with_base_url(api_key, url)
    } else {
        CoreClient::new(api_key)
    }
    .with_debug(debug)
    .with_debug_reveal_secrets(reveal_secrets);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `ShieldClient` with optional base URL override and reveal-secrets flag.
pub fn shield_client_with_reveal(
    debug: bool,
    record: Option<&str>,
    reveal_secrets: bool,
) -> Result<ShieldClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_api_url() {
        ShieldClient::with_base_url(api_key, url)
    } else {
        ShieldClient::new(api_key)
    }
    .with_debug(debug)
    .with_debug_reveal_secrets(reveal_secrets);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `ComputeClient` with optional base URL override and reveal-secrets flag.
pub fn compute_client_with_reveal(
    debug: bool,
    record: Option<&str>,
    reveal_secrets: bool,
) -> Result<ComputeClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_api_url() {
        ComputeClient::with_base_url(api_key, url)
    } else {
        ComputeClient::new(api_key)
    }
    .with_debug(debug)
    .with_debug_reveal_secrets(reveal_secrets);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `ContainersClient` with optional base URL override and reveal-secrets flag.
pub fn containers_client_with_reveal(
    debug: bool,
    record: Option<&str>,
    reveal_secrets: bool,
) -> Result<ContainersClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_containers_url() {
        ContainersClient::with_base_url(api_key, url)
    } else {
        ContainersClient::new(api_key)
    }
    .with_debug(debug)
    .with_debug_reveal_secrets(reveal_secrets);
    if let Some(dir) = get_record_dir(record) {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Read the bunny.net Stream API key from the BUNNY_STREAM_KEY environment variable.
///
/// Returns `Some(key)` if set and non-empty, `None` otherwise.
/// The caller is responsible for falling back to fetching the key from the
/// VideoLibrary's `ApiKey` field via the Core API if `None` is returned.
pub fn get_stream_key() -> Option<String> {
    match env::var("BUNNY_STREAM_KEY") {
        Ok(key) if !key.is_empty() => Some(key),
        _ => None,
    }
}
