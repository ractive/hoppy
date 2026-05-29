use crate::cli::OutputFormat;
use crate::redact::{RedactConfig, redact_env_in_json, redact_secrets_in_json};
use serde::Serialize;
use tabled::Tabled;

/// JSON wrapper for paginated list output — includes the pagination envelope
/// (CurrentPage, TotalItems, HasMoreItems) instead of a bare items array.
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaginatedListJson<'a, T> {
    pub items: &'a [T],
    pub current_page: i64,
    pub total_items: i64,
    pub has_more_items: bool,
}

/// Print data in the requested format to stdout.
pub fn print_data<T: Serialize + Tabled>(items: &[T], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(items).expect("failed to serialize to JSON");
            println!("{json}");
        }
        OutputFormat::Table => {
            if items.is_empty() {
                eprintln!("No results.");
            } else {
                let table = tabled::Table::new(items).to_string();
                println!("{table}");
            }
        }
        OutputFormat::Text => {
            // Tab-separated values — easy to parse with awk/cut
            let json_value =
                serde_json::to_value(items).expect("failed to serialize for text output");
            if let Some(arr) = json_value.as_array() {
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        let vals: Vec<String> = obj.values().map(format_text_value).collect();
                        println!("{}", vals.join("\t"));
                    }
                }
            }
        }
    }
}

/// Print a single item in the requested format.
pub fn print_single<T: Serialize + Tabled>(item: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(item).expect("failed to serialize to JSON");
            println!("{json}");
        }
        OutputFormat::Table => {
            let table = tabled::Table::new(std::iter::once(item)).to_string();
            println!("{table}");
        }
        OutputFormat::Text => {
            let json_value =
                serde_json::to_value(item).expect("failed to serialize for text output");
            if let Some(obj) = json_value.as_object() {
                for (key, val) in obj {
                    println!("{key}\t{}", format_text_value(val));
                }
            }
        }
    }
}

fn format_text_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Print a single item as a vertical Field/Value table.
///
/// For `Table`/`Text` formats: serialises to JSON, walks the top-level keys
/// in order, and renders a 2-column table with columns `Field` / `Value`.
/// Strings are printed as-is; null → `-`; primitive arrays joined with `, `;
/// nested arrays/objects collapsed to `<N items>` / `<object: K fields>` so
/// they don't blow out the Value column on terminals. After the table, a
/// stderr hint suggests the JSON view for inspecting the collapsed fields.
///
/// For `Json`: behaves like `print_single` but applies redaction first.
/// `cmd` is the command suffix used in the follow-up hint that nudges
/// users at the JSON view of collapsed fields. Pass something like
/// `"container app get --id 123"`; the helper builds the full
/// `hoppy --format json … | jq .<field>` hint. Pass `None` to disable.
pub fn print_single_vertical_with_cmd<T: Serialize>(
    item: &T,
    format: OutputFormat,
    redact_cfg: &RedactConfig,
    cmd: Option<&str>,
) {
    let mut value = serde_json::to_value(item).expect("failed to serialize for vertical output");
    redact_secrets_in_json(&mut value, redact_cfg);
    redact_env_in_json(&mut value, redact_cfg);

    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(&value).expect("failed to serialize to JSON");
        println!("{json}");
        return;
    }

    #[derive(Serialize, Tabled)]
    struct FieldRow {
        #[tabled(rename = "Field")]
        field: String,
        #[tabled(rename = "Value")]
        value: String,
    }

    if let Some(obj) = value.as_object() {
        let mut nested_fields: Vec<String> = Vec::new();
        let rows: Vec<FieldRow> = obj
            .iter()
            .map(|(k, v)| {
                let (rendered, nested) = format_vertical_value(v);
                if nested {
                    nested_fields.push(k.clone());
                }
                FieldRow {
                    field: k.clone(),
                    value: rendered,
                }
            })
            .collect();

        match format {
            OutputFormat::Table => {
                if rows.is_empty() {
                    eprintln!("No data.");
                } else {
                    let table = tabled::Table::new(&rows).to_string();
                    println!("{table}");
                }
            }
            OutputFormat::Text => {
                for row in &rows {
                    println!("{}\t{}", row.field, row.value);
                }
            }
            OutputFormat::Json => unreachable!(),
        }

        if !nested_fields.is_empty() && matches!(format, OutputFormat::Table | OutputFormat::Text) {
            // Pick the first nested field for the jq example.
            let example = &nested_fields[0];
            let cmd = cmd.unwrap_or("<command>");
            hints::tip(&format!(
                "view nested fields as JSON: hoppy --format json {cmd} | jq .{example}"
            ));
        }
    }
}

/// Render a JSON value for the Value column of the vertical table.
///
/// Returns `(rendered_string, is_nested)` — `is_nested` is true when the
/// value was an array of non-primitive items or an object, signalling
/// that the cell is a `<…>` placeholder rather than the real data.
fn format_vertical_value(v: &serde_json::Value) -> (String, bool) {
    match v {
        serde_json::Value::Null => ("-".to_owned(), false),
        serde_json::Value::String(s) => (s.clone(), false),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                ("<empty list>".to_owned(), false)
            } else if arr.iter().all(is_primitive) {
                let joined = arr
                    .iter()
                    .map(|item| match item {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                (joined, false)
            } else {
                let n = arr.len();
                let label = if n == 1 {
                    "<1 item>".to_owned()
                } else {
                    format!("<{n} items>")
                };
                (label, true)
            }
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                ("<empty object>".to_owned(), false)
            } else {
                let n = obj.len();
                let label = if n == 1 {
                    "<object: 1 field>".to_owned()
                } else {
                    format!("<object: {n} fields>")
                };
                (label, true)
            }
        }
        other => (other.to_string(), false),
    }
}

