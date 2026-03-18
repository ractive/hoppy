use std::path::PathBuf;

use clap::CommandFactory;
use clap_mangen::Man;
use hoppy::cli::Cli;

fn main() -> anyhow::Result<()> {
    let output_dir = parse_output_dir();

    std::fs::create_dir_all(&output_dir)?;

    let cmd = Cli::command();
    generate_man_pages(&cmd, &[], &output_dir)?;

    println!("Man pages written to {}", output_dir.display());
    Ok(())
}

/// Parses `--output-dir <path>` from argv, defaulting to `target/man`.
fn parse_output_dir() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--output-dir" {
            if let Some(val) = args.next() {
                return PathBuf::from(val);
            }
        } else if let Some(val) = arg.strip_prefix("--output-dir=") {
            return PathBuf::from(val);
        }
    }
    // Default: workspace root's target/man. Walk up from the manifest dir.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap_or(&manifest_dir)
        .join("target")
        .join("man")
}

/// Recursively generates man pages for `cmd` and all its subcommands.
///
/// `parents` holds the ancestor command names (e.g. `["hoppy", "pull-zone"]`)
/// so that nested pages are named correctly (`hoppy-pull-zone-list.1`).
fn generate_man_pages(
    cmd: &clap::Command,
    parents: &[&str],
    output_dir: &PathBuf,
) -> anyhow::Result<()> {
    // Build the full hyphen-separated name: hoppy, hoppy-pull-zone, …
    let name = if parents.is_empty() {
        cmd.get_name().to_owned()
    } else {
        format!("{}-{}", parents.join("-"), cmd.get_name())
    };

    let filename = format!("{name}.1");
    let path = output_dir.join(&filename);

    // clap_mangen expects the command's display name to match the man page name.
    // `Command::name` requires `&'static str`, so we leak the String to satisfy
    // the lifetime. This is acceptable in a short-lived xtask binary.
    let static_name: &'static str = Box::leak(name.into_boxed_str());
    let page_cmd = cmd.clone().name(static_name);

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
