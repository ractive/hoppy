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

    // Hints are off when --no-hints or --quiet is set, or whenever output is
    // machine readable (`--format json`) so paired stdout/stderr stays clean.
    let hints_enabled =
        !cli.no_hints && !cli.quiet && !matches!(cli.format, cli::OutputFormat::Json);
    output::hints::set_enabled(hints_enabled);

    let result = match &cli.command {
        Commands::Auth { action } => {
            commands::auth::handle(action, cli.format, cli.debug, cli.yes, cli.quiet, record).await
        }
        Commands::PullZone { action } => {
            commands::pull_zone::handle(action, cli.format, cli.debug, cli.yes, record, &redact_cfg)
                .await
        }
        Commands::StorageZone { action } => {
            commands::storage_zone::handle(
                action,
                cli.format,
                cli.debug,
                cli.yes,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::Storage { action } => {
            commands::storage::handle(action, cli.format, cli.debug, cli.yes, cli.quiet, record)
                .await
        }
        Commands::Dns { action } => {
            commands::dns::handle(action, cli.format, cli.debug, cli.yes, record).await
        }
        Commands::Stream { action } => {
            commands::stream::handle(
                action,
                cli.format,
                cli.debug,
                cli.yes,
                cli.quiet,
                record,
                &redact_cfg,
            )
            .await
        }
        Commands::Shield { action } => {
            commands::shield::handle(action, cli.format, cli.debug, cli.yes, record).await
        }
        Commands::Script { action } => {
            commands::script::handle(action, cli.format, cli.debug, cli.yes, record).await
        }
        Commands::Container { action } => {
            commands::container::handle(action, cli.format, cli.debug, cli.yes, record, &redact_cfg)
                .await
        }
        Commands::Db { action } => {
            commands::database::handle(
                action,
                cli.format,
                cli.debug,
                cli.yes,
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
            commands::logs::handle(action, cli.format, cli.debug, cli.quiet, record).await
        }
        Commands::VideoLibrary { action } => {
            commands::video_library::handle(action, cli.format, cli.debug, record).await
        }
        Commands::Purge {
            url,
            exact_path,
            is_async,
        } => {
            commands::purge::handle(url, *exact_path, *is_async, cli.format, cli.debug, record)
                .await
        }
        Commands::Completions { shell } => {
            let mut cmd = <Cli as clap::CommandFactory>::command();
            generate(*shell, &mut cmd, "hoppy", &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(err) = result {
        output::print_error(&format!("{err:#}"), cli.format);
        std::process::exit(1);
    }
}
