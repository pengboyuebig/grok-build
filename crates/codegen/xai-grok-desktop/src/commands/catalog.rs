use serde::Serialize;

use crate::domain::command_catalog::DesktopCommand;

#[derive(Serialize)]
pub struct CommandCatalog {
    pub commands: Vec<DesktopCommand>,
}

#[cfg_attr(feature = "tauri-runtime", tauri::command)]
pub fn get_command_catalog() -> CommandCatalog {
    command_catalog()
}

pub fn command_catalog() -> CommandCatalog {
    CommandCatalog {
        commands: vec![
            DesktopCommand::from_slash("/rename", true, true),
            DesktopCommand::from_slash("/model", true, true),
            DesktopCommand::from_slash("/effort", true, true),
            DesktopCommand::from_slash("/new", false, false),
            DesktopCommand::from_slash("/clear", false, false),
            DesktopCommand::from_slash("/quit", false, false),
        ],
    }
}
