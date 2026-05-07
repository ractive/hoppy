//! Cross-cutting secret redaction layer.
//!
//! Sensitive values (env-var values for Magic Containers, storage-zone
//! passwords, database tokens, …) are masked by default whenever the CLI
//! prints them. The user opts in to raw output with the global `--reveal`
//! flag (or, for env vars, the per-key `--reveal-env <KEY>`).
//!
//! Format precedence: redaction applies to JSON, table, and text output
//! identically, so a `--format json | jq` pipeline doesn't accidentally
//! leak a secret into a logfile.
//!
//! Naming conventions for the placeholder:
//! - empty / `null`  → `<unset>`
//! - non-empty       → `<set, length=N>` (N = char count of the raw value)

use std::collections::HashSet;

/// Runtime configuration for redacting secret-bearing fields.
#[derive(Debug, Clone, Default)]
pub struct RedactConfig {
    /// `--reveal`: bypass redaction for every secret-bearing field.
    pub reveal_all: bool,
    /// `--reveal-env KEY`: bypass redaction for these env-var names only
    /// (case-insensitive). Ignored unless the value being redacted is an
    /// env var and we know its name.
    pub reveal_env_keys: HashSet<String>,
}

impl RedactConfig {
    pub fn new(reveal_all: bool, reveal_env_keys: Vec<String>) -> Self {
        Self {
            reveal_all,
            reveal_env_keys: reveal_env_keys
                .into_iter()
                .map(|k| k.to_lowercase())
                .collect(),
        }
    }

    /// True if the env-var with this name should be revealed (raw value).
    pub fn reveal_env(&self, name: &str) -> bool {
        self.reveal_all || self.reveal_env_keys.contains(&name.to_lowercase())
    }

    /// True if a generic secret field (no env-var name) should be revealed.
    /// Used by storage-zone passwords (iter-19) and DB token mint (iter-20)
    /// to share the redaction surface.
    pub fn reveal_field(&self) -> bool {
        self.reveal_all
    }
}

/// Render a redacted placeholder for a string value.
pub fn placeholder(value: &str) -> String {
    if value.is_empty() {
        "<unset>".to_owned()
    } else {
        format!("<set, length={}>", value.chars().count())
    }
}

/// Heuristic: does a field name suggest it holds a secret?
///
/// Used to drive crate-wide audits; the `Application` / `ContainerTemplate`
/// path simply redacts every env-var value by default and does not need this
/// heuristic. iter-19 (storage-zone Password / ReadOnlyPassword) and iter-20
/// (DB token mint) consume this — keep public.
pub fn is_secret_field_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    // Allowlist: a few `_key` suffixes are not secrets in this codebase.
    const KEY_SUFFIX_NOT_SECRET: &[&str] = &[
        "zonesecuritykey", // PullZone — already gated separately
        "userpkkey",
        "publickey",
    ];
    if KEY_SUFFIX_NOT_SECRET.iter().any(|s| lower.ends_with(s)) {
        return false;
    }
    lower.ends_with("password")
        || lower.ends_with("_password")
        || lower.ends_with("secret")
        || lower.ends_with("_secret")
        || lower.ends_with("token")
        || lower.ends_with("_token")
        || lower.ends_with("apikey")
        || lower.ends_with("api_key")
        || lower.ends_with("_key")
        || lower.contains("credential")
}

/// Walk a JSON value and rewrite every string-typed field whose name matches
/// [`is_secret_field_name`] to a placeholder, unless `--reveal` opts in.
///
/// Used by storage-zone JSON output (Password / ReadOnlyPassword) and any
/// future endpoint that surfaces credentials directly. Non-string values
/// (numbers, nulls, nested objects) are walked but not rewritten.
pub fn redact_secrets_in_json(value: &mut serde_json::Value, config: &RedactConfig) {
    if config.reveal_field() {
        return;
    }
    walk_secrets(value);
}

fn walk_secrets(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if is_secret_field_name(k) {
                    if let serde_json::Value::String(raw) = v {
                        *v = serde_json::Value::String(placeholder(raw));
                    } else if v.is_null() {
                        *v = serde_json::Value::String(placeholder(""));
                    } else {
                        // Recurse — secret-named nested objects can still hold
                        // non-secret fields (rare).
                        walk_secrets(v);
                    }
                } else {
                    walk_secrets(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                walk_secrets(v);
            }
        }
        _ => {}
    }
}

