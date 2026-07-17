---
title: Modern Rust CLI Best Practices with Clap
date: 2026-03-17
tags:
  - rust
  - clap
  - cli
  - clap_complete
  - best-practices
status: completed
type: research
---

# Modern Rust CLI Best Practices with Clap

## Clap Version

Current stable version: **clap 4.6.0** (as of March 2026). Licensed under MIT OR Apache-2.0.

## Derive API (Recommended)

The derive API is the recommended approach for most CLI applications. It uses Rust's derive macros for a declarative style:

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "hoppy", version, about = "CLI for bunny.net services")]
struct Cli {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    format: OutputFormat,

    /// API key (overrides BUNNY_API_KEY env var)
    #[arg(long, env = "BUNNY_API_KEY", global = true, hide_env_values = true)]
    api_key: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage pull zones
    PullZone {
        #[command(subcommand)]
        action: PullZoneAction,
    },
    /// Manage storage zones
    StorageZone {
        #[command(subcommand)]
        action: StorageZoneAction,
    },
}

#[derive(Copy, Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
    Text,
}
```

## Nested Subcommands Pattern

For `hoppy pull-zone create --name=foo`, use two levels of subcommands:

```rust
#[derive(Subcommand)]
enum PullZoneAction {
    /// List all pull zones
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        per_page: Option<u32>,
    },
    /// Get a specific pull zone
    Get {
        #[arg(long)]
        id: u64,
    },
    /// Create a new pull zone
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        origin_url: String,
    },
    /// Delete a pull zone
    Delete {
        #[arg(long)]
        id: u64,
    },
}
```

## Global Arguments

Use `#[arg(global = true)]` for flags that apply everywhere (format, api-key, verbose):

```rust
#[derive(Parser)]
struct Cli {
    #[arg(long, global = true)]
    verbose: bool,

    #[arg(long, value_enum, default_value_t = OutputFormat::Table, global = true)]
    format: OutputFormat,
}
```

## clap_complete for Shell Completions

Add a hidden `completions` subcommand:

```rust
use clap_complete::{generate, Shell};

#[derive(Subcommand)]
enum Commands {
    /// Generate shell completions
    #[command(hide = true)]
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    // ... other commands
}

// In handler:
fn print_completions(shell: Shell, cmd: &mut clap::Command) {
    generate(shell, cmd, "hoppy", &mut std::io::stdout());
}
```

## Output Formatting

For machine-readable output, implement a trait-based formatting system:

```rust
use serde::Serialize;

fn output<T: Serialize + Tabled>(data: &T, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(data).unwrap()),
        OutputFormat::Table => println!("{}", Table::new(vec![data]).to_string()),
        OutputFormat::Text => { /* tab-separated values */ },
    }
}
```

Consider using `tabled` crate for table formatting and `serde_json` for JSON output.

## Error Handling

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let cli = Cli::parse();
    // ...
    Ok(())
}
```

For user-facing errors, format them nicely rather than showing Rust debug output.

## Project Structure

```
src/
  main.rs          # Entry point, CLI parsing
  cli.rs           # CLI struct definitions
  commands/
    mod.rs
    pull_zone.rs   # Pull zone command handlers
    storage_zone.rs
    dns.rs
    ...
  client/          # Generated or hand-written API client
    mod.rs
  output.rs        # Output formatting (json, table, text)
  error.rs         # Error types
```

## Cross-Platform GitHub Actions

Use a matrix strategy for building binaries:

```yaml
strategy:
  matrix:
    include:
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
      - target: aarch64-unknown-linux-gnu
        os: ubuntu-latest
      - target: x86_64-apple-darwin
        os: macos-latest
      - target: aarch64-apple-darwin
        os: macos-latest
      - target: x86_64-pc-windows-msvc
        os: windows-latest
```

Key tools:
- `cross` for Linux cross-compilation (uses Docker)
- `houseabsolute/actions-rust-cross` GitHub Action
- `taiki-e/upload-rust-binary-action` for uploading to releases
- `softprops/action-gh-release` for creating releases

Trigger on version tags: `on: push: tags: ['v*']`

## Key Dependencies

```toml
[dependencies]
clap = { version = "4.6", features = ["derive", "env"] }
clap_complete = "4.6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tabled = "0.17"
```

## Sources

- [Clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [Rain's Rust CLI Recommendations](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html)
- [How to Build CLI Applications with Clap in Rust (2026)](https://oneuptime.com/blog/post/2026-02-03-rust-clap-cli-applications/view)
- [Deploy Rust Binaries with GitHub Actions](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)
- [Cross-Platform Rust CI/CD Pipeline](https://ahmedjama.com/blog/2025/12/cross-platform-rust-pipeline-github-actions/)
- [actions-rust-cross](https://github.com/houseabsolute/actions-rust-cross)
- [upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action)

## Related

- [[research/cli-design-patterns]] — CLI patterns from cloud CLIs
- [[Seed]] — project brief
