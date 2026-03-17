use crate::cli::{OutputFormat, PullZoneAction};
use anyhow::Result;

pub fn handle(action: &PullZoneAction, _format: OutputFormat) -> Result<()> {
    let label = match action {
        PullZoneAction::List => "pull-zone list",
        PullZoneAction::Get { id } => return not_implemented(&format!("pull-zone get --id {id}")),
        PullZoneAction::Create { name, .. } => {
            return not_implemented(&format!("pull-zone create --name {name}"));
        }
        PullZoneAction::Update { id } => {
            return not_implemented(&format!("pull-zone update --id {id}"));
        }
        PullZoneAction::Delete { id } => {
            return not_implemented(&format!("pull-zone delete --id {id}"));
        }
        PullZoneAction::Purge { id } => {
            return not_implemented(&format!("pull-zone purge --id {id}"));
        }
    };
    not_implemented(label)
}

fn not_implemented(command: &str) -> Result<()> {
    eprintln!("Not implemented yet: {command}");
    std::process::exit(2);
}
