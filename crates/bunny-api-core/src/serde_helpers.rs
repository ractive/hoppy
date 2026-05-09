//! Forward-compatible serde helpers for repr-based enums.
//!
//! Bunny.net adds new enum variants without API versioning. A naive
//! `Deserialize_repr` impl panics on unknown values (e.g. `OriginType: 5` for
//! Magic-Container-backed Pull Zones — added after `OriginType` was modelled
//! against `0/2/3/4`). To keep the CLI usable when the API ships a new value,
//! response types use [`deserialize_repr_option`] on every `Option<EnumType>`
//! field that wraps a `Serialize_repr/Deserialize_repr` enum: an unrecognised
//! integer deserialises to `None` instead of failing.
//!
//! Round-trip: an unknown value is *lost* on echo (deserialises to `None`,
//! serialises as absent). This is the documented trade-off — losing a single
//! field is preferable to the entire response failing.
//!
//! See `decision-log.md` ("forward-compat enum deserialisation") and
//! `api/bunny-api-quirks.md` for context.

use serde::Deserialize;
use serde::de::{DeserializeOwned, Deserializer};

/// Deserialize an `Option<T>` where `T` uses `Deserialize_repr`. On unknown
/// integer values, return `None` instead of erroring.
///
/// Use as `#[serde(default, deserialize_with = "deserialize_repr_option")]`.
pub fn deserialize_repr_option<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let raw = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(raw.and_then(|v| {
        if v.is_null() {
            None
        } else {
            serde_json::from_value::<T>(v).ok()
        }
    }))
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
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

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

    // ── deserialize_repr_option tests ────────────────────────────────────────

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    enum Color {
        Red = 0,
        Green = 1,
    }

    #[derive(Debug, Deserialize)]
    struct Wrapper {
        #[serde(default, deserialize_with = "deserialize_repr_option")]
        color: Option<Color>,
    }

    #[test]
    fn known_value_deserialises() {
        let w: Wrapper = serde_json::from_str(r#"{"color":1}"#).unwrap();
        assert_eq!(w.color, Some(Color::Green));
    }

    #[test]
    fn unknown_value_becomes_none() {
        let w: Wrapper = serde_json::from_str(r#"{"color":99}"#).unwrap();
        assert_eq!(w.color, None);
    }

    #[test]
    fn missing_field_uses_default() {
        let w: Wrapper = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(w.color, None);
    }

    #[test]
    fn null_value_becomes_none() {
        let w: Wrapper = serde_json::from_str(r#"{"color":null}"#).unwrap();
        assert_eq!(w.color, None);
    }

    #[test]
    fn round_trip_drops_unknown() {
        // Documented trade-off — round-tripping loses unknown values.
        #[derive(Debug, Deserialize, Serialize)]
        struct W {
            #[serde(
                default,
                skip_serializing_if = "Option::is_none",
                deserialize_with = "deserialize_repr_option"
            )]
            color: Option<Color>,
        }
        let w: W = serde_json::from_str(r#"{"color":99}"#).unwrap();
        let back = serde_json::to_string(&w).unwrap();
        assert_eq!(back, "{}");
    }
}
