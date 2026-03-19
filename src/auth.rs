use anyhow::{Result, bail};
use bunny_api_compute::ComputeClient;
use bunny_api_containers::ContainersClient;
use bunny_api_core::CoreClient;
use bunny_api_shield::ShieldClient;
use std::env;

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
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_api_url() {
        CoreClient::with_base_url(api_key, url)
    } else {
        CoreClient::new(api_key)
    }
    .with_debug(debug);
    if let Some(dir) = record {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `ShieldClient` with optional base URL override.
pub fn shield_client(debug: bool, record: Option<&str>) -> Result<ShieldClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_api_url() {
        ShieldClient::with_base_url(api_key, url)
    } else {
        ShieldClient::new(api_key)
    }
    .with_debug(debug);
    if let Some(dir) = record {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `ComputeClient` with optional base URL override.
pub fn compute_client(debug: bool, record: Option<&str>) -> Result<ComputeClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_api_url() {
        ComputeClient::with_base_url(api_key, url)
    } else {
        ComputeClient::new(api_key)
    }
    .with_debug(debug);
    if let Some(dir) = record {
        client = client.with_record(dir);
    }
    Ok(client)
}

/// Build a `ContainersClient` with optional base URL override.
pub fn containers_client(debug: bool, record: Option<&str>) -> Result<ContainersClient> {
    let api_key = get_api_key()?;
    let mut client = if let Some(url) = get_containers_url() {
        ContainersClient::with_base_url(api_key, url)
    } else {
        ContainersClient::new(api_key)
    }
    .with_debug(debug);
    if let Some(dir) = record {
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
