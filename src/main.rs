mod auth;
mod cli;
mod commands;
mod output;

use clap::Parser;
use clap_complete::generate;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::PullZone { action } => {
            commands::pull_zone::handle(action, cli.format, cli.debug, cli.yes).await
        }
        Commands::StorageZone { action } => {
            commands::storage_zone::handle(action, cli.format, cli.debug, cli.yes).await
        }
        Commands::Storage { action } => {
            commands::storage::handle(action, cli.format, cli.debug, cli.yes).await
        }
        Commands::Dns { action } => commands::dns::handle(action, cli.format),
        Commands::Stream { action } => commands::stream::handle(action, cli.format),
        Commands::Shield { action } => commands::shield::handle(action, cli.format),
        Commands::Script { action } => commands::script::handle(action, cli.format),
        Commands::Container { action } => commands::container::handle(action, cli.format),
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
