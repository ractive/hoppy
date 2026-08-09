//! Shared `--debug` body formatting for every domain client.
//!
//! Every client's `send`/`execute` helper prints `>> METHOD URL` before
//! issuing a request; when the method is mutating (POST/PUT/PATCH/DELETE)
//! it also prints the request body via [`print_debug_request_body`], and
//! the response body (via [`format_debug_body`] directly) after reading it.
//! Both paths redact secret-shaped fields unless the caller opts in with
//! `reveal`.

const DEBUG_BODY_TRUNCATE: usize = 4096;

/// Returns `true` for methods whose request typically carries a body worth
/// inspecting in debug output.
fn is_mutating(method: &reqwest::Method) -> bool {
    *method == reqwest::Method::POST
        || *method == reqwest::Method::PUT
        || *method == reqwest::Method::PATCH
        || *method == reqwest::Method::DELETE
}

/// Print a built request's body to stderr for `--debug` output.
///
/// No-op for non-mutating methods. For mutating methods: prints the
/// (possibly redacted) body when it is buffered in memory, or
/// `<streaming body>` when it is a chunked/streamed body whose bytes
/// aren't available up front (e.g. file uploads).
pub fn print_debug_request_body(request: &reqwest::Request, reveal: bool) {
    if !is_mutating(request.method()) {
        return;
    }
    if let Some(body_bytes) = request.body().and_then(|b| b.as_bytes()) {
        eprintln!(">>> {}", format_debug_body(body_bytes, reveal));
    } else if request.body().is_some() {
        eprintln!(">>> <streaming body>");
    }
}

/// Format a request/response body for debug output.
///
/// - If the bytes are valid JSON, pretty-prints them (with optional redaction).
/// - Otherwise returns a UTF-8 lossy representation truncated at 4 KiB.
pub fn format_debug_body(bytes: &[u8], reveal: bool) -> String {
    if bytes.is_empty() {
        return "<empty>".to_owned();
    }
    if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(bytes) {
        if !reveal {
            redact_debug_body(&mut value);
        }
        return serde_json::to_string_pretty(&value).unwrap_or_else(|_| "<json error>".to_owned());
    }
    let text = String::from_utf8_lossy(bytes);
    if bytes.len() > DEBUG_BODY_TRUNCATE {
        // Truncate on a UTF-8 char boundary (raw byte slice would panic on
        // multi-byte chars or lossy replacement chars straddling the cut).
        let mut end = DEBUG_BODY_TRUNCATE.min(text.len());
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}… ({} bytes total)", &text[..end], bytes.len())
    } else {
        text.into_owned()
    }
}

/// Walk a JSON value and redact secret-shaped fields, delegating the
/// what-is-sensitive decision to [`super::redact::is_sensitive_key`] /
/// [`super::redact::is_sensitive_value`] — the same rules that protect
/// `--record` fixtures — so debug output can never be weaker than the
/// fixture redactor. Unlike the fixture redactor, string secrets keep
/// their length (`<set, length=N>`) because "did I send a token at all,
/// and roughly the right one" is exactly what --debug is for.
///
/// A key matching the sensitive rules force-redacts every string and
/// number leaf beneath it (arrays of tokens included); a string value
/// that itself looks sensitive (JWT, signed URL, 72-char account key)
/// is redacted wherever it appears.
fn redact_debug_body(value: &mut serde_json::Value) {
    redact_debug_value(value, false);
}

fn redact_debug_value(value: &mut serde_json::Value, force: bool) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let sensitive = force || super::redact::is_sensitive_key(k);
                if sensitive && v.is_null() {
                    *v = serde_json::Value::String("<unset>".to_owned());
                } else {
                    redact_debug_value(v, sensitive);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                redact_debug_value(v, force);
            }
        }
        serde_json::Value::String(s) if force || super::redact::is_sensitive_value(s) => {
            let len = s.chars().count();
            *value = if len == 0 {
                serde_json::Value::String("<unset>".to_owned())
            } else {
                serde_json::Value::String(format!("<set, length={len}>"))
            };
        }
        serde_json::Value::Number(_) if force => {
            *value = serde_json::Value::Number(0.into());
        }
        // Bool and Null (outside sensitive keys) are never redacted.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_debug_body_redacts_token_by_default() {
        let body = serde_json::json!({"LogForwardingToken": "abc"});
        let bytes = serde_json::to_vec(&body).unwrap();
        let out = format_debug_body(&bytes, false);
        assert!(
            out.contains("<set, length=3>"),
            "expected redacted token, got: {out}"
        );
        assert!(!out.contains("abc"), "expected token to be redacted");
    }

    #[test]
    fn format_debug_body_reveals_token_when_reveal_true() {
        let body = serde_json::json!({"LogForwardingToken": "abc"});
        let bytes = serde_json::to_vec(&body).unwrap();
        let out = format_debug_body(&bytes, true);
        assert!(out.contains("abc"), "expected token revealed, got: {out}");
    }

    #[test]
    fn format_debug_body_redacts_pascal_case_key_fields() {
        // Regression: the original suffix heuristic (`ends_with("_key")`)
        // missed every PascalCase `…Key` name, printing account API keys,
        // AccessKey and DeploymentKey values in cleartext.
        let account_key = format!(
            "{}{}",
            "12345678-1234-1234-1234-123456789abc", "12345678-1234-1234-1234-123456789abc"
        );
        let body = serde_json::json!({
            "Key": account_key,
            "AccessKey": "s3cr3t",
            "DeploymentKey": "dk-value",
            "PublicKey": "keep-me-readable",
        });
        let out = format_debug_body(&serde_json::to_vec(&body).unwrap(), false);
        assert!(!out.contains(&account_key), "account key leaked: {out}");
        assert!(!out.contains("s3cr3t"), "AccessKey leaked: {out}");
        assert!(!out.contains("dk-value"), "DeploymentKey leaked: {out}");
        assert!(
            out.contains("keep-me-readable"),
            "PublicKey wrongly redacted: {out}"
        );
    }

    #[test]
    fn format_debug_body_redacts_arrays_under_secret_keys() {
        let body = serde_json::json!({"Tokens": ["tok-one", "tok-two"]});
        let out = format_debug_body(&serde_json::to_vec(&body).unwrap(), false);
        assert!(!out.contains("tok-one"), "array token leaked: {out}");
        assert!(!out.contains("tok-two"), "array token leaked: {out}");
    }

    #[test]
    fn format_debug_body_redacts_sensitive_values_anywhere() {
        // JWTs and signed URLs are secrets regardless of the field name.
        let body = serde_json::json!({
            "Playback": "https://cdn.example/video.m3u8?token=abc123",
            "Session": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.c2ln",
        });
        let out = format_debug_body(&serde_json::to_vec(&body).unwrap(), false);
        assert!(!out.contains("token=abc123"), "signed URL leaked: {out}");
        assert!(!out.contains("eyJhbGciOiJIUzI1NiJ9"), "JWT leaked: {out}");
    }

    #[test]
    fn format_debug_body_empty() {
        assert_eq!(format_debug_body(&[], false), "<empty>");
    }

    #[test]
    fn format_debug_body_non_json_truncation() {
        let long = "x".repeat(5000);
        let bytes = long.as_bytes();
        let out = format_debug_body(bytes, false);
        assert!(
            out.contains("5000 bytes total"),
            "expected truncation note, got: {out}"
        );
    }
}