/// Walk a JSON value and rewrite every `environmentVariables[*].value` to a
/// placeholder unless `--reveal` (or matching `--reveal-env`) opts in.
pub fn redact_env_in_json(value: &mut serde_json::Value, config: &RedactConfig) {
    if config.reveal_all {
        return;
    }
    walk(value, config);
}

fn walk(value: &mut serde_json::Value, config: &RedactConfig) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if k == "environmentVariables" || k == "environment_variables" {
                    if let serde_json::Value::Array(items) = v {
                        for item in items {
                            redact_env_var_object(item, config);
                        }
                    }
                } else {
                    walk(v, config);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                walk(v, config);
            }
        }
        _ => {}
    }
}

fn redact_env_var_object(item: &mut serde_json::Value, config: &RedactConfig) {
    let serde_json::Value::Object(map) = item else {
        return;
    };
    let name = map
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    if config.reveal_env(&name) {
        return;
    }
    if let Some(val) = map.get_mut("value") {
        let raw = val.as_str().unwrap_or("");
        *val = serde_json::Value::String(placeholder(raw));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn placeholder_empty() {
        assert_eq!(placeholder(""), "<unset>");
    }

    #[test]
    fn placeholder_non_empty() {
        assert_eq!(placeholder("abc"), "<set, length=3>");
        assert_eq!(placeholder("héllo"), "<set, length=5>");
    }

    #[test]
    fn is_secret_field_name_basic() {
        assert!(is_secret_field_name("password"));
        assert!(is_secret_field_name("Password"));
        assert!(is_secret_field_name("readOnlyPassword"));
        assert!(is_secret_field_name("api_key"));
        assert!(is_secret_field_name("authToken"));
        assert!(is_secret_field_name("clientSecret"));
        assert!(is_secret_field_name("dbCredential"));
        assert!(!is_secret_field_name("name"));
        assert!(!is_secret_field_name("hostName"));
    }

    #[test]
    fn redact_env_in_json_redacts_by_default() {
        let cfg = RedactConfig::default();
        let mut v = json!({
            "environmentVariables": [
                {"name": "DATABASE_URL", "value": "postgres://user:pw@h/db"},
                {"name": "EMPTY", "value": ""},
                {"name": "NULL_VAL", "value": null},
            ],
        });
        redact_env_in_json(&mut v, &cfg);
        assert_eq!(
            v["environmentVariables"][0]["value"],
            json!("<set, length=23>")
        );
        assert_eq!(v["environmentVariables"][1]["value"], json!("<unset>"));
        assert_eq!(v["environmentVariables"][2]["value"], json!("<unset>"));
    }

    #[test]
    fn redact_env_in_json_reveal_all() {
        let cfg = RedactConfig::new(true, vec![]);
        let mut v = json!({
            "environmentVariables": [{"name": "K", "value": "secret"}],
        });
        redact_env_in_json(&mut v, &cfg);
        assert_eq!(v["environmentVariables"][0]["value"], json!("secret"));
    }

    #[test]
    fn redact_env_in_json_reveal_specific_key() {
        let cfg = RedactConfig::new(false, vec!["PUBLIC_VAR".to_owned()]);
        let mut v = json!({
            "environmentVariables": [
                {"name": "PUBLIC_VAR", "value": "open"},
                {"name": "SECRET", "value": "hidden"},
            ],
        });
        redact_env_in_json(&mut v, &cfg);
        assert_eq!(v["environmentVariables"][0]["value"], json!("open"));
        assert_eq!(
            v["environmentVariables"][1]["value"],
            json!("<set, length=6>")
        );
    }

    #[test]
    fn redact_env_in_json_recurses() {
        let cfg = RedactConfig::default();
        let mut v = json!({
            "containerTemplates": [
                {"environmentVariables": [{"name": "K", "value": "v"}]},
            ],
        });
        redact_env_in_json(&mut v, &cfg);
        assert_eq!(
            v["containerTemplates"][0]["environmentVariables"][0]["value"],
            json!("<set, length=1>")
        );
    }
}
