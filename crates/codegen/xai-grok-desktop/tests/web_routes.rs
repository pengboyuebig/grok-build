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
