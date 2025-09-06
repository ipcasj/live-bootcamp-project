use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use crate::app_state::AppState;
use std::sync::Arc;
use crate::utils::auth::validate_token;
use crate::domain::AuthAPIError;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
	pub token: String,
}

pub async fn verify_token(
	State(_state): State<Arc<AppState>>,
	Json(payload): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
	match validate_token(&payload.token).await {
		Ok(_) => Ok(StatusCode::OK),
		Err(_) => Err(AuthAPIError::InvalidToken),
	}
}
