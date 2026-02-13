use axum::{
    body::Body,
    http::{header::AUTHORIZATION, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::services;
use crate::state::AppState;
use axum::extract::State;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let token = extract_token(req.headers())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing bearer token".into()))?;

    let claims = services::auth::decode_token(&token, &state.jwt_secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token subject".into()))?;

    req.extensions_mut().insert(AuthUser { user_id });
    Ok(next.run(req).await)
}

fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(auth_header) = headers
        .get(AUTHORIZATION)
        .and_then(|value: &HeaderValue| value.to_str().ok())
    {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let trimmed = part.trim();
        if let Some(value) = trimmed.strip_prefix("auth_token=") {
            return Some(value.to_string());
        }
    }

    None
}
