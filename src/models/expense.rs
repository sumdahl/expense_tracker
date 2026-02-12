use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateExpense {
    pub title: String,
    pub amount: f64,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Expense {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub amount: f64,
    pub category: Option<String>,
    pub created_at: DateTime<Utc>,
}
