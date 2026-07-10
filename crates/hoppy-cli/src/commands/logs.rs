use crate::auth;
use crate::cli::{LogsAction, OutputFormat};
use crate::output;
use anyhow::{Context, Result};
use bunny_net_api::logging::{LegacyLogParams, LogEntry, LogQueryParams};
use bunny_net_api::origin_errors::OriginErrorEntry;
use std::io;

// ---------------------------------------------------------------------------
// Display structs
// ---------------------------------------------------------------------------

/// Compact table/text row for a v2 CDN access log entry. The full detail set is
/// available via `--format json`.
#[derive(serde::Serialize, tabled::Tabled)]
struct LogEntryRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Status")]
    status_code: i32,
    #[tabled(rename = "Cache")]
    cache_status: String,
    #[tabled(rename = "Bytes")]
    bytes_sent: i64,
    #[tabled(rename = "Country")]
    country: String,
    #[tabled(rename = "Edge")]
    edge_location: String,
    #[tabled(rename = "URL")]
    url: String,
}

impl From<&LogEntry> for LogEntryRow {
    fn from(e: &LogEntry) -> Self {
        Self {
            timestamp: e.timestamp.clone(),
            status_code: e.status_code,
            cache_status: e.cache_status.clone().unwrap_or_default(),
            bytes_sent: e.bytes_sent,
            country: e.country_code.clone().unwrap_or_default(),
            edge_location: e.edge_location.clone().unwrap_or_default(),
            url: e.url.clone().unwrap_or_default(),
        }
    }
}

/// Table/text row for an origin error log entry.
#[derive(serde::Serialize, tabled::Tabled)]
struct OriginErrorRow {
    #[tabled(rename = "Timestamp")]
    timestamp: String,
    #[tabled(rename = "Status")]
    status_code: String,
    #[tabled(rename = "Error Code")]
    error_code: String,
    #[tabled(rename = "Server Zone")]
    server_zone: String,
    #[tabled(rename = "Log")]
    log: String,
}

