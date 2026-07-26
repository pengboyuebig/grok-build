use xai_grok_desktop::web::auth::LocalAuth;

#[test]
fn rejects_missing_token_and_non_loopback_origin() {
    let auth = LocalAuth::new_for_test("test-token", 43123);

    assert!(!auth.authorizes(None, Some("http://127.0.0.1:43123")));
    assert!(!auth.authorizes(Some("test-token"), Some("http://localhost:43123")));
}

#[test]
fn accepts_matching_token_and_loopback_origin() {
    let auth = LocalAuth::new_for_test("test-token", 43123);

    assert!(auth.authorizes(Some("test-token"), Some("http://127.0.0.1:43123")));
}
