use std::path::{Path, PathBuf};
use std::process::Command;

use clap::CommandFactory;
use clap_mangen::Man;
use hoppy_cli::cli::Cli;

fn main() -> anyhow::Result<()> {
    let first = std::env::args().nth(1);
    match first.as_deref() {
        Some("check-iteration-ready") => {
            let rest: Vec<String> = std::env::args().skip(2).collect();
            check_iteration_ready(&rest)
        }
        _ => {
            let output_dir = parse_output_dir();
            generate_man_pages_at(&output_dir)
        }
    }
}

fn generate_man_pages_at(output_dir: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(output_dir)?;
    let cmd = Cli::command();
    generate_man_pages(&cmd, &[], output_dir)?;
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
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .unwrap_or(&manifest_dir)
            .join("target")
            .join("man")
    })
}

/// Recursively generates man pages for `cmd` and all its subcommands.
fn generate_man_pages(
    cmd: &clap::Command,
    parents: &[&str],
    output_dir: &Path,
) -> anyhow::Result<()> {
    let name = if parents.is_empty() {
        cmd.get_name().to_owned()
    } else {
        format!("{}-{}", parents.join("-"), cmd.get_name())
    };

    let filename = format!("{name}.1");
    let path = output_dir.join(&filename);
    let page_cmd = cmd.clone().display_name(name);

    let man = Man::new(page_cmd.clone());
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    std::fs::write(&path, &buf)?;
    println!("  {filename}");

    let mut next_parents: Vec<&str> = parents.to_vec();
    next_parents.push(cmd.get_name());

    for sub in page_cmd.get_subcommands() {
        if sub.get_name() == "help" {
            continue;
        }
        generate_man_pages(sub, &next_parents, output_dir)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// check-iteration-ready
// ---------------------------------------------------------------------------
//
// Lightweight pre-PR gate. Verifies:
//   1. The plan file exists.
//   2. The branch has commits ahead of `--base` (default `origin/main`).
//   3. `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`
//      all succeed in the workspace.
//
// Intended to be cheap to re-run and to fail loudly on the same gates a human
// reviewer would care about. Not a substitute for code review.

fn check_iteration_ready(args: &[String]) -> anyhow::Result<()> {
    let mut plan: Option<PathBuf> = None;
    let mut base = String::from("origin/main");

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--plan" => {
                plan = it.next().map(PathBuf::from);
            }
            s if s.starts_with("--plan=") => {
                plan = Some(PathBuf::from(&s["--plan=".len()..]));
            }
            "--base" => {
                if let Some(v) = it.next() {
                    base = v.clone();
                }
            }
            s if s.starts_with("--base=") => {
                base = s["--base=".len()..].to_string();
            }
            other => {
                eprintln!("error: unrecognised arg: {other}");
                std::process::exit(2);
            }
        }
    }

    let plan = plan.ok_or_else(|| anyhow::anyhow!("--plan <path> is required"))?;
    let mut failures: Vec<String> = Vec::new();

    if !plan.exists() {
        failures.push(format!("plan file not found: {}", plan.display()));
    } else {
        println!("✓ plan file exists: {}", plan.display());
    }

    match Command::new("git")
        .args(["rev-list", "--count", &format!("{base}..HEAD")])
        .output()
    {
        Ok(out) if out.status.success() => {
            let n: u64 = String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse()
                .unwrap_or(0);
            if n == 0 {
                failures.push(format!("no commits ahead of {base}"));
            } else {
                println!("✓ {n} commit(s) ahead of {base}");
            }
        }
        Ok(out) => failures.push(format!(
            "git rev-list failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) => failures.push(format!("git rev-list error: {e}")),
    }

    run_gate(
        "cargo fmt --check",
        &["fmt", "--all", "--", "--check"],
        &mut failures,
    );
    run_gate(
        "cargo clippy -D warnings",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &mut failures,
    );
    run_gate(
        "cargo test --workspace --quiet",
        &["test", "--workspace", "--quiet"],
        &mut failures,
    );

    if failures.is_empty() {
        println!("\nAll iteration-ready checks passed.");
        Ok(())
    } else {
        eprintln!("\nIteration-ready checks failed:");
        for f in &failures {
            eprintln!("  ✗ {f}");
        }
        std::process::exit(1);
    }
}

fn run_gate(label: &str, args: &[&str], failures: &mut Vec<String>) {
    println!("→ running: cargo {}", args.join(" "));
    let status = Command::new("cargo").args(args).status();
    match status {
        Ok(s) if s.success() => println!("✓ {label}"),
        Ok(s) => failures.push(format!("{label} exited with status {s}")),
        Err(e) => failures.push(format!("{label} failed to start: {e}")),
    }
}
