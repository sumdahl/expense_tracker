use askama::Template;
use axum::{extract::State, http::StatusCode, response::Html, Extension};
use uuid::Uuid;

use crate::middleware::auth::AuthUser;
use crate::models::expense::Expense;
use crate::state::AppState;

#[derive(Template)]
#[template(path = "signin.html")]
struct SignInTemplate;

#[derive(Template)]
#[template(path = "signup.html")]
struct SignUpTemplate;

#[derive(Template)]
#[template(path = "landing.html")]
struct LandingTemplate;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    expenses: Vec<DashboardExpense>,
    total: f64,
}

#[derive(Clone)]
struct DashboardExpense {
    id: Uuid,
    title: String,
    category: String,
    amount: f64,
}

pub async fn index() -> Result<Html<String>, (StatusCode, String)> {
    let template = LandingTemplate;

    template
        .render()
        .map(Html)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error".into()))
}

pub async fn sign_in() -> Result<Html<String>, (StatusCode, String)> {
    let template = SignInTemplate;

    template
        .render()
        .map(Html)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error".into()))
}

pub async fn sign_up() -> Result<Html<String>, (StatusCode, String)> {
    let template = SignUpTemplate;

    template
        .render()
        .map(Html)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error".into()))
}

pub async fn dashboard(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> Result<Html<String>, (StatusCode, String)> {
    let expenses = fetch_expenses(&state, auth.user_id).await?;

    let view_models: Vec<DashboardExpense> = expenses
        .into_iter()
        .map(|expense| DashboardExpense {
            id: expense.id,
            title: expense.title,
            category: expense
                .category
                .unwrap_or_else(|| "Uncategorized".to_string()),
            amount: expense.amount,
        })
        .collect();

    let total: f64 = if view_models.is_empty() {
        0.0
    } else {
        view_models.iter().map(|e| e.amount).sum()
    };

    let template = DashboardTemplate {
        expenses: view_models,
        total,
    };

    template
        .render()
        .map(Html)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Template error".into()))
}

async fn fetch_expenses(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<Expense>, (StatusCode, String)> {
    let expenses = sqlx::query_as::<_, Expense>(
        "SELECT id, user_id, title, amount, category, created_at
        FROM expenses
        WHERE user_id = $1
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".into()))?;

    Ok(expenses)
}
