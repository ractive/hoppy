//! Forward-compatible serde helpers.
//!
//! Bunny.net adds enum values without API versioning. The crate-wide rule
//! (decision-log "Response enums never fail on unknown values"):
//!
//! - **Integer-coded enums** are declared with the crate-internal
//!   `forward_compat_repr_enum!` macro (see `macros.rs`). It adds an
//!   `Unknown(n)` variant that captures any undeclared integer and serializes
//!   back to the same integer, so read-modify-write commands never drop or
//!   rewrite a value they do not understand. Response fields are plain
//!   `Option<Enum>` / `Enum`; no per-field helper is needed.
//! - **String-coded enums** carry a `#[serde(other)] Unknown` variant. It does
//!   not keep the raw string (serde limitation), so it is only used where the
//!   value is never written back.
//!
//! The helpers below cover other lenient-shape quirks of the API.

use serde::Deserialize;

/// Strict conversion from a wire integer to a *declared* enum variant.
///
/// Implemented by every enum generated with `forward_compat_repr_enum!`.
/// Deserialization is lenient (`Unknown(n)`), so code that validates *user
/// input* (CLI flags, config) must use this instead of going through serde.
pub trait KnownRepr: Sized {
    /// The wire integer type.
    type Repr: Copy + std::fmt::Display;

    /// `None` when `n` is not a declared variant.
    fn try_from_known(n: Self::Repr) -> Option<Self>;
}
use serde::de::Deserializer;

/// Deserialize a `Vec<T>` that tolerates a JSON `null` value.
///
/// The bunny.net scan API returns `"Records": null` for in-progress jobs
/// (before any records have been discovered). Serde's built-in `#[serde(default)]`
/// only fills in the default when the field is *absent*; it still fails on an
/// explicit `null`. This helper coerces `null` → empty `Vec`.
///
/// Use as `#[serde(default, deserialize_with = "null_to_empty_vec")]`.
pub fn null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Deserialize an `Option<String>` that is *tolerant* of non-string JSON values.
///
/// The bunny.net API documents `OptimizerClasses` as a JSON string (a serialised
/// map of class-name → URL parameters), but returns an empty array `[]` when
/// no classes are configured. This helper returns:
///
/// - `None` for JSON `null` or any non-string value (array, object, integer, …)
/// - `Some(s)` for a JSON string value
///
/// Use as `#[serde(default, deserialize_with = "deserialize_string_lossy_option")]`.
pub fn deserialize_string_lossy_option<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(raw.and_then(|v| {
        if let serde_json::Value::String(s) = v {
            Some(s)
        } else {
            None
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    // ── null_to_empty_vec tests ──────────────────────────────────────────────

    #[derive(Debug, Deserialize)]
    struct VecWrapper {
        #[serde(default, deserialize_with = "null_to_empty_vec")]
        items: Vec<u32>,
    }

    #[test]
    fn null_vec_becomes_empty() {
        let w: VecWrapper = serde_json::from_str(r#"{"items":null}"#).unwrap();
        assert!(w.items.is_empty());
    }

    #[test]
    fn missing_vec_becomes_empty() {
        let w: VecWrapper = serde_json::from_str(r#"{}"#).unwrap();
        assert!(w.items.is_empty());
    }

    #[test]
    fn populated_vec_is_preserved() {
        let w: VecWrapper = serde_json::from_str(r#"{"items":[1,2,3]}"#).unwrap();
        assert_eq!(w.items, vec![1, 2, 3]);
    }

    // ── deserialize_string_lossy_option tests ────────────────────────────────

    #[derive(Debug, Deserialize)]
    struct StringWrapper {
        #[serde(default, deserialize_with = "deserialize_string_lossy_option")]
        value: Option<String>,
    }

    #[test]
    fn string_lossy_null_becomes_none() {
        let w: StringWrapper = serde_json::from_str(r#"{"value":null}"#).unwrap();
        assert_eq!(w.value, None);
    }

    #[test]
    fn string_lossy_string_becomes_some() {
        let w: StringWrapper =
            serde_json::from_str(r#"{"value":"{\"thumb\":\"width=200\"}"}"#).unwrap();
        assert_eq!(w.value, Some("{\"thumb\":\"width=200\"}".to_owned()));
    }

    #[test]
    fn string_lossy_array_becomes_none() {
        let w: StringWrapper = serde_json::from_str(r#"{"value":[]}"#).unwrap();
        assert_eq!(w.value, None);
    }

    #[test]
    fn string_lossy_object_becomes_none() {
        let w: StringWrapper = serde_json::from_str(r#"{"value":{"a":1}}"#).unwrap();
        assert_eq!(w.value, None);
    }

    #[test]
    fn string_lossy_integer_becomes_none() {
        let w: StringWrapper = serde_json::from_str(r#"{"value":42}"#).unwrap();
        assert_eq!(w.value, None);
    }

    #[test]
    fn string_lossy_missing_field_becomes_none() {
        let w: StringWrapper = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(w.value, None);
    }
}
