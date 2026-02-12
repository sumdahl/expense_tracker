use axum::{extract::State, Json};
use uuid::Uuid;

use crate::models::auth::{SignInInput, SignUpInput};
use crate::models::user::User;
use crate::services;
use crate::state::AppState;

pub async fn sign_up(
    State(state): State<AppState>,
    Json(payload): Json<SignUpInput>,
) -> Result<Json<String>, (axum::http::StatusCode, String)> {
    let password_hash = User::hash_password(&payload.password);

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (id, name, email, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(payload.name)
    .bind(payload.email)
    .bind(password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

    Ok(Json(format!("User created with id : {}", user_id)))
}

pub async fn sign_in(
    State(state): State<AppState>,
    Json(payload): Json<SignInInput>,
) -> Result<Json<String>, (axum::http::StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email=$1")
        .bind(payload.email)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                "Invalid email or password".into(),
            )
        })?;

    if !user.verify_password(&payload.password) {
        return Err((
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid password".into(),
        ))?;
    }

    let token = services::auth::encode_token(&user.id.to_string(), &state.jwt_secret, 24).map_err(
        |_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Token error".into(),
            )
        },
    )?;

    Ok(Json(token))
}
