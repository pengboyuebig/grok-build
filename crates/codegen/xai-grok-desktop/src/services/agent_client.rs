use crate::domain::chat::{FrontendEvent, FrontendEventKind, TestAgentEvent};

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
