#[tauri::command]
pub fn start_session() -> Result<String, String> {
    Ok("s1".to_string())
}

#[tauri::command]
pub fn send_message(session_id: String, message: String) -> Result<(), String> {
    if session_id.trim().is_empty() {
        return Err("会话标识不能为空".to_string());
    }
    if message.trim().is_empty() {
        return Err("消息不能为空".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn respond_to_approval(approval_id: String, approved: bool) -> Result<(), String> {
    if approval_id.trim().is_empty() {
        return Err("审批标识不能为空".to_string());
    }
    let _ = approved;
    Ok(())
}

#[tauri::command]
pub fn list_sessions() -> Result<Vec<String>, String> {
    Ok(vec!["s1".to_string()])
}

#[tauri::command]
pub fn set_preference(key: String, value: String) -> Result<(), String> {
    let _ = (key, value);
    Ok(())
}
