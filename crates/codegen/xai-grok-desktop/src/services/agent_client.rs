use crate::domain::chat::{FrontendEvent, FrontendEventKind, TestAgentEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    AllowOnce,
    Cancel,
}

pub fn approval_decision(
    approved: bool,
    has_allow_once: bool,
    has_allow_always: bool,
) -> ApprovalDecision {
    let _ = has_allow_always;
    if approved && has_allow_once {
        ApprovalDecision::AllowOnce
    } else {
        ApprovalDecision::Cancel
    }
}

pub fn map_agent_event(event: TestAgentEvent) -> FrontendEvent {
    match event {
        TestAgentEvent::TextDelta(text) => FrontendEvent {
            kind: FrontendEventKind::AssistantDelta,
            text: Some(text),
            approval_id: None,
            approved: false,
        },
        TestAgentEvent::ApprovalRequested { id } => FrontendEvent {
            kind: FrontendEventKind::ApprovalRequested,
            text: None,
            approval_id: Some(id),
            approved: false,
        },
    }
}
