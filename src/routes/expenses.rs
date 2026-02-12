use axum::{middleware, routing::get, routing::post, Router};

use crate::handlers;
use crate::middleware::auth::require_auth;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/expenses",
            post(handlers::expenses::create_expense).get(handlers::expenses::list_expenses),
        )
        .route("/expenses/:id", get(handlers::expenses::get_expense))
        .route_layer(middleware::from_fn(require_auth))
}
