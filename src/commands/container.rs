use crate::cli::{ContainerAction, OutputFormat};
use anyhow::Result;

pub fn handle(action: &ContainerAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        ContainerAction::List => "container list",
        ContainerAction::Get { id } => return stub(&format!("container get --id {id}")),
        ContainerAction::Create { name } => {
            return stub(&format!("container create --name {name}"));
        }
        ContainerAction::Delete { id } => return stub(&format!("container delete --id {id}")),
        ContainerAction::Logs { id } => return stub(&format!("container logs --id {id}")),
    };
    stub(cmd)
}

fn stub(command: &str) -> Result<()> {
    eprintln!("Not implemented yet: {command}");
    std::process::exit(2);
}
