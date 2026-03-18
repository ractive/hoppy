use anyhow::{Result, bail};
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
