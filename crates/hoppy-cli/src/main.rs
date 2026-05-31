mod auth;
mod cli;
mod commands;
mod date;
mod output;
mod progress;
mod redact;

use clap::Parser;
use clap_complete::generate;
use cli::{Cli, Commands};
use redact::RedactConfig;

fn main() {
    let cli = Cli::parse();

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
            commands::auth::handle(action, cli.format, cli.debug, cli.yes, record).await
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
            commands::stream::handle(action, cli.format, cli.debug, cli.yes, cli.quiet, record)
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
            commands::database::handle(action, cli.format, cli.debug, cli.yes, record, &redact_cfg)
                .await
        }
        Commands::Statistics {
            date_from,
            date_to,
            pull_zone,
            hourly,
        } => {
            commands::statistics::handle(
                cli.format,
                cli.debug,
                record,
                date_from.as_deref(),
                date_to.as_deref(),
                *pull_zone,
                *hourly,
            )
            .await
        }
        Commands::VideoLibrary { action } => {
            commands::video_library::handle(action, cli.format, cli.debug, record).await
        }
        Commands::Purge { url } => commands::purge::handle(url, cli.debug, record).await,
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
