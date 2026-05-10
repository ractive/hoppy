use std::path::{Path, PathBuf};

use clap::CommandFactory;
use clap_mangen::Man;
use hoppy_cli::cli::Cli;

fn main() -> anyhow::Result<()> {
    let output_dir = parse_output_dir();

    std::fs::create_dir_all(&output_dir)?;

    let cmd = Cli::command();
    generate_man_pages(&cmd, &[], &output_dir)?;

    println!("Man pages written to {}", output_dir.display());
    Ok(())
}

/// Parses `--output-dir <path>` from argv, defaulting to `target/man`.
///
/// Prints an error and exits if any unrecognized arguments are present.
fn parse_output_dir() -> PathBuf {
    let mut args = std::env::args().skip(1);
    let mut output_dir: Option<PathBuf> = None;
    let mut unknown: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "--output-dir" {
            match args.next() {
                Some(val) => output_dir = Some(PathBuf::from(val)),
                None => {
                    eprintln!("error: --output-dir requires a value");
                    eprintln!("usage: cargo xtask [--output-dir <path>]");
                    std::process::exit(1);
                }
            }
        } else if let Some(val) = arg.strip_prefix("--output-dir=") {
            output_dir = Some(PathBuf::from(val));
        } else {
            unknown.push(arg);
        }
    }

    if !unknown.is_empty() {
        eprintln!("error: unrecognized argument(s): {}", unknown.join(", "));
        eprintln!("usage: cargo xtask [--output-dir <path>]");
        std::process::exit(1);
    }

    output_dir.unwrap_or_else(|| {
        // Default: workspace root's target/man. Walk up from the manifest dir.
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .unwrap_or(&manifest_dir)
            .join("target")
            .join("man")
    })
}

/// Recursively generates man pages for `cmd` and all its subcommands.
///
/// `parents` holds the ancestor command names (e.g. `["hoppy", "pull-zone"]`)
/// so that nested pages are named correctly (`hoppy-pull-zone-list.1`).
fn generate_man_pages(
    cmd: &clap::Command,
    parents: &[&str],
    output_dir: &Path,
) -> anyhow::Result<()> {
    // Build the full hyphen-separated name: hoppy, hoppy-pull-zone, …
    let name = if parents.is_empty() {
        cmd.get_name().to_owned()
    } else {
        format!("{}-{}", parents.join("-"), cmd.get_name())
    };

    let filename = format!("{name}.1");
    let path = output_dir.join(&filename);

    // clap_mangen renders the man page title from get_display_name(), falling back
    // to get_name(). Command::display_name accepts impl IntoResettable<String> so
    // an owned String works directly — no &'static str or Box::leak needed.
    let page_cmd = cmd.clone().display_name(name);

    let man = Man::new(page_cmd.clone());
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    std::fs::write(&path, &buf)?;
    println!("  {filename}");

    // Recurse into subcommands.
    let mut next_parents: Vec<&str> = parents.to_vec();
    next_parents.push(cmd.get_name());

    for sub in page_cmd.get_subcommands() {
        // Skip the built-in `help` subcommand.
        if sub.get_name() == "help" {
            continue;
        }
        generate_man_pages(sub, &next_parents, output_dir)?;
    }

    Ok(())
}
