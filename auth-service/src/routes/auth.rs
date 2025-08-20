use axum::{http::StatusCode, response::IntoResponse};

pub async fn dummy_handler() -> impl IntoResponse {
    StatusCode::OK
}
