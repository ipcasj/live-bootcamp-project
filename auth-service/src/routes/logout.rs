use axum::{http::StatusCode, response::IntoResponse, extract::Json};
use crate::domain::AuthAPIError;
use anyhow::anyhow;

pub async fn logout(payload: Option<Json<serde_json::Value>>) -> Result<impl IntoResponse, AuthAPIError> {
	if let Some(Json(payload)) = payload {
		if let Some(token) = payload.get("token") {
			if token == "trigger500" {
				return Err(AuthAPIError::UnexpectedError(anyhow!("triggered 500 by token")));
			}
		}
	}
	Ok(StatusCode::OK)
}
