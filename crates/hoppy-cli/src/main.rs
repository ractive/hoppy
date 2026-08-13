mod auth;
mod cli;
mod commands;
mod date;
mod output;
mod progress;
mod redact;

use clap_complete::generate;
use cli::{Cli, Commands};
use redact::RedactConfig;

fn main() {
    let cli = cli::parse_or_exit();

    // Propagate --no-redact to the recording layer via env var, BEFORE the
    // Tokio runtime is constructed. The env-var indirection avoids threading
    // a flag through every domain client builder; `bunny-net-api` reads
    // HOPPY_NO_REDACT=1 inside `maybe_record_response` before writing
    // fixtures to disk.
    //
    // SAFETY: we are still single-threaded — the Tokio runtime has not been
    // built yet, so no other thread can be observing the environment.
    if cli.no_redact {
        unsafe {
            std::env::set_var("HOPPY_NO_REDACT", "1");
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");
    runtime.block_on(run(cli));
}

async fn run(cli: Cli) {
    let record = cli.record.as_deref();
    let redact_cfg = RedactConfig::new(cli.reveal, cli.reveal_env.clone());
    // `--dry-run` skips confirmation prompts too — the mutation is blocked at
    // the client layer regardless, so there is nothing to confirm.
    let yes = cli.yes || cli.dry_run;

    // Hints are off when --no-hints or --quiet is set, or whenever output is
    // machine readable (`--format json`) so paired stdout/stderr stays clean.
    let hints_enabled =
        !cli.no_hints && !cli.quiet && !matches!(cli.format, cli::OutputFormat::Json);
    output::hints::set_enabled(hints_enabled);

    let result = match &cli.command {
        Commands::Auth { action } => {
            commands::auth::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                cli.quiet,
                record,
            )
            .await
        }
        Commands::PullZone { action } => {
            commands::pull_zone::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::StorageZone { action } => {
            commands::storage_zone::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::Storage { action } => {
            commands::storage::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                cli.quiet,
                record,
                redact_cfg.reveal_all,
            )
            .await
        }
        Commands::Dns { action } => {
            commands::dns::handle(action, cli.format, cli.debug, cli.dry_run, yes, record).await
        }
        Commands::Stream { action } => {
            commands::stream::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                cli.quiet,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::Shield { action } => {
            commands::shield::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                record,
                redact_cfg.reveal_all,
            )
            .await
        }
        Commands::Script { action } => {
            commands::script::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                record,
                redact_cfg.reveal_all,
            )
            .await
        }
        Commands::Container { action } => {
            commands::container::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::Db { action } => {
            commands::database::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                yes,
                cli.quiet,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::Statistics {
            date_from,
            date_to,
            pull_zone,
            server_zone_id,
            hourly,
            load_errors,
            load_origin_response_times,
            load_origin_traffic,
            load_requests_served,
            load_bandwidth_used,
            load_origin_shield_bandwidth,
            load_geographic_traffic_distribution,
            load_user_balance_history,
        } => {
            commands::statistics::handle(
                cli.format,
                cli.debug,
                cli.dry_run,
                record,
                commands::statistics::StatisticsArgs {
                    date_from: date_from.as_deref(),
                    date_to: date_to.as_deref(),
                    pull_zone: *pull_zone,
                    server_zone_id: *server_zone_id,
                    hourly: *hourly,
                    load_errors: *load_errors,
                    load_origin_response_times: *load_origin_response_times,
                    load_origin_traffic: *load_origin_traffic,
                    load_requests_served: *load_requests_served,
                    load_bandwidth_used: *load_bandwidth_used,
                    load_origin_shield_bandwidth: *load_origin_shield_bandwidth,
                    load_geographic_traffic_distribution: *load_geographic_traffic_distribution,
                    load_user_balance_history: *load_user_balance_history,
                },
            )
            .await
        }
        Commands::Logs { action } => {
            commands::logs::handle(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                cli.quiet,
                record,
                redact_cfg.reveal_all,
            )
            .await
        }
        Commands::VideoLibrary { action } => {
            commands::video_library::handle(action, cli.format, cli.debug, cli.dry_run, record)
                .await
        }
        Commands::Purge {
            url,
            exact_path,
            is_async,
        } => {
            commands::purge::handle(
                url,
                *exact_path,
                *is_async,
                cli.format,
                cli.debug,
                cli.dry_run,
                record,
            )
            .await
        }
        Commands::Apikey { action } => {
            commands::account::handle_apikey(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                cli.reveal,
                record,
            )
            .await
        }
        Commands::Billing { action } => {
            commands::account::handle_billing(
                action,
                cli.format,
                cli.debug,
                cli.dry_run,
                cli.quiet,
                cli.reveal,
                record,
            )
            .await
        }
        Commands::Region { action } => {
            commands::account::handle_region(action, cli.format, cli.debug, cli.dry_run, record)
                .await
        }
        Commands::Country { action } => {
            commands::account::handle_country(action, cli.format, cli.debug, cli.dry_run, record)
                .await
        }
        Commands::Search { query, from, size } => {
            commands::account::handle_search(
                query,
                *from,
                *size,
                cli.format,
                cli.debug,
                cli.dry_run,
                record,
            )
            .await
        }
        Commands::User { action } => {
            commands::account::handle_user(action, cli.format, cli.debug, cli.dry_run, record).await
        }
        Commands::Completions { shell } => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            generate(*shell, &mut cmd, "hoppy", &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(err) = result {
        if let Some(skipped) = find_dry_run_skipped(&err) {
            print_dry_run_preview(skipped, cli.format);
            return;
        }
        output::print_error(&format!("{err:#}"), cli.format);
        std::process::exit(1);
    }
}

/// Walk the error chain looking for a [`bunny_net_api::dry_run::DryRunSkipped`],
/// which any domain client returns instead of actually sending a mutating
/// request under `--dry-run`. Walking the chain (rather than matching the
/// top-level error) means this still finds the marker after callers wrap it
/// with `anyhow::Context`.
fn find_dry_run_skipped(err: &anyhow::Error) -> Option<&bunny_net_api::dry_run::DryRunSkipped> {
    err.chain()
        .find_map(|e| e.downcast_ref::<bunny_net_api::dry_run::DryRunSkipped>())
}

/// Render the dry-run preview and exit 0 — a blocked mutation is the
/// intended, successful outcome of `--dry-run`, not a failure.
///
/// `--format json` prints a machine-readable envelope to stdout (matching
/// the `print_mutation_result` envelope contract elsewhere in the CLI);
/// `table`/`text` print an `[dry-run]`-prefixed preview to stderr, keeping
/// stdout pipe-clean either way.
fn print_dry_run_preview(
    skipped: &bunny_net_api::dry_run::DryRunSkipped,
    format: cli::OutputFormat,
) {
    match format {
        cli::OutputFormat::Json => {
            let body = skipped
                .body
                .as_deref()
                .and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok())
                .or_else(|| skipped.body.clone().map(serde_json::Value::String));
            let mut envelope = serde_json::json!({
                "status": "dry-run",
                "method": skipped.method,
                "url": skipped.url,
            });
            if let Some(body) = body {
                envelope["body"] = body;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| envelope.to_string())
            );
        }
        cli::OutputFormat::Table | cli::OutputFormat::Text => {
            eprintln!("[dry-run] Would send: {} {}", skipped.method, skipped.url);
            if let Some(body) = &skipped.body {
                eprintln!("[dry-run] Body: {body}");
            }
        }
    }
}
