use crate::cli::{OutputFormat, StreamAction};
use anyhow::Result;

pub fn handle(action: &StreamAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        StreamAction::Library { .. } => "stream library ...",
        StreamAction::Video { .. } => "stream video ...",
    };
    eprintln!("Not implemented yet: {cmd}");
    std::process::exit(2);
}
