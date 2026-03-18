use anyhow::{Context, Result, anyhow};
use bytes::Bytes;

use crate::types::{StorageError, StorageObject};

/// URL-encode each `/`-separated segment of a path, preserving the slashes.
///
/// For example `"images/my dir"` becomes `"images/my%20dir"`.
fn encode_path_segments(path: &str) -> String {
    path.split('/')
        .map(|seg| urlencoding::encode(seg).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Client for the bunny.net Edge Storage API.
///
/// Each storage zone has a primary region, which determines the API hostname.
/// Construct the client with the region prefix (e.g. `"storage"`, `"la"`,
/// `"sg"`, `"syd"`) and the storage zone access key.
///
/// # Example
///
/// ```no_run
/// use bunny_api_storage::StorageClient;
///
/// #[tokio::main]
/// async fn main() {
///     let client = StorageClient::new("storage", "my-access-key");
///     let files = client.list_files("my-zone", "images").await.unwrap();
/// }
/// ```
pub struct StorageClient {
    http: reqwest::Client,
    base_url: String,
    access_key: String,
    debug: bool,
}

impl StorageClient {
    /// Creates a new `StorageClient`.
    ///
    /// `region` is the subdomain prefix for the storage endpoint:
    /// - `"storage"` — Falkenstein (default)
    /// - `"la"` — New York / Los Angeles
    /// - `"sg"` — Singapore
    /// - `"syd"` — Sydney
    ///
    /// `access_key` is the storage zone password shown in the bunny.net dashboard.
    pub fn new(region: &str, access_key: impl Into<String>) -> Self {
        let base_url = format!("https://{region}.bunnycdn.com");
        Self {
            http: reqwest::Client::new(),
            base_url,
            access_key: access_key.into(),
            debug: false,
        }
    }

    /// Creates a client pointing at a custom base URL (useful for tests /
    /// staging environments).
    pub fn with_base_url(access_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            access_key: access_key.into(),
            debug: false,
        }
    }

    /// Enable or disable debug logging of HTTP requests and responses to stderr.
    #[must_use]
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
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

        if !response.status().is_success() {
            return Err(self.extract_error(response).await);
        }

        response
            .json::<Vec<StorageObject>>()
            .await
            .context("deserializing list response")
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

        if !response.status().is_success() {
            return Err(self.extract_error(response).await);
        }

        response.bytes().await.context("reading response body")
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

        if !response.status().is_success() {
            return Err(self.extract_error(response).await);
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

        if !response.status().is_success() {
            return Err(self.extract_error(response).await);
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
        let response = self
            .http
            .execute(request)
            .await
            .context("HTTP request failed")?;
        if self.debug {
            eprintln!("<< {}", response.status());
        }
        Ok(response)
    }

    // --- Error extraction ---

    /// Attempts to parse a [`StorageError`] from the response body; falls back
    /// to a generic HTTP status error if the body is not valid JSON.
    async fn extract_error(&self, response: reqwest::Response) -> anyhow::Error {
        let status = response.status();
        match response.json::<StorageError>().await {
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
        StorageClient::new("storage", "test-key")
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
