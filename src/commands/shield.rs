use crate::cli::{OutputFormat, ShieldAction};
use anyhow::Result;

pub fn handle(action: &ShieldAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        ShieldAction::Waf => "shield waf",
        ShieldAction::RateLimit => "shield rate-limit",
        ShieldAction::Ddos => "shield ddos",
    };
    eprintln!("Not implemented yet: {cmd}");
    std::process::exit(2);
}
