/// Field-name patterns (case-insensitive substring match) that indicate
/// the value should be redacted regardless of its type.
const SENSITIVE_KEY_PATTERNS: &[&str] = &[
    "email",
    "payer",
    "payment",
    "balance",
    "charges",
    "recharge",
    "invoice",
    "downloadurl",
    "apikey",
    "accesskey",
    "signingkey",
    "signingsecret",
    "secret",
    "token",
    "password",
];

/// Returns `true` when the object key suggests a sensitive value.
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    SENSITIVE_KEY_PATTERNS.iter().any(|pat| lower.contains(pat))
}

/// Returns `true` when the string value itself looks sensitive:
/// - A URL carrying `?token=`, `&token=`, `signature=`, or `expires=`
/// - A JWT (three base64url-ish segments separated by `.`)
pub fn is_sensitive_value(value: &str) -> bool {
    is_signed_url(value) || is_jwt(value)
}

fn is_signed_url(s: &str) -> bool {
    s.contains("?token=")
        || s.contains("&token=")
        || s.contains("signature=")
        || s.contains("expires=")
}

fn is_jwt(s: &str) -> bool {
    // A JWT has exactly two dots splitting three non-empty, base64url-ish segments.
    let parts: Vec<&str> = s.splitn(4, '.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|part| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'=')
    })
}

/// Recursively redact sensitive fields and values in a `serde_json::Value`.
///
/// Redaction rules:
/// - If an object key matches `is_sensitive_key`, its value is fully redacted:
///   strings → `"<redacted>"`, numbers → `0`, nested objects/arrays → every
///   leaf is redacted in turn.
/// - If a string value (in any position) matches `is_sensitive_value`, it is
///   replaced with `"<redacted>"`.
/// - Booleans, nulls, and array/object structure are preserved.
pub fn redact_in_place(value: &mut serde_json::Value) {
    redact_value(value, false);
}

fn redact_value(value: &mut serde_json::Value, force: bool) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                let sensitive = force || is_sensitive_key(key);
                redact_value(val, sensitive);
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                redact_value(item, force);
            }
        }
        serde_json::Value::String(s) if force || is_sensitive_value(s) => {
            *value = serde_json::Value::String("<redacted>".to_owned());
        }
        serde_json::Value::Number(_) if force => {
            *value = serde_json::Value::Number(0.into());
        }
        // Bool and Null are never redacted.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn redacted(mut v: serde_json::Value) -> serde_json::Value {
        redact_in_place(&mut v);
        v
    }

    #[test]
    fn sensitive_keys_are_redacted() {
        let cases = [
            ("Email", json!("user@example.com")),
            ("PayerEmail", json!("payer@x.com")),
            ("PaymentId", json!("pm_abc")),
            ("Balance", json!(99.99)),
            ("ThirtyDayCharges", json!(12.5)),
            ("RechargeAmount", json!(50.0)),
            ("InvoiceId", json!(42)),
            ("DownloadUrl", json!("https://example.com/file")),
            ("ApiKey", json!("secret-key")),
            ("AccessKey", json!("access-secret")),
            ("Token", json!("tok_abc")),
            ("Password", json!("hunter2")),
            ("AutomaticPaymentIdentifier", json!("auto_pm_xyz")),
            ("AWSSigningKey", json!("AKIA...")),
            ("AWSSigningSecret", json!("secret")),
        ];
        for (key, val) in cases {
            let result = redacted(json!({ key: val }));
            let redacted_val = &result[key];
            assert!(
                redacted_val == "<redacted>" || redacted_val == 0,
                "key={key}: expected redaction, got {redacted_val}"
            );
        }
    }

    #[test]
    fn case_insensitive_key_match() {
        let result = redacted(json!({ "BALANCE": 100.0, "email": "x@y.com" }));
        assert_eq!(result["BALANCE"], 0);
        assert_eq!(result["email"], "<redacted>");
    }

    #[test]
    fn signed_url_with_token_param() {
        let result = redacted(json!({ "url": "https://cdn.example.com/file?token=abc123" }));
        assert_eq!(result["url"], "<redacted>");
    }

    #[test]
    fn signed_url_with_amp_token_param() {
        let result = redacted(json!({ "url": "https://cdn.example.com/file?foo=1&token=abc" }));
        assert_eq!(result["url"], "<redacted>");
    }

    #[test]
    fn signed_url_with_signature() {
        let result =
            redacted(json!({ "link": "https://s3.example.com/x?signature=abc&expires=9999" }));
        assert_eq!(result["link"], "<redacted>");
    }

    #[test]
    fn signed_url_with_expires() {
        let result = redacted(json!({ "src": "https://cdn.example.com/f?expires=9999999" }));
        assert_eq!(result["src"], "<redacted>");
    }

    #[test]
    fn jwt_shaped_string_is_redacted() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let result = redacted(json!({ "auth": jwt }));
        assert_eq!(result["auth"], "<redacted>");
    }

    #[test]
    fn plain_url_not_redacted() {
        let result = redacted(json!({ "website": "https://example.com/page" }));
        assert_eq!(result["website"], "https://example.com/page");
    }

    #[test]
    fn safe_fields_preserved() {
        let result = redacted(json!({ "Id": 1, "Name": "My Zone", "Active": true }));
        assert_eq!(result["Id"], 1);
        assert_eq!(result["Name"], "My Zone");
        assert_eq!(result["Active"], true);
    }

    #[test]
    fn nested_object_redacted() {
        let result = redacted(json!({
            "Zone": {
                "Id": 42,
                "Email": "admin@example.com",
                "Balance": 100.0
            }
        }));
        assert_eq!(result["Zone"]["Id"], 42);
        assert_eq!(result["Zone"]["Email"], "<redacted>");
        assert_eq!(result["Zone"]["Balance"], 0);
    }

    #[test]
    fn array_of_objects_redacted() {
        let result = redacted(json!([
            { "Id": 1, "Email": "a@b.com", "Balance": 5.0 },
            { "Id": 2, "Email": "c@d.com", "Balance": 10.0 }
        ]));
        assert_eq!(result[0]["Id"], 1);
        assert_eq!(result[0]["Email"], "<redacted>");
        assert_eq!(result[0]["Balance"], 0);
        assert_eq!(result[1]["Id"], 2);
        assert_eq!(result[1]["Email"], "<redacted>");
    }

    #[test]
    fn sensitive_key_with_nested_object_value() {
        // When the key itself is sensitive, every leaf in a nested object is redacted.
        let result = redacted(json!({
            "InvoiceDetails": {
                "Number": 999,
                "Url": "https://billing.example.com/inv/1"
            }
        }));
        assert_eq!(result["InvoiceDetails"]["Number"], 0);
        assert_eq!(result["InvoiceDetails"]["Url"], "<redacted>");
    }

    #[test]
    fn sensitive_key_with_array_value() {
        // "InvoiceItems" contains "invoice" — every leaf inside is force-redacted.
        let result = redacted(json!({
            "InvoiceItems": [
                { "Amount": 9.99, "Description": "CDN" }
            ]
        }));
        assert_eq!(result["InvoiceItems"][0]["Amount"], 0);
        assert_eq!(result["InvoiceItems"][0]["Description"], "<redacted>");
    }

    #[test]
    fn bool_and_null_not_redacted() {
        let result = redacted(json!({ "Active": true, "Deleted": null }));
        assert_eq!(result["Active"], true);
        assert_eq!(result["Deleted"], serde_json::Value::Null);
    }
}
