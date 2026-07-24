#[cfg(feature = "tauri-runtime")]
pub mod catalog;
#[cfg(feature = "tauri-runtime")]
pub mod chat;
#[cfg(feature = "tauri-runtime")]
pub mod terminal;

pub const DESKTOP_INVOKE_COMMANDS: [&str; 8] = [
    "get_command_catalog",
    "start_session",
    "send_message",
    "respond_to_approval",
    "launch_terminal_session",
    "list_sessions",
    "open_workspace_dialog",
    "set_preference",
];

pub fn desktop_invoke_schema() -> &'static [&'static str; 8] {
    &DESKTOP_INVOKE_COMMANDS
}
