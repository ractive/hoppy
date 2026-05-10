use crate::auth;
use crate::cli::OutputFormat;
use crate::date;
use anyhow::{Context, Result};

#[derive(serde::Serialize, tabled::Tabled)]
struct AccountStatsRow {
    #[tabled(rename = "Metric")]
    metric: String,
    #[tabled(rename = "Value")]
    value: String,
}

pub async fn handle(
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
    date_from: Option<&str>,
    date_to: Option<&str>,
    pull_zone: Option<i64>,
    hourly: bool,
) -> Result<()> {
    let date_from = date::normalise_datetime_opt(date_from)?;
    let date_to = date::normalise_datetime_opt(date_to)?;
    let client = auth::core_client(debug, record)?;
    let stats = client
        .get_statistics(date_from.as_deref(), date_to.as_deref(), pull_zone, hourly)
        .await?;

    if let OutputFormat::Json = format {
        let json = serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?;
        println!("{json}");
    } else {
        let rows = vec![
            AccountStatsRow {
                metric: "Total Bandwidth Used".to_string(),
                value: format_bytes(stats.total_bandwidth_used),
            },
            AccountStatsRow {
                metric: "Total Origin Traffic".to_string(),
                value: format_bytes(stats.total_origin_traffic),
            },
            AccountStatsRow {
                metric: "Avg Origin Response Time".to_string(),
                value: format!("{} ms", stats.average_origin_response_time),
            },
            AccountStatsRow {
                metric: "Total Requests Served".to_string(),
                value: stats.total_requests_served.to_string(),
            },
            AccountStatsRow {
                metric: "Cache Hit Rate".to_string(),
                value: format!("{:.2}%", stats.cache_hit_rate * 100.0),
            },
        ];
        crate::output::print_data(&rows, format);
    }
    Ok(())
}

fn format_bytes(bytes: i64) -> String {
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;
    const KB: f64 = 1_024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}
