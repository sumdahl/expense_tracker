use axum::{middleware, routing::get, routing::post, Router};

use crate::handlers;
use crate::middleware::auth::require_auth;
use crate::state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/expenses",
            post(handlers::expenses::create_expense).get(handlers::expenses::list_expenses),
        )
        .route(
            "/expenses/:id",
            get(handlers::expenses::get_expense)
                .put(handlers::expenses::update_expense)
                .delete(handlers::expenses::delete_expense),
        )
        .route(
            "/dashboard/expenses",
            post(handlers::expenses::create_expense_form),
        )
        .route(
            "/dashboard/expenses/:id/delete",
            post(handlers::expenses::delete_expense_form),
        )
        .route_layer(middleware::from_fn_with_state(state, require_auth))
}
