use axum::{middleware, routing::get, Router};
use tower_http::services::ServeDir;

use crate::handlers;
use crate::middleware::auth::require_auth;
use crate::state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let public_routes = Router::new()
        .route("/", get(handlers::pages::index))
        .route("/index", get(handlers::pages::index))
        .route("/signin", get(handlers::pages::sign_in))
        .route("/signup", get(handlers::pages::sign_up))
        .nest_service("/static", ServeDir::new("frontend"));

    let protected_routes = Router::new()
        .route("/dashboard", get(handlers::pages::dashboard))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    public_routes.merge(protected_routes)
}
