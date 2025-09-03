use axum::{http::StatusCode, response::IntoResponse, Json};
use crate::domain::AuthAPIError;
use anyhow::anyhow;

pub async fn login(Json(payload): Json<serde_json::Value>) -> Result<impl IntoResponse, AuthAPIError> {
	if let Some(email) = payload.get("email") {
		if email == "trigger500@example.com" {
			return Err(AuthAPIError::UnexpectedError(anyhow!("triggered 500 by email")));
		}
	}
	Ok(StatusCode::OK)
}
