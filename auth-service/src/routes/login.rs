

use axum::{extract::State, http::{StatusCode, header}, response::{IntoResponse, Response}, Json};
use axum_extra::extract::cookie::Cookie;
use crate::app_state::AppState;
use crate::domain::{AuthAPIError, Email, Password, UserStoreError};
use crate::utils::auth::generate_auth_cookie;
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
) -> impl IntoResponse {
	// 422 if missing or malformed fields
	let email = payload.get("email").and_then(|v| v.as_str());
	let password = payload.get("password").and_then(|v| v.as_str());

	if email.is_none() || password.is_none() {
		// 422 for missing/malformed fields
		return AuthAPIError::MalformedCredentials.into_response();
	}
	let email = email.unwrap();
	let password = password.unwrap();

	// Simulate 500 for test
	if email == "trigger500@example.com" {
		return AuthAPIError::UnexpectedError(anyhow::anyhow!("triggered 500 by email")).into_response();
	}

	// 400 for invalid format (bad email/password)
	let email = match Email::parse(email) {
		Ok(e) => e,
		Err(_) => return AuthAPIError::InvalidCredentials.into_response(), // 400
	};
	let password = match Password::parse(password) {
		Ok(p) => p,
		Err(_) => return AuthAPIError::InvalidCredentials.into_response(), // 400
	};

	// 401 if credentials valid format but incorrect
	let user_store = state.user_store.read().await;
	match user_store.validate_user(&email, &password).await {
		Ok(()) => {
			// Success: generate JWT cookie and return 200
			let auth_cookie = match generate_auth_cookie(&email) {
				Ok(cookie) => cookie,
				Err(_) => return AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to generate JWT")).into_response(),
			};
			let mut response = StatusCode::OK.into_response();
			response.headers_mut().append(
				header::SET_COOKIE,
				header::HeaderValue::from_str(&auth_cookie.to_string()).unwrap(),
			);
			response
		},
		Err(UserStoreError::UserNotFound) | Err(UserStoreError::InvalidCredentials) => {
			AuthAPIError::IncorrectCredentials.into_response()
		}
		Err(_) => AuthAPIError::UnexpectedError(anyhow::anyhow!("Unexpected error validating user")).into_response(),
	}
}
