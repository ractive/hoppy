use crate::auth;
use crate::cli::{OutputFormat, VideoLibraryAction};
use crate::date;
use anyhow::{Context, Result};

#[derive(serde::Serialize, tabled::Tabled)]
struct StatsRow {
    #[tabled(rename = "Metric")]
    metric: String,
    #[tabled(rename = "Value")]
    value: String,
}

pub async fn handle(
    action: &VideoLibraryAction,
    format: OutputFormat,
    debug: bool,
    dry_run: bool,
    reveal: bool,
    record: Option<&str>,
) -> Result<()> {
    let client = auth::core_client(&auth::ClientOpts {
        debug,
        dry_run,
        record,
        reveal_secrets: reveal,
    })?;

    match action {
        VideoLibraryAction::DrmStatistics {
            id,
            date_from,
            date_to,
        } => {
            let date_from = date::normalise_datetime_opt(date_from.as_deref())?;
            let date_to = date::normalise_datetime_opt(date_to.as_deref())?;
            let stats = client
                .get_video_library_drm_statistics(*id, date_from.as_deref(), date_to.as_deref())
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let rows = vec![StatsRow {
                    metric: "Total Licenses Issued".to_string(),
                    value: stats.total_licenses_issued.to_string(),
                }];
                crate::output::print_data(&rows, format);
            }
        }
        VideoLibraryAction::TranscribingStatistics {
            id,
            date_from,
            date_to,
        } => {
            let date_from = date::normalise_datetime_opt(date_from.as_deref())?;
            let date_to = date::normalise_datetime_opt(date_to.as_deref())?;
            let stats = client
                .get_video_library_transcribing_statistics(
                    *id,
                    date_from.as_deref(),
                    date_to.as_deref(),
                )
                .await?;
            if let OutputFormat::Json = format {
                let json =
                    serde_json::to_string_pretty(&stats).context("failed to serialize to JSON")?;
                println!("{json}");
            } else {
                let rows = vec![StatsRow {
                    metric: "Total Transcription Seconds".to_string(),
                    value: stats.total_transcription_seconds.to_string(),
                }];
                crate::output::print_data(&rows, format);
            }
        }
    }
    Ok(())
}
