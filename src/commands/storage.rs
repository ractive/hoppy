use crate::cli::{OutputFormat, StorageAction};
use anyhow::Result;

pub fn handle(action: &StorageAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        StorageAction::Upload { zone, .. } => format!("storage upload --zone {zone}"),
        StorageAction::Download { zone, .. } => format!("storage download --zone {zone}"),
        StorageAction::Ls { zone, path } => format!("storage ls --zone {zone} --path {path}"),
        StorageAction::Rm { zone, .. } => format!("storage rm --zone {zone}"),
    };
    eprintln!("Not implemented yet: {cmd}");
    std::process::exit(2);
}