impl From<&OriginErrorEntry> for OriginErrorRow {
    fn from(e: &OriginErrorEntry) -> Self {
        let (error_code, status_code, server_zone) = match &e.labels {
            Some(l) => (
                l.error_code.clone().unwrap_or_default(),
                l.status_code.clone().unwrap_or_default(),
                l.server_zone.clone().unwrap_or_default(),
            ),
            None => (String::new(), String::new(), String::new()),
        };
        Self {
            timestamp: e.timestamp.map(|t| t.to_string()).unwrap_or_default(),
            status_code,
            error_code,
            server_zone,
            log: e.log.clone().unwrap_or_default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn handle(
    action: &LogsAction,
    format: OutputFormat,
    debug: bool,
    quiet: bool,
    record: Option<&str>,
) -> Result<()> {
    match action {
        LogsAction::PullZone {
            id,
            from,
            to,
            status,
            cache_status,
            country,
            edge_location,
            remote_ip,
            url_contains,
            user_agent_contains,
            referer_contains,
            search,
            request_id,
            include_origin_shield,
            limit,
            offset,
            order,
            legacy,
            date,
            start,
            end,
            output,
        } => {
            let client = auth::logging_client(debug, record)?;

            if *legacy {
                let date = date
                    .as_deref()
                    .context("--date is required with --legacy (e.g. --date 07-08-26)")?;
                let params = LegacyLogParams {
                    start: *start,
                    end: *end,
                    ..Default::default()
                };
                // Stream the raw body straight to the sink instead of buffering.
                let written = match output {
                    Some(path) => {
                        let mut out = std::fs::File::create(path)
                            .with_context(|| format!("creating output file: {path}"))?;
                        let n = client
                            .stream_legacy_logs(date, *id, &params, &mut out)
                            .await?;
                        if !quiet {
                            eprintln!("Saved {n} bytes to {path}");
                        }
                        n
                    }
                    None => {
                        let stdout = io::stdout();
                        let mut handle = stdout.lock();
                        client
                            .stream_legacy_logs(date, *id, &params, &mut handle)
                            .await?
                    }
                };
                let _ = written;
                return Ok(());
            }

            // v2 structured query.
            let params = LogQueryParams {
                from: from.clone(),
                to: to.clone(),
                status: status.clone(),
                cache_status: cache_status.clone(),
                country: country.clone(),
                edge_location: edge_location.clone(),
                remote_ip: remote_ip.clone(),
                url_contains: url_contains.clone(),
                user_agent_contains: user_agent_contains.clone(),
                referer_contains: referer_contains.clone(),
                search: search.clone(),
                request_id: request_id.clone(),
                include_origin_shield: *include_origin_shield,
                limit: *limit,
                offset: *offset,
                order: order.clone(),
            };
            let resp = client.query_logs(*id, &params).await?;

            match (format, output) {
                (OutputFormat::Json, Some(path)) => {
                    let json =
                        serde_json::to_string_pretty(&resp).expect("failed to serialize to JSON");
                    std::fs::write(path, format!("{json}\n"))
                        .with_context(|| format!("writing output file: {path}"))?;
                    if !quiet {
                        eprintln!("Wrote {} entries to {path}", resp.data.len());
                    }
                }
                (OutputFormat::Json, None) => {
                    let json =
                        serde_json::to_string_pretty(&resp).expect("failed to serialize to JSON");
                    println!("{json}");
                }
                (_, out) => {
                    let rows: Vec<LogEntryRow> = resp.data.iter().map(LogEntryRow::from).collect();
                    if let Some(path) = out {
                        // Non-JSON to a file: write the tab-separated text form.
                        let text = render_text(&rows);
                        std::fs::write(path, text)
                            .with_context(|| format!("writing output file: {path}"))?;
                        if !quiet {
                            eprintln!("Wrote {} entries to {path}", rows.len());
                        }
                    } else {
                        output::print_data(&rows, format);
                    }
                    if !quiet && resp.pagination.has_more {
                        eprintln!(
                            "More results available — re-run with --offset {}",
                            resp.pagination.offset + i64::from(resp.pagination.returned)
                        );
                    }
                }
            }
        }
        LogsAction::OriginErrors { id, date } => {
            let client = auth::origin_errors_client(debug, record)?;
            let resp = client.get_origin_errors(*id, date).await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&resp).expect("failed to serialize to JSON");
                println!("{json}");
            } else {
                let rows: Vec<OriginErrorRow> =
                    resp.logs.iter().map(OriginErrorRow::from).collect();
                output::print_data(&rows, format);
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render rows as tab-separated lines (used when writing non-JSON to a file).
fn render_text<T: serde::Serialize>(rows: &[T]) -> String {
    use std::fmt::Write as _;
    let mut buf = String::new();
    let value = serde_json::to_value(rows).expect("serialize rows for text output");
    if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(obj) = item.as_object() {
                let vals: Vec<String> = obj
                    .values()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        other => other.to_string(),
                    })
                    .collect();
                let _ = writeln!(buf, "{}", vals.join("\t"));
            }
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use bunny_net_api::logging::LogEntry;
    use bunny_net_api::origin_errors::{OriginErrorEntry, OriginErrorLabels};

    fn sample_entry() -> LogEntry {
        LogEntry {
            timestamp: "2026-07-08T12:00:00.000Z".into(),
            pull_zone_id: 1,
            request_id: Some("abc".into()),
            cache_status: Some("HIT".into()),
            status_code: 200,
            bytes_sent: 1234,
            remote_ip: None,
            country_code: Some("EE".into()),
            edge_location: Some("TLL".into()),
            scheme: Some("https".into()),
            host: Some("cdn.example.com".into()),
            path: Some("/img.png".into()),
            url: Some("https://cdn.example.com/img.png".into()),
            user_agent: None,
            referer: None,
            body_bytes_sent: None,
            content_range: None,
            authorization_header: None,
            ja4_fingerprint: None,
            asn: None,
            asn_organization: None,
        }
    }

    #[test]
    fn log_entry_row_maps_fields() {
        let row = LogEntryRow::from(&sample_entry());
        assert_eq!(row.status_code, 200);
        assert_eq!(row.cache_status, "HIT");
        assert_eq!(row.country, "EE");
        assert_eq!(row.edge_location, "TLL");
        assert_eq!(row.url, "https://cdn.example.com/img.png");
    }

    #[test]
    fn log_entry_row_handles_missing_optionals() {
        let mut e = sample_entry();
        e.cache_status = None;
        e.country_code = None;
        e.url = None;
        let row = LogEntryRow::from(&e);
        assert_eq!(row.cache_status, "");
        assert_eq!(row.country, "");
        assert_eq!(row.url, "");
    }

    #[test]
    fn origin_error_row_extracts_labels() {
        let e = OriginErrorEntry {
            log_id: Some("id".into()),
            timestamp: Some(1728952065848),
            log: Some("{\"Message\":\"boom\"}".into()),
            labels: Some(OriginErrorLabels {
                error_code: Some("dns_lookup".into()),
                status_code: Some("502".into()),
                server_zone: Some("CA".into()),
            }),
        };
        let row = OriginErrorRow::from(&e);
        assert_eq!(row.timestamp, "1728952065848");
        assert_eq!(row.status_code, "502");
        assert_eq!(row.error_code, "dns_lookup");
        assert_eq!(row.server_zone, "CA");
    }

    #[test]
    fn origin_error_row_handles_missing_labels() {
        let e = OriginErrorEntry {
            log_id: None,
            timestamp: None,
            log: None,
            labels: None,
        };
        let row = OriginErrorRow::from(&e);
        assert_eq!(row.timestamp, "");
        assert_eq!(row.status_code, "");
        assert_eq!(row.error_code, "");
        assert_eq!(row.server_zone, "");
        assert_eq!(row.log, "");
    }

    #[test]
    fn render_text_tab_separates() {
        let rows = vec![OriginErrorRow::from(&OriginErrorEntry {
            log_id: None,
            timestamp: Some(1),
            log: Some("x".into()),
            labels: Some(OriginErrorLabels {
                error_code: Some("dns_lookup".into()),
                status_code: Some("502".into()),
                server_zone: Some("CA".into()),
            }),
        })];
        let text = render_text(&rows);
        assert!(text.contains('\t'));
        assert!(text.contains("dns_lookup"));
        assert!(text.ends_with('\n'));
    }
}
