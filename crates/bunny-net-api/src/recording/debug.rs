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

/// Walk a JSON value and redact string-valued fields whose names suggest
/// they hold secrets (token, password, secret, _key).
fn redact_debug_body(value: &mut serde_json::Value) {
    const KEY_SUFFIX_NOT_SECRET: &[&str] = &["zonesecuritykey", "userpkkey", "publickey"];
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower = k.to_lowercase();
                let is_allowlisted = KEY_SUFFIX_NOT_SECRET.iter().any(|s| lower.ends_with(s));
                let is_secret = !is_allowlisted
                    && (lower.ends_with("password")
                        || lower.ends_with("_password")
                        || lower.ends_with("secret")
                        || lower.ends_with("_secret")
                        || lower.ends_with("token")
                        || lower.ends_with("_token")
                        || lower.ends_with("apikey")
                        || lower.ends_with("api_key")
                        || lower.ends_with("_key")
                        || lower.contains("credential"));
                if is_secret {
                    if let serde_json::Value::String(raw) = v {
                        let len = raw.chars().count();
                        if len == 0 {
                            *v = serde_json::Value::String("<unset>".to_owned());
                        } else {
                            *v = serde_json::Value::String(format!("<set, length={len}>"));
                        }
                    } else if v.is_null() {
                        *v = serde_json::Value::String("<unset>".to_owned());
                    } else {
                        redact_debug_body(v);
                    }
                } else {
                    redact_debug_body(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                redact_debug_body(v);
            }
        }
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
