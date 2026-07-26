use xai_grok_desktop::services::agent_client::{ApprovalDecision, approval_decision};

#[test]
fn approved_request_selects_allow_once() {
    assert_eq!(
        approval_decision(true, true, true),
        ApprovalDecision::AllowOnce
    );
}

#[test]
fn approved_request_without_allow_option_is_cancelled() {
    assert_eq!(
        approval_decision(true, false, false),
        ApprovalDecision::Cancel
    );
}

#[test]
fn rejected_request_is_always_cancelled() {
    assert_eq!(
        approval_decision(false, true, true),
        ApprovalDecision::Cancel
    );
}
