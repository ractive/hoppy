use crate::cli::OutputFormat;
use serde::Serialize;
use tabled::Tabled;

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
                        let vals: Vec<String> =
                            obj.values().map(format_text_value).collect();
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
