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

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

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
