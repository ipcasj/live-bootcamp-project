use axum::{http::StatusCode, response::IntoResponse, Json};
use crate::domain::AuthAPIError;
use anyhow::anyhow;

pub async fn verify_2fa(Json(payload): Json<serde_json::Value>) -> Result<impl IntoResponse, AuthAPIError> {
	if let Some(code) = payload.get("code") {
		if code == "trigger500" {
			return Err(AuthAPIError::UnexpectedError(anyhow!("triggered 500 by code")));
		}
	}
	Ok(StatusCode::OK)
}
