/// Date-string normalisation for CLI flags.
///
/// Several bunny.net API endpoints accept date-range strings in full ISO 8601
/// datetime form (`YYYY-MM-DDT00:00:00Z`).  Requiring users to type the full
/// timestamp is tedious.  This module provides helpers that:
///
/// * accept date-only input (`YYYY-MM-DD`) and expand it to midnight UTC
///   (`YYYY-MM-DDT00:00:00Z`).
/// * pass through any value that already looks like an ISO 8601 datetime.
/// * translate the legacy US format used by the Shield event-logs endpoint
///   (`MM-dd-yyyy`) from ISO input (`YYYY-MM-DD`).
/// * emit a clear error message for input that matches neither format.
use anyhow::{Result, bail};

/// Accepted date formats listed in user-facing error messages.
const ISO_DATE_FORMAT: &str = "YYYY-MM-DD";
/// Anything starting with `YYYY-MM-DDT` is forwarded as-is — fractional
/// seconds and offset suffixes are accepted, so the user-facing label is
/// deliberately the broad RFC 3339 / ISO 8601 datetime form.
const ISO_DATETIME_FORMAT: &str = "RFC 3339 datetime (e.g. 2024-03-15T12:30:00Z)";

/// Normalise a date-range string to a full ISO 8601 datetime.
///
/// Accepted inputs:
/// * `YYYY-MM-DD`           → `YYYY-MM-DDT00:00:00Z`
/// * `YYYY-MM-DDT...` (any) → returned as-is
///
/// Returns an error with a human-readable message on any other input.
pub fn normalise_datetime(input: &str) -> Result<String> {
    // Already a datetime — pass through.
    if looks_like_datetime(input) {
        return Ok(input.to_owned());
    }

    // Date-only.
    if looks_like_iso_date(input) {
        return Ok(format!("{input}T00:00:00Z"));
    }

    bail!("invalid date {input:?} — accepted formats: {ISO_DATE_FORMAT} or {ISO_DATETIME_FORMAT}");
}

/// Same as [`normalise_datetime`] but operates on an `Option<&str>`,
/// returning `None` when the input is `None`.
pub fn normalise_datetime_opt(input: Option<&str>) -> Result<Option<String>> {
    input.map(normalise_datetime).transpose()
}

/// Convert an ISO date (`YYYY-MM-DD`) or US-format date (`MM-dd-yyyy`) to
/// the US `MM-dd-yyyy` format expected by the Shield event-logs endpoint.
///
/// Accepted inputs:
/// * `YYYY-MM-DD`   → `MM-dd-yyyy`
/// * `MM-dd-yyyy`   → returned as-is (already in target format)
///
/// Returns an error on any other input.
pub fn normalise_shield_date(input: &str) -> Result<String> {
    if looks_like_iso_date(input) {
        // Parse YYYY-MM-DD and reformat as MM-dd-yyyy.
        let parts: Vec<&str> = input.split('-').collect();
        if parts.len() == 3 {
            let yyyy = parts[0];
            let mm = parts[1];
            let dd = parts[2];
            return Ok(format!("{mm}-{dd}-{yyyy}"));
        }
    }

    if looks_like_us_date(input) {
        return Ok(input.to_owned());
    }

    bail!("invalid date {input:?} — accepted formats: {ISO_DATE_FORMAT} or MM-dd-yyyy");
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Returns `true` if the string looks like `YYYY-MM-DD` (10 chars, digit groups).
fn looks_like_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit)
}

/// Returns `true` if the string starts with `YYYY-MM-DDT` (contains a `T`
/// after the date part), indicating it's already a datetime string.
///
/// Operates on bytes only — slicing `&s[..10]` would panic on non-ASCII
/// input that lands in the middle of a UTF-8 codepoint.
fn looks_like_datetime(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() < 11 || b[10] != b'T' {
        return false;
    }
    b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..10].iter().all(u8::is_ascii_digit)
}

/// Returns `true` if the string looks like `MM-dd-yyyy` (10 chars, 2-2-4 digit groups).
fn looks_like_us_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[2] == b'-'
        && b[5] == b'-'
        && b[..2].iter().all(u8::is_ascii_digit)
        && b[3..5].iter().all(u8::is_ascii_digit)
        && b[6..10].iter().all(u8::is_ascii_digit)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_only_expands_to_midnight() {
        assert_eq!(
            normalise_datetime("2024-03-15").unwrap(),
            "2024-03-15T00:00:00Z"
        );
    }

    #[test]
    fn full_datetime_passes_through() {
        assert_eq!(
            normalise_datetime("2024-03-15T12:30:00Z").unwrap(),
            "2024-03-15T12:30:00Z"
        );
    }

    #[test]
    fn invalid_format_returns_error() {
        assert!(normalise_datetime("15/03/2024").is_err());
        assert!(normalise_datetime("march 15").is_err());
        assert!(normalise_datetime("").is_err());
    }

    #[test]
    fn opt_none_passes_through() {
        assert_eq!(normalise_datetime_opt(None).unwrap(), None);
    }

    #[test]
    fn opt_some_normalises() {
        assert_eq!(
            normalise_datetime_opt(Some("2024-03-15")).unwrap(),
            Some("2024-03-15T00:00:00Z".to_owned())
        );
    }

    #[test]
    fn shield_date_iso_to_us() {
        assert_eq!(normalise_shield_date("2024-03-15").unwrap(), "03-15-2024");
    }

    #[test]
    fn shield_date_us_passthrough() {
        assert_eq!(normalise_shield_date("03-15-2024").unwrap(), "03-15-2024");
    }

    #[test]
    fn shield_date_invalid_returns_error() {
        assert!(normalise_shield_date("2024/03/15").is_err());
    }

    #[test]
    fn non_ascii_input_does_not_panic() {
        // Multi-byte UTF-8 input used to panic when looks_like_datetime
        // sliced &s[..10] across a codepoint boundary. We only assert
        // that the call returns (no panic) — the Ok/Err split depends on
        // the prefix shape.
        let _ = normalise_datetime("日本語のテスト文");
        let _ = normalise_datetime("éééé-éé-éé");
        let _ = normalise_shield_date("日本語のテスト文");
    }
}
