use crate::cli::{OutputFormat, StorageZoneAction};
use anyhow::Result;

pub fn handle(action: &StorageZoneAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        StorageZoneAction::List => "storage-zone list",
        StorageZoneAction::Get { id } => return stub(&format!("storage-zone get --id {id}")),
        StorageZoneAction::Create { name } => {
            return stub(&format!("storage-zone create --name {name}"));
        }
        StorageZoneAction::Update { id } => return stub(&format!("storage-zone update --id {id}")),
        StorageZoneAction::Delete { id } => return stub(&format!("storage-zone delete --id {id}")),
    };
    stub(cmd)
}

fn stub(command: &str) -> Result<()> {
    eprintln!("Not implemented yet: {command}");
    std::process::exit(2);
}
