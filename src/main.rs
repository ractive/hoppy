mod auth;
mod cli;
mod commands;
mod output;
mod progress;

use clap::Parser;
use clap_complete::generate;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let record = cli.record.as_deref();

    let result = match &cli.command {
        Commands::Auth { action } => {
            commands::auth::handle(action, cli.format, cli.debug, cli.yes, record).await
        }
        Commands::PullZone { action } => {
            commands::pull_zone::handle(action, cli.format, cli.debug, cli.yes, record).await
        }
        Commands::StorageZone { action } => {
            commands::storage_zone::handle(action, cli.format, cli.debug, cli.yes, record).await
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
            commands::container::handle(action, cli.format, cli.debug, cli.yes, record).await
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
