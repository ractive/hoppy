use crate::cli::OutputFormat;
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
