use sqlx::postgres::PgPoolOptions;
use std::env;

pub type DbPool = sqlx::PgPool;

pub async fn init_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}
