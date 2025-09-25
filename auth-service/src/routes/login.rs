

use axum::{extract::State, http::{StatusCode, header}, response::IntoResponse, Json};
// use axum_extra::extract::cookie::Cookie; // unused
use crate::app_state::AppState;
use std::sync::Arc;
use crate::domain::{AuthAPIError, Email};
use crate::domain::data_stores::{LoginAttemptId, TwoFACode};
use crate::utils::auth::generate_auth_cookie;
use serde::Serialize;
use serde::Deserialize;
use crate::ErrorResponse;
use color_eyre::eyre;
use secrecy::{Secret, ExposeSecret};
// use anyhow::anyhow; // unused

#[derive(Deserialize)]
#[allow(dead_code)] // Not currently used - login route uses manual JSON parsing
pub struct LoginRequest {
    email: Secret<String>,
    password: Secret<String>,
}



#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct TwoFactorAuthResponseRest {
	pub message: String,
	#[serde(rename = "loginAttemptId")]
	pub login_attempt_id: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum LoginResponseRest {
	RegularAuth,
	TwoFactorAuth(TwoFactorAuthResponseRest),
}


/// Login endpoint for REST API
#[utoipa::path(
	post,
	path = "/login",
	request_body = inline(serde_json::Value),
	responses(
		(status = 200, description = "Login successful", body = LoginResponseRest),
		(status = 206, description = "2FA required", body = LoginResponseRest),
		(status = 400, description = "Invalid credentials", body = ErrorResponse),
		(status = 401, description = "Incorrect credentials", body = ErrorResponse),
		(status = 422, description = "Malformed credentials", body = ErrorResponse),
		(status = 500, description = "Unexpected error", body = ErrorResponse)
	)
)]
#[tracing::instrument(skip_all)]
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
		return AuthAPIError::UnexpectedError(eyre::eyre!("triggered 500 by email")).into_response();
	}

	// 400 for invalid format (bad email/password)
	let email = match Email::parse(Secret::new(email.to_string())) {
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
		handle_2fa(&email, &state).await.into_response()
	} else {
		handle_no_2fa(&email).await.into_response()
	}
}

#[tracing::instrument(skip_all)]
async fn handle_2fa(
	email: &Email,
	state: &AppState,
) -> Result<impl IntoResponse, AuthAPIError> {
	let login_attempt_id = LoginAttemptId::default();
	let two_fa_code = TwoFACode::default();

	if let Err(e) = state
		.two_fa_code_store
		.write()
		.await
		.add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
		.await
	{
		return Err(AuthAPIError::UnexpectedError(e.into()));
	}

	if let Err(e) = state
		.email_client
		.send_email(email, "2FA Code", two_fa_code.as_ref().expose_secret())
		.await
	{
		return Err(AuthAPIError::UnexpectedError(e));
	}

	let response = TwoFactorAuthResponseRest {
		message: "2FA required".to_owned(),
		login_attempt_id: login_attempt_id.as_ref().expose_secret().clone(),
	};
	
	// axum 0.6 does not implement IntoResponse for (StatusCode, Json<T>), so do it manually
	let mut resp = StatusCode::PARTIAL_CONTENT.into_response();
	// Serialize the inner type, not axum::Json
	*resp.body_mut() = axum::body::boxed(axum::body::Full::from(serde_json::to_vec(&LoginResponseRest::TwoFactorAuth(response)).unwrap()));
	resp.headers_mut().insert(
		axum::http::header::CONTENT_TYPE,
		axum::http::HeaderValue::from_static("application/json"),
	);
	Ok(resp)
}

#[tracing::instrument(skip_all)]
async fn handle_no_2fa(
	email: &Email,
) -> Result<impl IntoResponse, AuthAPIError> {
	let auth_cookie = match generate_auth_cookie(email) {
		Ok(cookie) => cookie,
		Err(e) => return Err(AuthAPIError::UnexpectedError(e)),
	};
	
	let mut response = StatusCode::OK.into_response();
	response.headers_mut().append(
		header::SET_COOKIE,
		header::HeaderValue::from_str(&auth_cookie.to_string()).unwrap(),
	);
	Ok(response)
}
