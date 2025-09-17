use axum::{extract::{State, Json}, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use std::sync::Arc;
use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, Password},
    domain::data_stores::{TwoFACodeStore, LoginAttemptId, TwoFACode},
};

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "code")]
    pub code: String,
    pub new_password: String,
}

use serde::Serialize;

#[derive(Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
    pub login_attempt_id: String,
}

pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Always respond with generic message
    let email = match Email::parse(&request.email) {
        Ok(e) => e,
        Err(_) => return Ok((StatusCode::OK, Json(serde_json::json!({"message": "If this email exists, you'll receive a reset code."})))),
    };
    // Generate a code and a new login_attempt_id (UUID)
    let code = TwoFACode::default();
    let login_attempt_id = LoginAttemptId::default();
    {
        let mut store = state.two_fa_code_store.write().await;
        let _ = store.add_code(email.clone(), login_attempt_id.clone(), code.clone()).await;
    }
    // Send code and login_attempt_id via email (reuse email_client)
    let _ = state.email_client.send_2fa_code(email.as_ref(), code.as_ref()).await;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "message": "If this email exists, you'll receive a reset code.",
        "loginAttemptId": login_attempt_id.as_ref()
    }))))
}

pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let mut store = state.two_fa_code_store.write().await;
    let (login_attempt_id, code) = store.get_code(&email).await.map_err(|_| AuthAPIError::InvalidToken)?;
    if login_attempt_id.as_ref() != request.login_attempt_id || code.as_ref() != request.code {
        return Err(AuthAPIError::InvalidToken);
    }
    // Validate new password
    let new_password = Password::parse(&request.new_password).map_err(|_| AuthAPIError::MalformedCredentials)?;
    // Update password in user store
    {
        let mut user_store = state.user_store.write().await;
        user_store.update_password(&email, new_password).await.map_err(|_| AuthAPIError::InvalidToken)?;
    }
    // Remove code
    let _ = store.remove_code(&email).await;
    Ok((StatusCode::OK, Json(serde_json::json!({"message": "Password reset successful."}))))
}
