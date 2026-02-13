use axum::{extract::State, extract::Path, Extension, Json, Form};
use axum::response::IntoResponse;
use chrono::Utc;
use uuid::Uuid;

use crate::models::expense::{CreateExpense, Expense, UpdateExpense};
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn create_expense(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreateExpense>,
) -> Result<Json<Expense>, (axum::http::StatusCode, String)> {
    let expense = insert_expense(&state, auth.user_id, payload).await?;
    Ok(Json(expense))
}

pub async fn list_expenses(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<Expense>>, (axum::http::StatusCode, String)> {
    let expenses = fetch_expenses(&state, auth.user_id).await?;
    Ok(Json(expenses))
}

pub async fn get_expense(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(expense_id): Path<Uuid>,
) -> Result<Json<Expense>, (axum::http::StatusCode, String)> {
    let expense = sqlx::query_as::<_, Expense>(
        "SELECT id, user_id, title, amount, category, created_at
        FROM expenses
        WHERE id = $1 AND user_id = $2",
    )
    .bind(expense_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?
    .ok_or((axum::http::StatusCode::NOT_FOUND, "Not found".into()))?;

    Ok(Json(expense))
}

pub async fn update_expense(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(expense_id): Path<Uuid>,
    Json(payload): Json<UpdateExpense>,
) -> Result<Json<Expense>, (axum::http::StatusCode, String)> {
    let updated = sqlx::query_as::<_, Expense>(
        "UPDATE expenses
        SET title = COALESCE($1, title),
            amount = COALESCE($2, amount),
            category = COALESCE($3, category)
        WHERE id = $4 AND user_id = $5
        RETURNING id, user_id, title, amount, category, created_at",
    )
    .bind(payload.title)
    .bind(payload.amount)
    .bind(payload.category)
    .bind(expense_id)
    .bind(auth.user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?
    .ok_or((axum::http::StatusCode::NOT_FOUND, "Not found".into()))?;

    Ok(Json(updated))
}

pub async fn delete_expense(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(expense_id): Path<Uuid>,
) -> Result<Json<String>, (axum::http::StatusCode, String)> {
    let deleted = sqlx::query(
        "DELETE FROM expenses WHERE id = $1 AND user_id = $2",
    )
    .bind(expense_id)
    .bind(auth.user_id)
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

    if deleted.rows_affected() == 0 {
        return Err((axum::http::StatusCode::NOT_FOUND, "Not found".into()));
    }

    Ok(Json("Deleted".to_string()))
}

pub async fn create_expense_form(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Form(payload): Form<CreateExpense>,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let _expense = insert_expense(&state, auth.user_id, payload).await?;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("HX-Redirect", "/dashboard".parse().unwrap());
    Ok((axum::http::StatusCode::OK, headers, axum::response::Html(String::new())).into_response())
}

pub async fn delete_expense_form(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(expense_id): Path<Uuid>,
) -> Result<axum::response::Response, (axum::http::StatusCode, String)> {
    let deleted = sqlx::query(
        "DELETE FROM expenses WHERE id = $1 AND user_id = $2",
    )
    .bind(expense_id)
    .bind(auth.user_id)
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

    if deleted.rows_affected() == 0 {
        return Err((axum::http::StatusCode::NOT_FOUND, "Not found".into()));
    }

    let mut headers = axum::http::HeaderMap::new();
    headers.insert("HX-Redirect", "/dashboard".parse().unwrap());
    Ok((axum::http::StatusCode::OK, headers, axum::response::Html(String::new())).into_response())
}

async fn fetch_expenses(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Expense>, (axum::http::StatusCode, String)> {
    let expenses = sqlx::query_as::<_, Expense>(
        "SELECT id, user_id, title, amount, category, created_at
        FROM expenses
        WHERE user_id = $1
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

    Ok(expenses)
}

async fn insert_expense(
    state: &AppState,
    user_id: Uuid,
    payload: CreateExpense,
) -> Result<Expense, (axum::http::StatusCode, String)> {
    let expense_id = Uuid::new_v4();
    let title = payload.title;
    let category = payload.category;
    let amount = payload.amount;

    sqlx::query(
        "INSERT INTO expenses (id, user_id, title, amount, category) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(expense_id)
    .bind(user_id)
    .bind(&title)
    .bind(amount)
    .bind(&category)
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

    Ok(Expense {
        id: expense_id,
        user_id,
        title,
        amount,
        category,
        created_at: Utc::now(),
    })
}
