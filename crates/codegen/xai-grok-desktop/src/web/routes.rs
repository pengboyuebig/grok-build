use axum::{
    Json, Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    commands::catalog::CommandCatalog,
    domain::terminal_launch::LaunchRequest,
    services::{
        session_service::SessionService,
        terminal_launcher::{build_launch_spec, launch_terminal_session},
    },
    web::auth::LocalAuth,
};

const TOKEN_HEADER: &str = "x-grok-local-token";

#[derive(Clone)]
struct WebState {
    auth: LocalAuth,
    sessions: Arc<SessionService>,
}

#[derive(Deserialize)]
struct SessionRequest {
    cwd: String,
}

#[derive(Deserialize)]
struct MessageRequest {
    message: String,
}

#[derive(Deserialize)]
struct ApprovalRequest {
    approved: bool,
}

#[derive(Deserialize)]
struct TerminalRequest {
    cwd: std::path::PathBuf,
    model: Option<String>,
    effort: Option<String>,
    permission_mode: crate::domain::terminal_launch::PermissionMode,
}

pub fn router(auth: LocalAuth) -> Router {
    router_with_sessions(auth, Arc::new(SessionService::default()))
}

pub fn router_with_sessions(auth: LocalAuth, sessions: Arc<SessionService>) -> Router {
    let middleware_auth = auth.clone();
    let api = Router::new()
        .route("/health", get(health))
        .route("/api/commands", get(command_catalog))
        .route("/api/sessions", post(start_session))
        .route("/api/sessions/{session_id}/messages", post(send_message))
        .route("/api/approvals/{approval_id}", post(respond_to_approval))
        .route("/api/terminal-sessions", post(start_terminal_session))
        .route("/api/events", get(events))
        .with_state(WebState { auth, sessions })
        // Reject unauthorized calls before Axum parses a request body or WebSocket upgrade.
        .route_layer(middleware::from_fn_with_state(
            middleware_auth,
            authorize_request,
        ));

    Router::new().merge(api)
}

async fn authorize_request(
    State(auth): State<LocalAuth>,
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().path() == "/api/events" {
        authorize_websocket(&auth, request.headers())?;
    } else {
        authorize(&auth, request.headers())?;
    }
    Ok(next.run(request).await)
}

async fn health(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Result<(StatusCode, &'static str), StatusCode> {
    authorize(&state.auth, &headers)?;
    Ok((StatusCode::OK, "ok"))
}

async fn command_catalog(
    State(state): State<WebState>,
    headers: HeaderMap,
) -> Result<axum::Json<CommandCatalog>, StatusCode> {
    authorize(&state.auth, &headers)?;
    Ok(axum::Json(crate::commands::catalog::command_catalog()))
}

fn authorize(auth: &LocalAuth, headers: &HeaderMap) -> Result<(), StatusCode> {
    let token = headers
        .get(TOKEN_HEADER)
        .and_then(|value| value.to_str().ok());
    if token != Some(auth.token()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if origin != Some(auth.origin()) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

async fn start_session(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(request): Json<SessionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    authorize(&state.auth, &headers)?;
    if request.cwd.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .sessions
        .start_session(request.cwd.into())
        .await
        .map(|session_id| Json(serde_json::json!({ "session_id": session_id })))
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn send_message(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(session_id): Path<String>,
    Json(request): Json<MessageRequest>,
) -> Result<StatusCode, StatusCode> {
    authorize(&state.auth, &headers)?;
    if request.message.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    state
        .sessions
        .send_message(session_id, request.message)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn respond_to_approval(
    State(state): State<WebState>,
    headers: HeaderMap,
    Path(approval_id): Path<String>,
    Json(request): Json<ApprovalRequest>,
) -> Result<StatusCode, StatusCode> {
    authorize(&state.auth, &headers)?;
    state
        .sessions
        .respond_to_approval(approval_id, request.approved)
        .await
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::BAD_REQUEST)
}

async fn start_terminal_session(
    State(state): State<WebState>,
    headers: HeaderMap,
    Json(request): Json<TerminalRequest>,
) -> Result<StatusCode, StatusCode> {
    authorize(&state.auth, &headers)?;
    let launch = LaunchRequest {
        cwd: request.cwd,
        model: request.model,
        effort: request.effort,
        permission_mode: request.permission_mode,
    };
    let spec = build_launch_spec(launch).map_err(|_| StatusCode::BAD_REQUEST)?;
    launch_terminal_session(&spec).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn events(
    State(state): State<WebState>,
    headers: HeaderMap,
    websocket: WebSocketUpgrade,
) -> Result<impl IntoResponse, StatusCode> {
    authorize_websocket(&state.auth, &headers)?;
    let events = state.sessions.subscribe_events();
    let protocol = format!("grok-local.{}", state.auth.token());
    Ok(websocket
        .protocols([protocol])
        .on_upgrade(move |socket| stream_events(socket, events)))
}

async fn stream_events(
    mut socket: WebSocket,
    mut events: tokio::sync::broadcast::Receiver<crate::domain::chat::FrontendEvent>,
) {
    while let Ok(event) = events.recv().await {
        let Ok(payload) = serde_json::to_string(&event) else {
            continue;
        };
        if socket.send(Message::Text(payload.into())).await.is_err() {
            break;
        }
    }
}

fn authorize_websocket(auth: &LocalAuth, headers: &HeaderMap) -> Result<(), StatusCode> {
    let origin = headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok());
    if origin != Some(auth.origin()) {
        return Err(StatusCode::FORBIDDEN);
    }
    let protocol = headers
        .get(header::SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok());
    let expected = format!("grok-local.{}", auth.token());
    if protocol
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .any(|value| value == expected)
    {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
