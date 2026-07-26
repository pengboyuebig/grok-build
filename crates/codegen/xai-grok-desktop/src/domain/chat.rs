use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrontendEventKind {
    AssistantDelta,
    AssistantFinal,
    ToolStarted,
    ToolFinished,
    FileChanged,
    TerminalOutput,
    ApprovalRequested,
    Error,
    SessionChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FrontendEvent {
    pub kind: FrontendEventKind,
    pub text: Option<String>,
    pub approval_id: Option<String>,
    pub approved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestAgentEvent {
    TextDelta(String),
    ApprovalRequested { id: String },
}

pub trait DesktopAgentClient: Send + Sync {
    fn send_message(&self, session_id: &str, message: &str) -> anyhow::Result<()>;
    fn respond_to_approval(&self, approval_id: &str, approved: bool) -> anyhow::Result<()>;
}
