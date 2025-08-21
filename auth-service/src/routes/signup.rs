use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

pub async fn signup(Json(_request): Json<SignupRequest>) -> impl IntoResponse {
    StatusCode::CREATED.into_response()
}

// Add a stub handler for /verify-2fa
use axum::response::Json as AxumJson;
pub async fn verify_2fa(_body: AxumJson<serde_json::Value>) -> impl IntoResponse {
    StatusCode::OK
}
