use serde::{Deserialize, Serialize};

/// A file or directory object returned by the Storage API list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageObject {
    /// Unique identifier for the object.
    pub guid: String,
    /// The name of the storage zone the object belongs to.
    pub storage_zone_name: String,
    /// The full directory path where the object is located.
    pub path: String,
    /// The name of the file or directory.
    pub object_name: String,
    /// Size of the file in bytes. Zero for directories.
    pub length: i64,
    /// ISO 8601 datetime string of when the object was last modified.
    pub last_changed: String,
    /// `true` if this object is a directory, `false` if it is a file.
    pub is_directory: bool,
    /// ID of the physical server storing the file.
    pub server_id: i64,
    /// ID of the storage zone.
    pub storage_zone_id: i64,
    /// ISO 8601 datetime string of when the object was created.
    pub date_created: String,
    /// ID of the account owner.
    pub user_id: String,
}

/// Error response returned by the Storage API on non-2xx responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StorageError {
    /// HTTP status code of the failed request.
    pub http_code: u16,
    /// Human-readable description of the error.
    pub message: String,
}
