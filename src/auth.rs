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
