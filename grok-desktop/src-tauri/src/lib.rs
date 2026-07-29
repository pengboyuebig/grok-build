mod runtime;

use runtime::{AgentRuntime, RuntimeConnection};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;

struct AppState(Mutex<AgentRuntime>);

#[tauri::command]
async fn select_workspace(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let path = app.dialog().file().blocking_pick_folder();
    Ok(path.map(|value| value.to_string()))
}

#[tauri::command]
fn start_agent(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<RuntimeConnection, String> {
    let workspace = std::path::PathBuf::from(&workspace_path);
    if !workspace.is_dir() {
        return Err("所选路径不是有效的工作目录。".into());
    }
    let resource = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("resources")
        .join("grok.exe");
    let mut runtime = state.0.lock().map_err(|_| "运行时状态不可用。")?;
    runtime.start(resource, workspace)
}

#[tauri::command]
fn stop_agent(state: State<'_, AppState>) -> Result<(), String> {
    state.0.lock().map_err(|_| "运行时状态不可用。")?.stop();
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(AgentRuntime::default())))
        .invoke_handler(tauri::generate_handler![
            select_workspace,
            start_agent,
            stop_agent
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Grok Desktop");
}
