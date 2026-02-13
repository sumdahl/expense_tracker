use axum::{routing::post, Router};

use crate::handlers;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/signup", post(handlers::auth::sign_up_form))
        .route("/signin", post(handlers::auth::sign_in_form))
        .route("/signout", post(handlers::auth::signout))
        .route("/api/signup", post(handlers::auth::sign_up_json))
        .route("/api/signin", post(handlers::auth::sign_in_json))
}
