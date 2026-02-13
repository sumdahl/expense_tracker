use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{extract::State, response::Html, Form, Json};
use uuid::Uuid;

use crate::models::auth::{SignInInput, SignUpInput};
use crate::models::response::{ApiResponse, AuthResult, ErrorResponse, UserData};
use crate::models::user::User;
use crate::services;
use crate::state::AppState;

type AuthError = (StatusCode, Json<ErrorResponse>);

// Database error helper
fn handle_db_error(e: sqlx::Error) -> AuthError {
    if let sqlx::Error::Database(db_err) = &e {
        // PostgreSQL unique violation
        if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse::new("Email already exists")),
            );
        }
    }

    tracing::error!("Database error: {:?}", e);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("Database error")),
    )
}

async fn process_sign_up(state: &AppState, payload: &SignUpInput) -> Result<String, AuthError> {
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
    .map_err(handle_db_error)?;

    Ok(user_id.to_string())
}

pub async fn process_sign_in(
    state: &AppState,
    payload: &SignInInput,
) -> Result<AuthResult, AuthError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email=$1")
        .bind(&payload.email)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error during sign in : {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Database error")),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse::new("Invalid email or password")),
            )
        })?;

    if !user.verify_password(&payload.password) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse::new("Invalid email or password")),
        ));
    }

    let token =
        services::auth::encode_token(&user.id.to_string(), &state.jwt_secret, 24).map_err(|e| {
            tracing::error!("Token generation error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Authentication error")),
            )
        })?;

    Ok(AuthResult {
        token,
        user: UserData {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
        },
    })
}

pub async fn sign_up_form(
    State(state): State<AppState>,
    Form(payload): Form<SignUpInput>,
) -> Result<Response, AuthError> {
    process_sign_up(&state, &payload).await?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/signin"));

    Ok((headers, Html(String::new())).into_response())
}

pub async fn sign_in_form(
    State(state): State<AppState>,
    Form(payload): Form<SignInInput>,
) -> Result<Response, AuthError> {
    let auth_result = process_sign_in(&state, &payload).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_str(&format!(
            "auth_token={}; HttpOnly; Path=/; SameSite=Lax",
            auth_result.token
        ))
        .map_err(|e| {
            tracing::error!("Invalid header value: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Authentication error")),
            )
        })?,
    );
    headers.insert("HX-Redirect", HeaderValue::from_static("/dashboard"));

    Ok((StatusCode::OK, headers, Html(String::new())).into_response())
}

pub async fn sign_up_json(
    State(state): State<AppState>,
    Json(payload): Json<SignUpInput>,
) -> Result<Json<ApiResponse<UserData>>, AuthError> {
    let name = payload.name.clone();
    let email = payload.email.clone();
    let user_id = process_sign_up(&state, &payload).await?;

    Ok(Json(ApiResponse::success(
        "User created successfully",
        UserData {
            id: user_id,
            name,
            email,
        },
    )))
}

pub async fn sign_in_json(
    State(state): State<AppState>,
    Json(payload): Json<SignInInput>,
) -> Result<Json<ApiResponse<AuthResult>>, AuthError> {
    let auth = process_sign_in(&state, &payload).await?;

    Ok(Json(ApiResponse::success("Signed in successfully", auth)))
}

pub async fn signout(State(_state): State<AppState>) -> Result<Response, AuthError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_static("auth_token=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0"),
    );
    headers.insert("HX-Redirect", HeaderValue::from_static("/signin"));

    Ok((StatusCode::OK, headers, Html(String::new())).into_response())
}
