use xai_grok_desktop::domain::chat::{FrontendEventKind, TestAgentEvent};
use xai_grok_desktop::services::agent_client::map_agent_event;

#[test]
fn maps_text_delta_to_frontend_event() {
    let event = map_agent_event(TestAgentEvent::TextDelta("hello".into()));

    assert_eq!(event.kind, FrontendEventKind::AssistantDelta);
    assert_eq!(event.text.as_deref(), Some("hello"));
}

#[test]
fn approval_starts_pending() {
    let event = map_agent_event(TestAgentEvent::ApprovalRequested { id: "a1".into() });

    assert_eq!(event.kind, FrontendEventKind::ApprovalRequested);
    assert!(!event.approved);
}
