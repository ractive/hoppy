use crate::cli::{DnsAction, OutputFormat};
use anyhow::Result;

pub fn handle(action: &DnsAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        DnsAction::Zone { action } => format!("dns zone {}", zone_label(action)),
        DnsAction::Record { action } => format!("dns record {}", record_label(action)),
    };
    eprintln!("Not implemented yet: {cmd}");
    std::process::exit(2);
}

fn zone_label(action: &crate::cli::DnsZoneAction) -> &'static str {
    use crate::cli::DnsZoneAction::*;
    match action {
        List => "list",
        Get { .. } => "get",
        Create { .. } => "create",
        Update { .. } => "update",
        Delete { .. } => "delete",
    }
}

fn record_label(action: &crate::cli::DnsRecordAction) -> &'static str {
    use crate::cli::DnsRecordAction::*;
    match action {
        List { .. } => "list",
        Add { .. } => "add",
        Update { .. } => "update",
        Delete { .. } => "delete",
    }
}
