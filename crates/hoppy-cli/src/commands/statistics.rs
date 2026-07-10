use crate::auth;
use crate::cli::OutputFormat;
use crate::date;
use anyhow::{Context, Result};
use bunny_net_api::core::StatisticsQuery;

#[derive(serde::Serialize, tabled::Tabled)]
struct AccountStatsRow {
    #[tabled(rename = "Metric")]
    metric: String,
    #[tabled(rename = "Value")]
    value: String,
}

/// Filters and chart-series selectors for the account statistics command.
///
/// Grouped into a struct so the dispatch in `main.rs` doesn't have to thread a
/// dozen positional arguments through `handle`.
pub struct StatisticsArgs<'a> {
    pub date_from: Option<&'a str>,
    pub date_to: Option<&'a str>,
    pub pull_zone: Option<i64>,
    pub server_zone_id: Option<i64>,
    pub hourly: bool,
    pub load_errors: bool,
    pub load_origin_response_times: bool,
    pub load_origin_traffic: bool,
    pub load_requests_served: bool,
    pub load_bandwidth_used: bool,
    pub load_origin_shield_bandwidth: bool,
    pub load_geographic_traffic_distribution: bool,
    pub load_user_balance_history: bool,
}

pub async fn handle(
    format: OutputFormat,
    debug: bool,
    record: Option<&str>,
    args: StatisticsArgs<'_>,
) -> Result<()> {
    let date_from = date::normalise_datetime_opt(args.date_from)?;
    let date_to = date::normalise_datetime_opt(args.date_to)?;
    let hourly = args.hourly;
    let client = auth::core_client(debug, record)?;
    let query = StatisticsQuery {
        date_from: date_from.as_deref(),
        date_to: date_to.as_deref(),
        pull_zone: args.pull_zone,
        server_zone_id: args.server_zone_id,
        hourly,
        load_errors: args.load_errors,
        load_origin_response_times: args.load_origin_response_times,
        load_origin_traffic: args.load_origin_traffic,
        load_requests_served: args.load_requests_served,
        load_bandwidth_used: args.load_bandwidth_used,
        load_origin_shield_bandwidth: args.load_origin_shield_bandwidth,
        load_geographic_traffic_distribution: args.load_geographic_traffic_distribution,
        load_user_balance_history: args.load_user_balance_history,
    };
    let stats = client.get_statistics(&query).await?;

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
        if hourly {
            crate::output::hints::tip(
                "hourly buckets aren't shown in table view — use --format json for the per-hour chart data (e.g. .BandwidthUsedChart)",
            );
        }
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
