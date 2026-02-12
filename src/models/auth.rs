use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignUpInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignInInput {
    pub email: String,
    pub password: String,
}
