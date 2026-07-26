use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    routing::get,
};

use crate::{domain::command_catalog::DesktopCommand, web::auth::LocalAuth};

const TOKEN_HEADER: &str = "x-grok-local-token";

#[derive(Clone)]
struct WebState {
    auth: LocalAuth,
}

pub fn router(auth: LocalAuth) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/commands", get(command_catalog))
        .with_state(WebState { auth })
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
) -> Result<axum::Json<Vec<DesktopCommand>>, StatusCode> {
    authorize(&state.auth, &headers)?;
    Ok(axum::Json(commands()))
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

fn commands() -> Vec<DesktopCommand> {
    vec![
        DesktopCommand::from_slash("/rename", true, true),
        DesktopCommand::from_slash("/model", true, true),
        DesktopCommand::from_slash("/effort", true, true),
        DesktopCommand::from_slash("/new", false, false),
        DesktopCommand::from_slash("/clear", false, false),
        DesktopCommand::from_slash("/quit", false, false),
    ]
}
