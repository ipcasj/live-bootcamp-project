// Free async handler for axum compatibility in tests
pub async fn audit_log_handler() -> AxumJson<Vec<AuditLogEntry>> {
	get_audit_log().await
}
use axum::{response::Json as AxumJson};
/// Returns the audit log as JSON (for admin/audit purposes only!)
pub async fn get_audit_log() -> AxumJson<Vec<AuditLogEntry>> {

	let log = AUDIT_LOG.lock().unwrap().clone();
	AxumJson(log)
}
use axum::{extract::{State, Json}, http::{StatusCode, header}, response::IntoResponse};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use std::sync::Arc;
use crate::{
	app_state::AppState,
	domain::{AuthAPIError, Email},
	domain::data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore},
	utils::auth::generate_auth_cookie,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Deserialize)]
pub struct Verify2FARequest {
	pub email: String,
	#[serde(rename = "loginAttemptId")]
	pub login_attempt_id: String,
	#[serde(rename = "2FACode")]
	pub two_fa_code: String,
}

// In-memory audit log for demonstration
use std::sync::Mutex;
use once_cell::sync::Lazy;
#[derive(Debug, Clone)]
pub struct AuditLogEntry {
	pub email: String,
	pub event: String,
	pub timestamp: u64,
	pub reason: Option<String>,
}
static AUDIT_LOG: Lazy<Mutex<Vec<AuditLogEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub async fn verify_2fa(
	State(state): State<Arc<AppState>>,
	jar: CookieJar,
	Json(request): Json<Verify2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
	let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	// Validate email
	let email = match Email::parse(&request.email) {
		Ok(e) => e,
		Err(_) => {
			AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
				email: request.email.clone(),
				event: "2fa_failed".into(),
				timestamp: now,
				reason: Some("invalid_email".into()),
			});
			return Err(AuthAPIError::InvalidCredentials);
		}
	};

	// Validate login_attempt_id and 2fa_code
	let login_attempt_id = match LoginAttemptId::parse(request.login_attempt_id.clone()) {
		Ok(id) => id,
		Err(_) => {
			AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
				email: request.email.clone(),
				event: "2fa_failed".into(),
				timestamp: now,
				reason: Some("invalid_login_attempt_id".into()),
			});
			return Err(AuthAPIError::InvalidCredentials);
		}
	};
	let two_fa_code = match TwoFACode::parse(request.two_fa_code.clone()) {
		Ok(code) => code,
		Err(_) => {
			AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
				email: request.email.clone(),
				event: "2fa_failed".into(),
				timestamp: now,
				reason: Some("invalid_2fa_code_format".into()),
			});
			return Err(AuthAPIError::InvalidCredentials);
		}
	};

	let mut code_store = state.two_fa_code_store.write().await;
	// Rate limiting: 5 max failed attempts
	let failed_attempts = code_store.get_failed_attempts(&email).await;
	if failed_attempts >= 5 {
		AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
			email: email.as_ref().to_string(),
			event: "2fa_failed".into(),
			timestamp: now,
			reason: Some("too_many_failed_attempts".into()),
		});
		return Err(AuthAPIError::IncorrectCredentials);
	}

	// Get code with meta (timestamp)
	let (stored_login_attempt_id, stored_code, issued_at) = match code_store.get_code_with_meta(&email).await {
		Ok(tuple) => tuple,
		Err(_) => {
			code_store.record_failed_attempt(&email).await;
			AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
				email: email.as_ref().to_string(),
				event: "2fa_failed".into(),
				timestamp: now,
				reason: Some("no_code_found".into()),
			});
			return Err(AuthAPIError::IncorrectCredentials);
		}
	};

	// Expiration: 5 minutes (300 seconds)
	if now > issued_at + 300 {
		code_store.remove_code(&email).await.ok();
		code_store.record_failed_attempt(&email).await;
		AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
			email: email.as_ref().to_string(),
			event: "2fa_failed".into(),
			timestamp: now,
			reason: Some("code_expired".into()),
		});
		return Err(AuthAPIError::IncorrectCredentials);
	}

	// Check login_attempt_id and code match (case sensitive)
	if stored_login_attempt_id != login_attempt_id || stored_code != two_fa_code {
		code_store.record_failed_attempt(&email).await;
		AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
			email: email.as_ref().to_string(),
			event: "2fa_failed".into(),
			timestamp: now,
			reason: Some("incorrect_code_or_attempt_id".into()),
		});
		return Err(AuthAPIError::IncorrectCredentials);
	}

	// Remove code after successful use (one-time)
	code_store.remove_code(&email).await.ok();
	code_store.reset_failed_attempts(&email).await;
	AUDIT_LOG.lock().unwrap().push(AuditLogEntry {
		email: email.as_ref().to_string(),
		event: "2fa_success".into(),
		timestamp: now,
		reason: None,
	});

	// Set JWT cookie
	let auth_cookie = generate_auth_cookie(&email)
		.map_err(|_| AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to generate JWT")))?;
	let mut response = StatusCode::OK.into_response();
	response.headers_mut().append(
		header::SET_COOKIE,
		header::HeaderValue::from_str(&auth_cookie.to_string()).unwrap(),
	);
	Ok((jar, response))
}
