use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: T,
}

#[derive(Serialize)]
pub struct AuthResult {
    pub token: String,
    pub user: UserData,
}

#[derive(Serialize)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
