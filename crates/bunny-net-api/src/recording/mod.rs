pub mod debug;
pub mod redact;

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
    // Redaction is enabled by default. Set HOPPY_NO_REDACT=1 to skip it.
    // The env-var indirection avoids threading a `redact: bool` flag through
    // every domain client builder — recording is already an env-var-driven
    // opt-in, so this matches the existing pattern.
    let no_redact = std::env::var("HOPPY_NO_REDACT").as_deref() == Ok("1");

    let payload: Vec<u8> = if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(bytes)
    {
        if !no_redact {
            redact::redact_in_place(&mut value);
        }
        if let Ok(pretty) = serde_json::to_string_pretty(&value) {
            format!("{pretty}\n").into_bytes()
        } else {
            bytes.to_vec()
        }
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
        // Balance is a sensitive key — it will be redacted to 0.
        let body = br#"{"Balance":12.5,"Name":"my-zone"}"#;
        maybe_record_response(Some(dir.path()), "core", &last, true, body);
        let p = dir.path().join("core").join("GET_billing.json");
        assert!(p.exists(), "fixture not written under domain subdir");
        let txt = std::fs::read_to_string(&p).unwrap();
        // Balance key is present but value is redacted to 0; safe field preserved.
        assert!(txt.contains("\"Balance\""));
        assert!(txt.contains("\"Name\": \"my-zone\""));
        assert!(
            !txt.contains("12.5"),
            "raw balance must not appear in fixture"
        );
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

    /// Feed a synthetic billing-shaped body through `maybe_record_response` and
    /// assert that all sensitive fields are masked and safe structure is preserved.
    #[test]
    fn billing_response_is_redacted_on_record() {
        let dir = tempdir().unwrap();
        let body = br#"{
            "Balance": 42.50,
            "ThisMonthCharges": 77.77,
            "BillingRecords": [
                {
                    "Id": 1001,
                    "Amount": 5.55,
                    "PaymentId": "pm_test_abc123",
                    "InvoiceDownloadUrl": "https://billing.bunny.net/invoice/1?token=tok_secret&expires=9999",
                    "PayerEmail": "john.doe@example.com",
                    "Description": "CDN charges",
                    "Timestamp": "2026-06-01T00:00:00Z"
                }
            ],
            "ReceivingFunds": false
        }"#;
        let last = Mutex::new(Some(("GET".into(), "/billing".into())));
        maybe_record_response(Some(dir.path()), "core", &last, true, body);
        let txt =
            std::fs::read_to_string(dir.path().join("core").join("GET_billing.json")).unwrap();

        // Sensitive values must not appear.
        assert!(!txt.contains("42.50"), "real balance must not appear");
        assert!(
            !txt.contains("77.77"),
            "real monthly charges must not appear"
        );
        assert!(
            !txt.contains("pm_test_abc123"),
            "payment ID must not appear"
        );
        assert!(
            !txt.contains("john.doe@example.com"),
            "email must not appear"
        );
        assert!(
            !txt.contains("tok_secret"),
            "signed URL token must not appear"
        );

        // Safe fields and structure must be preserved.
        assert!(txt.contains("1001"), "record ID must be preserved");
        assert!(
            txt.contains("ReceivingFunds"),
            "bool field must be preserved"
        );
        assert!(txt.contains("false"), "bool value must be preserved");
        assert!(txt.contains("2026-06-01"), "timestamp must be preserved");
    }

    /// Snapshot test: run the known-good `billing_raw.json` fixture through
    /// redaction and compare against the checked-in `billing_redacted.json`.
    #[test]
    fn billing_snapshot_matches_expected_redaction() {
        let raw: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/billing_raw.json"))
                .expect("billing_raw.json must be valid JSON");
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/billing_redacted.json"))
                .expect("billing_redacted.json must be valid JSON");

        let mut actual = raw;
        redact::redact_in_place(&mut actual);

        assert_eq!(
            actual, expected,
            "redacted output does not match billing_redacted.json snapshot"
        );
    }
}
