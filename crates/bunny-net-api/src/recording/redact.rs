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
    "deploymentkey",
    "amount",
    // Person-name fields: `Author` on compute releases carries the account
    // holder's real display name. Deliberately not plain "name" — that would
    // nuke resource names (`Name`, `Hostname`) that fixtures need. "author"
    // is handled separately in `is_sensitive_key` so that authorization-family
    // fields (`authorizationConfiguration`, `validateAuthorization`) stay
    // readable.
    "firstname",
    "lastname",
    "fullname",
];

/// Returns `true` when the object key suggests a sensitive value.
///
/// A bare `Key` (exact match, case-insensitive) is treated as sensitive too:
/// `GET /apikey` returns the account API key under exactly that name. The
/// substring patterns deliberately exclude plain "key" so identifiers like
/// `KeyId` or `errorKey` stay readable.
pub fn is_sensitive_key(key: &str) -> bool {
    let lower = key.to_lowercase();
    lower == "key"
        || (lower.contains("author") && !lower.contains("authoriz"))
        || SENSITIVE_KEY_PATTERNS.iter().any(|pat| lower.contains(pat))
}

/// Returns `true` when the string value itself looks sensitive:
/// - A URL carrying `?token=`, `&token=`, `signature=`, or `expires=`
/// - A JWT (`eyJ`-prefixed, three base64url-ish segments separated by `.`)
/// - A bunny.net account API key (two concatenated UUIDs, 72 chars)
pub fn is_sensitive_value(value: &str) -> bool {
    is_signed_url(value) || is_jwt(value) || is_account_api_key(value)
}

fn is_signed_url(s: &str) -> bool {
    s.contains("?token=")
        || s.contains("&token=")
        || s.contains("signature=")
        || s.contains("expires=")
}

fn is_jwt(s: &str) -> bool {
    // A JWT has exactly two dots splitting three non-empty, base64url-ish
    // segments. Require the standard `eyJ` header prefix (base64url of `{"`)
    // so three-label hostnames ("kiki.bunny.net") and version strings
    // ("1.2.3") don't false-positive.
    if !s.starts_with("eyJ") {
        return false;
    }
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

/// bunny.net account API keys are two concatenated UUIDs (72 chars).
fn is_account_api_key(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 72 && is_uuid(&bytes[..36]) && is_uuid(&bytes[36..])
}

fn is_uuid(bytes: &[u8]) -> bool {
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => *b == b'-',
            _ => b.is_ascii_hexdigit(),
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
            ("Amount", json!(2.24)),
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
    fn bare_key_field_is_redacted() {
        // GET /apikey returns the account API key under exactly "Key".
        let result = redacted(json!({ "Key": "eda66cfe-8fd7-4040-997f-77a6c66fe488" }));
        assert_eq!(result["Key"], "<redacted>");
    }

    #[test]
    fn key_substrings_stay_readable() {
        let result = redacted(json!({ "KeyId": 42, "errorKey": "invalid_plan_type" }));
        assert_eq!(result["KeyId"], 42);
        assert_eq!(result["errorKey"], "invalid_plan_type");
    }

    #[test]
    fn deployment_key_is_redacted() {
        let result = redacted(json!({ "DeploymentKey": "e5bb2cc3-0b2e-49e2-8858" }));
        assert_eq!(result["DeploymentKey"], "<redacted>");
    }

    #[test]
    fn account_api_key_value_redacted_under_any_field() {
        // Two concatenated UUIDs — the bunny.net account API key shape.
        let key = "eda66cfe-8fd7-4040-997f-77a6c66fe488ea41a773-201d-4cbf-81df-1735d605b486";
        let result = redacted(json!({ "SomeHarmlessField": key }));
        assert_eq!(result["SomeHarmlessField"], "<redacted>");
    }

    #[test]
    fn author_name_fields_redacted_but_resource_names_kept() {
        let result = redacted(json!({
            "Author": "Jane Doe",
            "AuthorEmail": "jane@real-company.com",
            "FirstName": "Jane",
            "Name": "hoppy-test-zone",
            "DefaultHostname": "zone.b-cdn.net"
        }));
        assert_eq!(result["Author"], "<redacted>");
        assert_eq!(result["AuthorEmail"], "<redacted>");
        assert_eq!(result["FirstName"], "<redacted>");
        assert_eq!(result["Name"], "hoppy-test-zone");
        assert_eq!(result["DefaultHostname"], "zone.b-cdn.net");
    }

    #[test]
    fn authorization_family_fields_not_redacted() {
        // "author" must not swallow authorization-family config fields
        // (real fields on the Shield API Guardian surface).
        let result = redacted(json!({
            "authorizationConfiguration": "{\"type\":\"bearer\"}",
            "validateAuthorization": "strict",
            "UnauthorizedReason": "expired"
        }));
        assert_eq!(
            result["authorizationConfiguration"],
            "{\"type\":\"bearer\"}"
        );
        assert_eq!(result["validateAuthorization"], "strict");
        assert_eq!(result["UnauthorizedReason"], "expired");
    }

    #[test]
    fn single_uuid_not_redacted() {
        // Plain GUIDs are identifiers (video guids, app ids) — keep them.
        let result = redacted(json!({ "guid": "7ddb2cac-63f5-46c0-beed-f6566e0f6a07" }));
        assert_eq!(result["guid"], "7ddb2cac-63f5-46c0-beed-f6566e0f6a07");
    }

    #[test]
    fn hostnames_and_versions_not_jwt_false_positives() {
        let result = redacted(json!({
            "Nameserver1": "kiki.bunny.net",
            "DefaultHostname": "new-script.b-cdn.net",
            "version": "1.2.3"
        }));
        assert_eq!(result["Nameserver1"], "kiki.bunny.net");
        assert_eq!(result["DefaultHostname"], "new-script.b-cdn.net");
        assert_eq!(result["version"], "1.2.3");
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
