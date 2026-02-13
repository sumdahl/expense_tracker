use axum::Router;
use dotenv::dotenv;
use std::env;
use tokio::net::TcpListener;

mod db;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod state;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let pool = db::init_pool().await.expect("Failed to connect to DB");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let state = state::AppState {
        pool,
        jwt_secret: jwt_secret.into(),
    };

    let app = Router::new()
        .merge(routes::pages::router(state.clone()))
        .merge(routes::auth::router())
        .merge(routes::expenses::router(state.clone()))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
