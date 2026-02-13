use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, response::Html, Form, Json};
use uuid::Uuid;

use crate::models::auth::{SignInInput, SignUpInput};
use crate::models::response::{ApiResponse, AuthResult, UserData};
use crate::models::user::User;
use crate::services;
use crate::state::AppState;

async fn process_sign_up(
    state: &AppState,
    payload: &SignUpInput,
) -> Result<String, (StatusCode, String)> {
    let password_hash = User::hash_password(&payload.password);

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO users (id, name, email, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id",
    )
    .bind(Uuid::new_v4())
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?;

    Ok(user_id.to_string())
}

pub async fn process_sign_in(
    state: &AppState,
    payload: &SignInInput,
) -> Result<AuthResult, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email=$1")
        .bind(&payload.email)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid email or password".into()))?;

    if !user.verify_password(&payload.password) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password".into()))?;
    }

    let token = services::auth::encode_token(&user.id.to_string(), &state.jwt_secret, 24)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token error".into()))?;

    Ok(AuthResult {
        token,
        user: UserData {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
        },
    })
}

async fn process_sign_in_token(
    state: &AppState,
    payload: &SignInInput,
) -> Result<String, (StatusCode, String)> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email=$1")
        .bind(&payload.email)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid email or password".into()))?;

    if !user.verify_password(&payload.password) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password".into()));
    }

    let token = services::auth::encode_token(&user.id.to_string(), &state.jwt_secret, 24)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token error".into()))?;

    Ok(token)
}

pub async fn sign_up_form(
    State(state): State<AppState>,
    Form(payload): Form<SignUpInput>,
) -> Result<Response, (StatusCode, String)> {
    let _user_id = process_sign_up(&state, &payload).await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/signin"));
    Ok((headers, Html(String::new())).into_response())
}

pub async fn sign_in_form(
    State(state): State<AppState>,
    Form(payload): Form<SignInInput>,
) -> Result<Response, (StatusCode, String)> {
    let token = process_sign_in_token(&state, &payload).await?;
    let mut headers = HeaderMap::new();

    headers.insert(
        "Set-Cookie",
        HeaderValue::from_str(&format!(
            "auth_token={}; HttpOnly; Path=/; SameSite=Lax",
            token
        ))
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid header: {}", e),
            )
        })?,
    );
    headers.insert("HX-Redirect", HeaderValue::from_static("/dashboard"));
    Ok((StatusCode::OK, headers, Html(String::new())).into_response())
}

pub async fn sign_up_json(
    State(state): State<AppState>,
    Json(payload): Json<SignUpInput>,
) -> Result<Json<ApiResponse<UserData>>, (StatusCode, String)> {
    let name = payload.name.clone();
    let email = payload.email.clone();
    let user_id = process_sign_up(&state, &payload).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: "User created successfully".to_string(),
        data: UserData {
            id: user_id,
            name,
            email,
        },
    };

    Ok(Json(response))
}

pub async fn sign_in_json(
    State(state): State<AppState>,
    Json(payload): Json<SignInInput>,
) -> Result<Json<ApiResponse<AuthResult>>, (StatusCode, String)> {
    let auth = process_sign_in(&state, &payload).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: "Signed in successfully".to_string(),
        data: auth,
    };

    Ok(Json(response))
}

pub async fn signout(State(_state): State<AppState>) -> Result<Response, (StatusCode, String)> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_static("auth_token=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"),
    );
    headers.insert("HX-Redirect", HeaderValue::from_static("/signin"));
    Ok((StatusCode::OK, headers, Html(String::new())).into_response())
}
