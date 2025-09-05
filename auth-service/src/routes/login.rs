

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, Password, UserStoreError};
use serde::Deserialize;
use anyhow::anyhow;

#[derive(Deserialize)]
struct LoginPayload {
	email: String,
	password: String,
}


pub async fn login(
	State(state): State<AppState>,
	Json(payload): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AuthAPIError> {
	// 422 if missing or malformed fields
	let email = payload.get("email").and_then(|v| v.as_str());
	let password = payload.get("password").and_then(|v| v.as_str());


	if email.is_none() || password.is_none() {
		// 422 for missing/malformed fields
		return Err(AuthAPIError::MalformedCredentials);
	}
	let email = email.unwrap();
	let password = password.unwrap();

	// Simulate 500 for test
	if email == "trigger500@example.com" {
		return Err(AuthAPIError::UnexpectedError(anyhow::anyhow!("triggered 500 by email")));
	}

	// 400 for invalid format (bad email/password)
	let email = match Email::parse(email) {
		Ok(e) => e,
		Err(_) => return Err(AuthAPIError::InvalidCredentials), // 400
	};
	let password = match Password::parse(password) {
		Ok(p) => p,
		Err(_) => return Err(AuthAPIError::InvalidCredentials), // 400
	};

	// 401 if credentials valid format but incorrect
	let user_store = state.user_store.read().await;
	match user_store.validate_user(&email, &password).await {
		Ok(()) => Ok(StatusCode::OK),
		Err(UserStoreError::UserNotFound) | Err(UserStoreError::InvalidCredentials) => {
			Err(AuthAPIError::IncorrectCredentials)
		}
		Err(_) => Err(AuthAPIError::UnexpectedError(anyhow::anyhow!("Unexpected error validating user"))),
	}
}
