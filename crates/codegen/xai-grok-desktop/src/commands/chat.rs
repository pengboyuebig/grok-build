use anyhow::{Result, bail};

pub fn validate_message(session_id: &str, message: &str) -> Result<()> {
    if session_id.trim().is_empty() {
        bail!("会话标识不能为空");
    }
    if message.trim().is_empty() {
        bail!("消息不能为空");
    }
    Ok(())
}

pub fn validate_approval_response(approval_id: &str) -> Result<()> {
    if approval_id.trim().is_empty() {
        bail!("审批标识不能为空");
    }
    Ok(())
}
