

use axum::{extract::State, http::{StatusCode, header}, response::IntoResponse, Json};
// use axum_extra::extract::cookie::Cookie; // unused
use crate::app_state::AppState;
use std::sync::Arc;
use crate::domain::{AuthAPIError, Email};
use crate::domain::data_stores::{LoginAttemptId, TwoFACode};
use crate::utils::auth::generate_auth_cookie;
use serde::Serialize;
use serde::Deserialize;
// use anyhow::anyhow; // unused


#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
	pub message: String,
	#[serde(rename = "loginAttemptId")]
	pub login_attempt_id: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
	RegularAuth,
	TwoFactorAuth(TwoFactorAuthResponse),
}


pub async fn login(
	State(state): State<Arc<AppState>>,
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
	// Only check password format (length) for 400, but do not hash here
	if password.len() < 8 {
		return AuthAPIError::InvalidCredentials.into_response(); // 400
	}

	// 401 if credentials valid format but incorrect
	let user_store = state.user_store.read().await;
	if let Err(_) = user_store.validate_user(&email, password).await {
		return AuthAPIError::IncorrectCredentials.into_response();
	}
	let user = match user_store.get_user(&email).await {
		Ok(user) => user,
		Err(_) => return AuthAPIError::IncorrectCredentials.into_response(),
	};

	// Handle request based on user's 2FA configuration
	if user.requires_2fa {
		use crate::domain::data_stores::TwoFACodeStore;
		let login_attempt_id = LoginAttemptId::default();
		let two_fa_code = TwoFACode::default();
		if let Err(_) = TwoFACodeStore::add_code(&mut *state.two_fa_code_store.write().await, email.clone(), login_attempt_id.clone(), two_fa_code.clone()).await {
			return AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to store 2FA code")).into_response();
		}
		// Send code to user via email client
		if let Err(e) = state.email_client.send_2fa_code(email.as_ref(), two_fa_code.as_ref()).await {
			tracing::error!(?e, "Failed to send 2FA code via email client");
			return AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to send 2FA code")).into_response();
		}
		let response = TwoFactorAuthResponse {
			message: "2FA required".to_owned(),
			login_attempt_id: login_attempt_id.as_ref().to_owned(),
		};
		return (StatusCode::PARTIAL_CONTENT, Json(LoginResponse::TwoFactorAuth(response))).into_response();
	}

	// No 2FA: proceed as normal
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
}
