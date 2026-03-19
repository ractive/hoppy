use std::path::Path;
use std::sync::Mutex;

/// Capture a request's method and path for later recording.
///
/// Uses `unwrap_or_else` to recover from a poisoned mutex rather than panicking.
pub fn capture_request(last_request: &Mutex<Option<(String, String)>>, method: &str, path: &str) {
    *last_request.lock().unwrap_or_else(|e| e.into_inner()) =
        Some((method.to_string(), path.to_string()));
}

/// Record a successful JSON response body to a fixture file.
///
/// Skips recording when:
/// - `is_success` is false (non-2xx status)
/// - `record_dir` is `None` (recording disabled)
/// - No prior request was captured
/// - The body does not start with `{` or `[` (not JSON)
///
/// Write errors are reported to stderr (best-effort).
pub fn maybe_record_response(
    record_dir: Option<&Path>,
    last_request: &Mutex<Option<(String, String)>>,
    is_success: bool,
    bytes: &[u8],
) {
    if !is_success {
        return;
    }
    let Some(dir) = record_dir else {
        return;
    };
    let Some((method, path)) = last_request
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take()
    else {
        return;
    };
    if bytes.first().is_none_or(|b| *b != b'{' && *b != b'[') {
        return;
    }
    let sanitized = path.trim_matches('/').replace('/', "_");
    let filename = if sanitized.is_empty() {
        format!("{method}_root.json")
    } else {
        format!("{method}_{sanitized}.json")
    };
    let file_path = dir.join(&filename);
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes)
        && let Ok(pretty) = serde_json::to_string_pretty(&value)
    {
        if let Err(e) = std::fs::write(&file_path, format!("{pretty}\n")) {
            eprintln!("record: failed to write {}: {e}", file_path.display());
        }
        return;
    }
    if let Err(e) = std::fs::write(&file_path, bytes) {
        eprintln!("record: failed to write {}: {e}", file_path.display());
    }
}