/// Maximum cell width for text columns in Table mode.
/// Values longer than this are truncated with a trailing `…`.
pub const TABLE_CELL_MAX: usize = 60;

/// Truncate a string so the rendered width is at most `max` Unicode scalar
/// values, appending `…` (counted in `max`) when shortened. Returns
/// `(rendered, was_truncated)`.
///
/// Truncation is always on character boundaries — never mid-codepoint.
/// Requires `max >= 1` so there is room for the ellipsis.
pub fn truncate_for_table(s: &str, max: usize) -> (String, bool) {
    debug_assert!(max >= 1, "truncate_for_table requires max >= 1");
    let char_count = s.chars().count();
    if char_count <= max {
        (s.to_owned(), false)
    } else {
        let keep = max.saturating_sub(1);
        let truncated: String = s.chars().take(keep).collect();
        (format!("{truncated}…"), true)
    }
}

fn is_primitive(v: &serde_json::Value) -> bool {
    matches!(
        v,
        serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_)
    )
}

/// Print an error in the appropriate format.
pub fn print_error(message: &str, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let err = serde_json::json!({
                "error": {
                    "message": message,
                }
            });
            eprintln!("{}", serde_json::to_string_pretty(&err).unwrap());
        }
        _ => {
            eprintln!("Error: {message}");
        }
    }
}

/// Drill-down hint helpers. Commands invoke [`hints::tip`] after their primary
/// output to suggest one or two natural follow-up commands on stderr.
///
/// Hints are globally toggled by `main()` based on `--no-hints` and the
/// chosen output format (json output suppresses hints so machine-readable
/// stdout stays paired with quiet stderr).
pub mod hints {
    use std::sync::atomic::{AtomicBool, Ordering};

    static ENABLED: AtomicBool = AtomicBool::new(false);

    /// Globally enable/disable hint emission. Called once from `main`.
    pub fn set_enabled(enabled: bool) {
        ENABLED.store(enabled, Ordering::Relaxed);
    }

    /// Return whether hints are currently enabled.
    pub fn is_enabled() -> bool {
        ENABLED.load(Ordering::Relaxed)
    }

    /// Print a single follow-up suggestion to stderr if hints are enabled.
    pub fn tip(next: &str) {
        if is_enabled() {
            eprintln!("tip: {next}");
        }
    }

    /// Print a set of related follow-up suggestions on consecutive lines.
    /// First line is prefixed with `tip:`, the rest with `  or:`.
    pub fn tips(nexts: &[&str]) {
        if !is_enabled() || nexts.is_empty() {
            return;
        }
        let mut iter = nexts.iter();
        if let Some(first) = iter.next() {
            eprintln!("tip: {first}");
        }
        for n in iter {
            eprintln!("  or: {n}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // -----------------------------------------------------------------------
    // format_vertical_value — plural/singular for items and fields
    // -----------------------------------------------------------------------

    #[test]
    fn vertical_array_zero_items_is_empty_list() {
        let v = json!([]);
        let (rendered, nested) = format_vertical_value(&v);
        assert_eq!(rendered, "<empty list>");
        assert!(!nested);
    }

    #[test]
    fn vertical_array_one_non_primitive_item_is_singular() {
        let v = json!([{"a": 1}]);
        let (rendered, nested) = format_vertical_value(&v);
        assert_eq!(rendered, "<1 item>");
        assert!(nested);
    }

    #[test]
    fn vertical_array_two_non_primitive_items_is_plural() {
        let v = json!([{"a": 1}, {"b": 2}]);
        let (rendered, nested) = format_vertical_value(&v);
        assert_eq!(rendered, "<2 items>");
        assert!(nested);
    }

    #[test]
    fn vertical_object_zero_fields_is_empty_object() {
        let v = json!({});
        let (rendered, nested) = format_vertical_value(&v);
        assert_eq!(rendered, "<empty object>");
        assert!(!nested);
    }

    #[test]
    fn vertical_object_one_field_is_singular() {
        let v = json!({"x": 1});
        let (rendered, nested) = format_vertical_value(&v);
        assert_eq!(rendered, "<object: 1 field>");
        assert!(nested);
    }

    #[test]
    fn vertical_object_two_fields_is_plural() {
        let v = json!({"x": 1, "y": 2});
        let (rendered, nested) = format_vertical_value(&v);
        assert_eq!(rendered, "<object: 2 fields>");
        assert!(nested);
    }

    // -----------------------------------------------------------------------
    // truncate_for_table
    // -----------------------------------------------------------------------

    #[test]
    fn truncate_short_string_unchanged() {
        let s = "hello";
        let (out, truncated) = truncate_for_table(s, 10);
        assert_eq!(out, "hello");
        assert!(!truncated);
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        let s = "abcde";
        let (out, truncated) = truncate_for_table(s, 5);
        assert_eq!(out, "abcde");
        assert!(!truncated);
    }

    #[test]
    fn truncate_long_string_adds_ellipsis() {
        let s = "a".repeat(70);
        let (out, truncated) = truncate_for_table(&s, TABLE_CELL_MAX);
        assert!(truncated);
        // Rendered width must not exceed max: 59 'a's + '…' = 60 chars.
        assert_eq!(out.chars().count(), TABLE_CELL_MAX);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn truncate_unicode_safe() {
        // Each emoji is one char (multiple bytes); we must not split mid-codepoint.
        let s = "😀".repeat(70);
        let (out, truncated) = truncate_for_table(&s, TABLE_CELL_MAX);
        assert!(truncated);
        // 59 emoji + '…' = 60 chars (in scalar values), but many more bytes.
        assert_eq!(out.chars().count(), TABLE_CELL_MAX);
        assert!(out.ends_with('…'));
    }
}
