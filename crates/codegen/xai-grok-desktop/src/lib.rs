pub mod commands;
pub mod domain;
pub mod services;
pub mod web;

pub const PRODUCT_NAME: &str = "Grok Desktop";
pub const BINARY_NAME: &str = "grok-desktop";

#[cfg(feature = "tauri-runtime")]
pub fn run() -> anyhow::Result<()> {
    use std::sync::Arc;

    use tauri::Emitter;

    let sessions = Arc::new(commands::chat::DesktopSessionState::default());
    let event_sessions = Arc::clone(&sessions);
    tauri::Builder::default()
        .manage(sessions)
        .setup(move |app| {
            let handle = app.handle().clone();
            let mut events = event_sessions.subscribe_events();
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = events.recv().await {
                    let _ = handle.emit("chat:event", event);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::catalog::get_command_catalog,
            commands::chat::start_session,
            commands::chat::send_message,
            commands::chat::respond_to_approval,
            commands::chat::list_sessions,
            commands::chat::set_preference,
            commands::terminal::launch_terminal_session,
            commands::terminal::open_workspace_dialog,
        ])
        .run(tauri::generate_context!())
        .map_err(|err| anyhow::anyhow!(err))
}

#[cfg(not(feature = "tauri-runtime"))]
pub fn run() -> anyhow::Result<()> {
    use crate::domain::terminal_launch::{LaunchRequest, PermissionMode};
    use crate::services::terminal_launcher::{build_launch_spec, launch_terminal_session};

    let spec = build_launch_spec(LaunchRequest::new(
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        None,
        None,
        PermissionMode::Ask,
    ))?;
    launch_terminal_session(&spec)
}
