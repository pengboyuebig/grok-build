use crate::domain::terminal_launch::LaunchRequest;
use crate::services::terminal_launcher::{
    build_launch_spec, launch_terminal_session as spawn_terminal_session,
};

#[tauri::command]
pub fn launch_terminal_session(request: LaunchRequest) -> Result<(), String> {
    let spec = build_launch_spec(request).map_err(|err| err.to_string())?;
    spawn_terminal_session(&spec).map_err(|err| err.to_string())
}

#[tauri::command]
pub fn open_workspace_dialog() -> Result<String, String> {
    Ok("C:/work/demo".to_string())
}
