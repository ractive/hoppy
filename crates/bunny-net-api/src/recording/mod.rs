use std::path::Path;
use std::sync::Mutex;

/// Capture a request's method and path for later recording.
///
/// Uses `unwrap_or_else` to recover from a poisoned mutex rather than panicking.
pub fn capture_request(last_request: &Mutex<Option<(String, String)>>, method: &str, path: &str) {
    *last_request.lock().unwrap_or_else(|e| e.into_inner()) =
        Some((method.to_string(), path.to_string()));
}

/// Record a successful JSON response body to a per-domain fixture file.
///
/// The output file lands at `record_dir/<domain>/<method>_<path>.json` so a
/// single `--record fixtures/` (or `HOPPY_RECORD_DIR=fixtures/`) refreshes
/// the on-disk layout in place.
///
/// Skips recording when:
/// - `is_success` is false (non-2xx status)
/// - `record_dir` is `None` (recording disabled)
/// - No prior request was captured
/// - The body does not start with `{` or `[` (not JSON)
///
/// Writes are idempotent: if the target file already exists with identical
/// bytes, no write occurs. On any write (creation or content change), a
/// single `record: updated <domain>/<file>` line is printed to stderr.
/// Write errors are reported to stderr (best-effort).
pub fn maybe_record_response(
    record_dir: Option<&Path>,
    domain: &str,
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
    let file_path = dir.join(domain).join(&filename);
    if let Some(parent) = file_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let payload: Vec<u8> = if let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes)
        && let Ok(pretty) = serde_json::to_string_pretty(&value)
    {
        format!("{pretty}\n").into_bytes()
    } else {
        bytes.to_vec()
    };

    if let Ok(existing) = std::fs::read(&file_path)
        && existing == payload
    {
        return;
    }

    match std::fs::write(&file_path, &payload) {
        Ok(()) => eprintln!("record: updated {domain}/{filename}"),
        Err(e) => eprintln!("record: failed to write {}: {e}", file_path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[test]
    fn records_under_domain_subdir() {
        let dir = tempdir().unwrap();
        let last = Mutex::new(Some(("GET".into(), "/billing".into())));
        let body = br#"{"Balance":12.5}"#;
        maybe_record_response(Some(dir.path()), "core", &last, true, body);
        let p = dir.path().join("core").join("GET_billing.json");
        assert!(p.exists(), "fixture not written under domain subdir");
        let txt = std::fs::read_to_string(&p).unwrap();
        assert!(txt.starts_with('{') && txt.contains("\"Balance\""));
    }

    #[test]
    fn skips_overwrite_when_unchanged() {
        let dir = tempdir().unwrap();
        let body = br#"{"a":1}"#;
        let last = Mutex::new(Some(("GET".into(), "/x".into())));
        maybe_record_response(Some(dir.path()), "core", &last, true, body);
        let p = dir.path().join("core").join("GET_x.json");
        let mtime1 = std::fs::metadata(&p).unwrap().modified().unwrap();
        // Sleep briefly so a re-write would produce a different mtime.
        std::thread::sleep(std::time::Duration::from_millis(20));
        let last2 = Mutex::new(Some(("GET".into(), "/x".into())));
        maybe_record_response(Some(dir.path()), "core", &last2, true, body);
        let mtime2 = std::fs::metadata(&p).unwrap().modified().unwrap();
        assert_eq!(mtime1, mtime2, "file was rewritten despite identical body");
    }

    #[test]
    fn overwrites_when_changed() {
        let dir = tempdir().unwrap();
        let last = Mutex::new(Some(("GET".into(), "/x".into())));
        maybe_record_response(Some(dir.path()), "core", &last, true, br#"{"a":1}"#);
        let last2 = Mutex::new(Some(("GET".into(), "/x".into())));
        maybe_record_response(Some(dir.path()), "core", &last2, true, br#"{"a":2}"#);
        let txt = std::fs::read_to_string(dir.path().join("core").join("GET_x.json")).unwrap();
        assert!(txt.contains("\"a\": 2"));
    }
}
