// Database error helper
fn handle_db_error(e: sqlx::Error) -> AuthError {
    if let sqlx::Error::Database(db_err) = &e {
        // PostgreSQL unique violation
        if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
            return (
                StatusCode::CONFLICT,
                Json(ErrorResponse::new("Email already exists")),
            );
        }
    }

    tracing::error!("Database error: {:?}", e);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("Database error")),
    )
}
