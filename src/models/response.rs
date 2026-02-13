use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub sucess: bool,
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

impl<T> ApiResponse<T> {
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            sucess: true,
            message: message.into(),
            data,
        }
    }
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            success: false,
            error: error.into(),
            details: Some(details.into()),
        }
    }
}
