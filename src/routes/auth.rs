use axum::{routing::post, Router};

use crate::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/signup", post(handlers::auth::sign_up))
        .route("/signin", post(handlers::auth::sign_in))
}
