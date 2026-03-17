use crate::cli::{OutputFormat, ScriptAction};
use anyhow::Result;

pub fn handle(action: &ScriptAction, _format: OutputFormat) -> Result<()> {
    let cmd = match action {
        ScriptAction::List => "script list",
        ScriptAction::Get { id } => return stub(&format!("script get --id {id}")),
        ScriptAction::Create { name } => return stub(&format!("script create --name {name}")),
        ScriptAction::Delete { id } => return stub(&format!("script delete --id {id}")),
        ScriptAction::Deploy { id } => return stub(&format!("script deploy --id {id}")),
    };
    stub(cmd)
}

fn stub(command: &str) -> Result<()> {
    eprintln!("Not implemented yet: {command}");
    std::process::exit(2);
}
