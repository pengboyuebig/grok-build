use tauri::State;

use crate::services::session_service::SessionService;

pub type DesktopSessionState = SessionService;

#[tauri::command]
pub async fn start_session(
    state: State<'_, std::sync::Arc<DesktopSessionState>>,
    cwd: String,
) -> Result<String, String> {
    state
        .start_session(cwd.into())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, std::sync::Arc<DesktopSessionState>>,
    session_id: String,
    message: String,
) -> Result<(), String> {
    state
        .send_message(session_id, message)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn respond_to_approval(
    state: State<'_, std::sync::Arc<DesktopSessionState>>,
    approval_id: String,
    approved: bool,
) -> Result<(), String> {
    state
        .respond_to_approval(approval_id, approved)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_sessions(
    state: State<'_, std::sync::Arc<DesktopSessionState>>,
) -> Result<Vec<String>, String> {
    Ok(state.list_sessions().await)
}

#[tauri::command]
pub fn set_preference(key: String, value: String) -> Result<(), String> {
    let _ = (key, value);
    Ok(())
}
