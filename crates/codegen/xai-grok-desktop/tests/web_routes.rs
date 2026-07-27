#![cfg(feature = "web-runtime")]

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use tower::ServiceExt;
use xai_grok_desktop::web::{auth::LocalAuth, routes::router};

#[tokio::test]
async fn commands_require_the_local_token_and_origin() {
    let app = router(LocalAuth::new_for_test("test-token", 43123));
    let request = Request::get("/api/commands").body(Body::empty()).unwrap();
    assert_eq!(
        app.clone().oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    let request = Request::get("/api/commands")
        .header("x-grok-local-token", "test-token")
        .header(header::ORIGIN, "http://localhost:43123")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(request).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    let request = Request::get("/api/commands")
        .header("x-grok-local-token", "test-token")
        .header(header::ORIGIN, "http://127.0.0.1:43123")
        .body(Body::empty())
        .unwrap();
    assert_eq!(app.oneshot(request).await.unwrap().status(), StatusCode::OK);
}

#[tokio::test]
async fn write_routes_reject_missing_tokens_before_processing_the_payload() {
    let app = router(LocalAuth::new_for_test("test-token", 43123));
    let request = Request::post("/api/sessions")
        .header(header::ORIGIN, "http://127.0.0.1:43123")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("not-json"))
        .unwrap();

    assert_eq!(
        app.oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn websocket_rejects_an_invalid_local_protocol() {
    let app = router(LocalAuth::new_for_test("test-token", 43123));
    let request = Request::get("/api/events")
        .header(header::ORIGIN, "http://127.0.0.1:43123")
        .header(header::CONNECTION, "upgrade")
        .header(header::UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
        .header(header::SEC_WEBSOCKET_PROTOCOL, "grok-local.wrong-token")
        .body(Body::empty())
        .unwrap();

    assert_eq!(
        app.oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn commands_match_the_shared_catalog_response_contract() {
    let app = router(LocalAuth::new_for_test("test-token", 43123));
    let request = Request::get("/api/commands")
        .header("x-grok-local-token", "test-token")
        .header(header::ORIGIN, "http://127.0.0.1:43123")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let catalog: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(catalog["commands"].is_array());
}

#[tokio::test]
async fn public_static_fallback_is_not_blocked_by_api_authentication() {
    let app = router(LocalAuth::new_for_test("test-token", 43123));
    let request = Request::get("/").body(Body::empty()).unwrap();

    assert_ne!(
        app.oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}
