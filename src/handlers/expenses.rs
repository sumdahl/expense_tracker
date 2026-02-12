use axum::{extract::State, extract::Path, Extension, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::models::expense::{CreateExpense, Expense};
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

pub async fn create_expense(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<CreateExpense>,
) -> Result<Json<Expense>, (axum::http::StatusCode, String)> {
    let expense_id = Uuid::new_v4();
    let title = payload.title.clone();
    let category = payload.category.clone();
    sqlx::query(
        "INSERT INTO expenses (id, user_id, title, amount, category) VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(expense_id)
    .bind(auth.user_id)
    .bind(title.clone())
    .bind(payload.amount)
    .bind(category.clone())
    .execute(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

    let expense = Expense {
        id: expense_id,
        user_id: auth.user_id,
        title,
        amount: payload.amount,
        category,
        created_at: Utc::now(),
    };

    Ok(Json(expense))
}

pub async fn list_expenses(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Json<Vec<Expense>>, (axum::http::StatusCode, String)> {
    let expenses = sqlx::query_as::<_, Expense>(
        "SELECT id, user_id, title, amount, category, created_at
        FROM expenses
        WHERE user_id = $1
        ORDER BY created_at DESC",
    )
    .bind(auth.user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "DB error".into(),
        )
    })?;

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
