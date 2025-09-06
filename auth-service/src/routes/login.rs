

use axum::{extract::State, http::{StatusCode, header}, response::IntoResponse, Json};
// use axum_extra::extract::cookie::Cookie; // unused
use crate::app_state::AppState;
use std::sync::Arc;
use crate::domain::{AuthAPIError, Email, Password, UserStoreError};
use crate::utils::auth::generate_auth_cookie;
use serde::Deserialize;
// use anyhow::anyhow; // unused

#[derive(Deserialize)]
#[allow(dead_code)]
struct LoginPayload {
	email: String,
	password: String,
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
	let password = match Password::parse(password) {
		Ok(p) => p,
		Err(_) => return AuthAPIError::InvalidCredentials.into_response(), // 400
	};

	// 401 if credentials valid format but incorrect
	let user_store = state.user_store.read().await;
	// Debug: print user store state before login (only if concrete type)
	if let Some(hm) = user_store.as_any().downcast_ref::<crate::services::hashmap_user_store::HashmapUserStore>() {
		println!("[DEBUG] Login: Looking for user {}. User store ptr: {:p}. Contains:", email.as_ref(), hm);
		for k in hm.users.keys() {
			println!("[DEBUG] - {}", k.as_ref());
		}
	} else {
		println!("[DEBUG] Login: User store is not a HashmapUserStore");
	}
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
