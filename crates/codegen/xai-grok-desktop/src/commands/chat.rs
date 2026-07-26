use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tokio::sync::Mutex;

use crate::services::live_agent::LiveAgent;

#[derive(Default)]
pub struct DesktopSessionState {
    agent: Mutex<Option<Arc<LiveAgent>>>,
    sessions: Mutex<Vec<String>>,
}

impl DesktopSessionState {
    async fn agent(&self, app: AppHandle) -> Result<Arc<LiveAgent>, String> {
        let mut current = self.agent.lock().await;
        if let Some(agent) = current.as_ref() {
            return Ok(Arc::clone(agent));
        }
        let agent = LiveAgent::connect(app)
            .await
            .map_err(|error| error.to_string())?;
        *current = Some(Arc::clone(&agent));
        Ok(agent)
    }
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    state: State<'_, DesktopSessionState>,
    cwd: String,
) -> Result<String, String> {
    let cwd = PathBuf::from(cwd);
    if !cwd.is_dir() {
        return Err("workspace directory does not exist".to_string());
    }
    let agent = state.agent(app).await?;
    let session_id = agent
        .start_session(cwd)
        .await
        .map_err(|error| error.to_string())?;
    state.sessions.lock().await.push(session_id.clone());
    Ok(session_id)
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: State<'_, DesktopSessionState>,
    session_id: String,
    message: String,
) -> Result<(), String> {
    if session_id.trim().is_empty() {
        return Err("session id cannot be empty".to_string());
    }
    if message.trim().is_empty() {
        return Err("message cannot be empty".to_string());
    }
    let agent = state.agent(app).await?;
    agent
        .send_message(session_id, message)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn respond_to_approval(
    app: AppHandle,
    state: State<'_, DesktopSessionState>,
    approval_id: String,
    approved: bool,
) -> Result<(), String> {
    if approval_id.trim().is_empty() {
        return Err("approval id cannot be empty".to_string());
    }
    let agent = state.agent(app).await?;
    agent
        .respond_to_approval(approval_id, approved)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, DesktopSessionState>) -> Result<Vec<String>, String> {
    Ok(state.sessions.lock().await.clone())
}

#[tauri::command]
pub fn set_preference(key: String, value: String) -> Result<(), String> {
    let _ = (key, value);
    Ok(())
}
