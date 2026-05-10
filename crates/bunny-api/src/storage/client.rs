use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result, anyhow, bail};
use bytes::Bytes;

use crate::recording::{capture_request, maybe_record_response};

use super::types::{StorageError, StorageObject};

/// URL-encode each `/`-separated segment of a path, preserving the slashes.
///
/// For example `"images/my dir"` becomes `"images/my%20dir"`.
fn encode_path_segments(path: &str) -> String {
    path.split('/')
        .map(|seg| urlencoding::encode(seg).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Known Bunny CDN storage region prefixes.
///
/// These are the only accepted values for the `region` parameter of
/// [`StorageClient::new`]. Each entry maps to a specific data center.
pub const VALID_REGIONS: &[&str] = &[
    "storage", // Falkenstein (default/primary)
    "uk",      // London
    "ny",      // New York
    "la",      // Los Angeles
    "sg",      // Singapore
    "syd",     // Sydney / Oceania
    "br",      // São Paulo
    "jh",      // Johannesburg / Africa
    "se",      // Stockholm
];

/// Client for the bunny.net Edge Storage API.
///
/// Each storage zone has a primary region, which determines the API hostname.
/// Construct the client with the region prefix (e.g. `"storage"`, `"la"`,
/// `"sg"`, `"syd"`) and the storage zone access key.
///
/// # Example
///
/// ```no_run
/// use bunny_api::storage::StorageClient;
///
/// #[tokio::main]
/// async fn main() {
///     let client = StorageClient::new("storage", "my-access-key").unwrap();
///     let files = client.list_files("my-zone", "images").await.unwrap();
/// }
/// ```
pub struct StorageClient {
    http: reqwest::Client,
    base_url: String,
    access_key: String,
    debug: bool,
    record_dir: Option<PathBuf>,
    last_request: Mutex<Option<(String, String)>>,
}

impl StorageClient {
    /// Creates a new `StorageClient`.
    ///
    /// `region` must be one of the known Bunny CDN storage region prefixes
    /// (see [`VALID_REGIONS`]). An unknown value is rejected with an error to
    /// prevent hostname injection via caller-controlled input.
    ///
    /// `access_key` is the storage zone password shown in the bunny.net dashboard.
    ///
    /// # Errors
    ///
    /// Returns an error if `region` is not a recognised Bunny CDN region.
    pub fn new(region: &str, access_key: impl Into<String>) -> Result<Self> {
        if !VALID_REGIONS.contains(&region) {
            bail!(
                "unknown storage region {:?}; valid regions are: {}",
                region,
                VALID_REGIONS.join(", ")
            );
        }
        let base_url = format!("https://{region}.bunnycdn.com");
        Ok(Self {
            http: reqwest::Client::new(),
            base_url,
            access_key: access_key.into(),
            debug: false,
            record_dir: None,
            last_request: Mutex::new(None),
        })
    }

    /// Creates a client pointing at a custom base URL (useful for tests /
    /// staging environments).
    pub fn with_base_url(access_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            access_key: access_key.into(),
            debug: false,
            record_dir: None,
            last_request: Mutex::new(None),
        }
    }

    /// Enable or disable debug logging of HTTP requests and responses to stderr.
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable recording API responses to files in the given directory.
    #[must_use]
    pub fn with_record(mut self, dir: impl Into<PathBuf>) -> Self {
        self.record_dir = Some(dir.into());
        self
    }

    /// Lists files and directories at the given path within a storage zone.
    ///
    /// `path` should not include a leading or trailing slash; it represents the
    /// directory hierarchy below the storage zone root (e.g. `"images/2024"`).
    /// Pass an empty string to list the root of the zone.
    pub async fn list_files(
        &self,
        storage_zone_name: &str,
        path: &str,
    ) -> Result<Vec<StorageObject>> {
        let url = self.listing_url(storage_zone_name, path);
        let rb = self.http.get(&url).header("AccessKey", &self.access_key);
        let response = self.send(rb).await?;
        let (status, bytes) = self.read_body(response).await?;

        if !status.is_success() {
            return Err(self.extract_error(status, &bytes));
        }

        serde_json::from_slice(&bytes).context("deserializing list response")
    }

    /// Downloads a single file and returns its raw bytes.
    pub async fn download_file(
        &self,
        storage_zone_name: &str,
        path: &str,
        file_name: &str,
    ) -> Result<Bytes> {
        let url = self.file_url(storage_zone_name, path, file_name);
        let rb = self.http.get(&url).header("AccessKey", &self.access_key);
        let response = self.send(rb).await?;
        let (status, bytes) = self.read_body(response).await?;

        if !status.is_success() {
            return Err(self.extract_error(status, &bytes));
        }

        Ok(bytes)
    }

    /// Uploads a file to the given path.
    ///
    /// `body` can be any type that converts into a [`reqwest::Body`], including
    /// `Vec<u8>`, `&'static [u8]`, `String`, `File` (via `Body::wrap_stream`),
    /// or a `tokio_util::codec::FramedRead` stream.
    ///
    /// `checksum` is an optional SHA-256 hex digest (uppercase) that the server
    /// will validate against the received content.
    pub async fn upload_file(
        &self,
        storage_zone_name: &str,
        path: &str,
        file_name: &str,
        body: impl Into<reqwest::Body>,
        checksum: Option<&str>,
    ) -> Result<()> {
        let url = self.file_url(storage_zone_name, path, file_name);
        let mut rb = self
            .http
            .put(&url)
            .header("AccessKey", &self.access_key)
            .body(body);

        if let Some(sha256) = checksum {
            rb = rb.header("Checksum", sha256);
        }

        let response = self.send(rb).await?;
        let (status, bytes) = self.read_body(response).await?;

        if !status.is_success() {
            return Err(self.extract_error(status, &bytes));
        }

        Ok(())
    }

    /// Deletes a file (or directory and all its contents) at the given path.
    pub async fn delete_file(
        &self,
        storage_zone_name: &str,
        path: &str,
        file_name: &str,
    ) -> Result<()> {
        let url = self.file_url(storage_zone_name, path, file_name);
        let rb = self.http.delete(&url).header("AccessKey", &self.access_key);
        let response = self.send(rb).await?;
        let (status, bytes) = self.read_body(response).await?;

        if !status.is_success() {
            return Err(self.extract_error(status, &bytes));
        }

        Ok(())
    }

    // --- URL helpers ---

    /// Builds the listing URL: `{base}/{zone}/{path}/`
    fn listing_url(&self, storage_zone_name: &str, path: &str) -> String {
        let zone = urlencoding::encode(storage_zone_name);
        if path.is_empty() {
            format!("{}/{zone}/", self.base_url)
        } else {
            let encoded_path = encode_path_segments(path);
            format!("{}/{zone}/{encoded_path}/", self.base_url)
        }
    }

    /// Builds the file URL: `{base}/{zone}/{path}/{file_name}`
    fn file_url(&self, storage_zone_name: &str, path: &str, file_name: &str) -> String {
        let zone = urlencoding::encode(storage_zone_name);
        let name = urlencoding::encode(file_name);
        if path.is_empty() {
            format!("{}/{zone}/{name}", self.base_url)
        } else {
            let encoded_path = encode_path_segments(path);
            format!("{}/{zone}/{encoded_path}/{name}", self.base_url)
        }
    }

    // --- Internal helpers ---

    /// Execute a prepared request, logging method and URL to stderr when debug
    /// mode is enabled. The `AccessKey` header value is never logged.
    async fn send(&self, rb: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let request = rb.build().context("failed to build request")?;
        if self.debug {
            eprintln!(">> {} {}", request.method(), request.url());
        }
        capture_request(
            &self.last_request,
            request.method().as_ref(),
            request.url().path(),
        );
        self.http
            .execute(request)
            .await
            .context("HTTP request failed")
    }

    /// Read the response body, logging status and body when debug is enabled.
    async fn read_body(&self, response: reqwest::Response) -> Result<(reqwest::StatusCode, Bytes)> {
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .context("failed to read response body")?;
        if self.debug {
            eprintln!("<< {status}");
            eprintln!("<<< {}", String::from_utf8_lossy(&bytes));
        }
        maybe_record_response(
            self.record_dir.as_deref(),
            &self.last_request,
            status.is_success(),
            &bytes,
        );
        Ok((status, bytes))
    }

    // --- Error extraction ---

    /// Attempts to parse a [`StorageError`] from the response body; falls back
    /// to a generic HTTP status error if the body is not valid JSON.
    fn extract_error(&self, status: reqwest::StatusCode, bytes: &Bytes) -> anyhow::Error {
        match serde_json::from_slice::<StorageError>(bytes) {
            Ok(api_err) => anyhow!(
                "Storage API error {}: {}",
                api_err.http_code,
                api_err.message
            ),
            Err(_) => anyhow!("Storage API returned HTTP {}", status),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> StorageClient {
        StorageClient::new("storage", "test-key").unwrap()
    }

    #[test]
    fn new_accepts_valid_regions() {
        for &region in VALID_REGIONS {
            assert!(
                StorageClient::new(region, "key").is_ok(),
                "expected region {region:?} to be accepted"
            );
        }
    }

    #[test]
    fn new_rejects_unknown_region() {
        let result = StorageClient::new("evil.com", "key");
        assert!(result.is_err(), "expected an error for unknown region");
        let msg = result.err().unwrap().to_string();
        assert!(
            msg.contains("evil.com"),
            "error message should contain the bad region: {msg}"
        );
        assert!(
            msg.contains("valid regions"),
            "error message should list valid regions: {msg}"
        );
    }

    #[test]
    fn new_rejects_empty_region() {
        assert!(StorageClient::new("", "key").is_err());
    }

    #[test]
    fn listing_url_empty_path() {
        let c = client();
        assert_eq!(
            c.listing_url("my-zone", ""),
            "https://storage.bunnycdn.com/my-zone/"
        );
    }

    #[test]
    fn listing_url_with_path() {
        let c = client();
        assert_eq!(
            c.listing_url("my-zone", "images/2024"),
            "https://storage.bunnycdn.com/my-zone/images/2024/"
        );
    }

    #[test]
    fn listing_url_encodes_special_chars_in_zone() {
        let c = client();
        assert_eq!(
            c.listing_url("my zone", ""),
            "https://storage.bunnycdn.com/my%20zone/"
        );
    }

    #[test]
    fn listing_url_encodes_special_chars_in_path() {
        let c = client();
        assert_eq!(
            c.listing_url("my-zone", "my folder/sub dir"),
            "https://storage.bunnycdn.com/my-zone/my%20folder/sub%20dir/"
        );
    }

    #[test]
    fn file_url_empty_path() {
        let c = client();
        assert_eq!(
            c.file_url("my-zone", "", "photo.jpg"),
            "https://storage.bunnycdn.com/my-zone/photo.jpg"
        );
    }

    #[test]
    fn file_url_with_path() {
        let c = client();
        assert_eq!(
            c.file_url("my-zone", "images/2024", "photo.jpg"),
            "https://storage.bunnycdn.com/my-zone/images/2024/photo.jpg"
        );
    }

    #[test]
    fn file_url_encodes_special_chars_in_file_name() {
        let c = client();
        assert_eq!(
            c.file_url("my-zone", "images", "my file #1.jpg"),
            "https://storage.bunnycdn.com/my-zone/images/my%20file%20%231.jpg"
        );
    }

    #[test]
    fn file_url_encodes_query_breaking_chars_in_file_name() {
        let c = client();
        // A `?` in the filename must be encoded so it is not treated as a query string.
        assert_eq!(
            c.file_url("my-zone", "", "file?name.txt"),
            "https://storage.bunnycdn.com/my-zone/file%3Fname.txt"
        );
    }
}
