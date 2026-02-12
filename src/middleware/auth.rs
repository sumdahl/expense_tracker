use axum::{
    body::Body,
    http::{header::AUTHORIZATION, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::services;
use crate::state::AppState;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

pub async fn require_auth(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let state = req
        .extensions()
        .get::<AppState>()
        .cloned()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "State missing".into()))?;

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value: &HeaderValue| value.to_str().ok())
        .unwrap_or("");

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "Missing bearer token".into()))?;

    let claims = services::auth::decode_token(token, &state.jwt_secret)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token subject".into()))?;

    req.extensions_mut().insert(AuthUser { user_id });
    Ok(next.run(req).await)
}
