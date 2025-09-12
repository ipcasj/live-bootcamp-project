use axum::{extract::{State, Json}, response::IntoResponse};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use std::sync::Arc;
use crate::{app_state::AppState, auth_middleware::AuthenticatedUser, domain::{AuthAPIError, Email}};

#[derive(Serialize, ToSchema)]
pub struct AccountSettingsResponse {
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
    #[serde(rename = "twoFAMethod")]
    pub two_fa_method: crate::domain::user::TwoFAMethod,
}

#[utoipa::path(
    get,
    path = "/account/settings",
    responses(
        (status = 200, description = "Current account 2FA settings", body = AccountSettingsResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Unexpected error")
    ),
    tag = "account"
)]
pub async fn get_account_settings(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&user.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let user_store = state.user_store.read().await;
    let (requires_2fa, two_fa_method) = user_store.get_user_settings(&email).await.map_err(|_| AuthAPIError::InvalidCredentials)?;
    Ok(axum::Json(AccountSettingsResponse {
        requires_2fa,
        two_fa_method,
    }))
}


#[derive(Deserialize, ToSchema)]
pub struct Update2FARequest {
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
    #[serde(rename = "twoFAMethod", default)]
    pub two_fa_method: Option<crate::domain::user::TwoFAMethod>,
}

#[utoipa::path(
    patch,
    path = "/account/settings",
    request_body = Update2FARequest,
    responses(
        (status = 200, description = "2FA setting updated"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Unexpected error")
    ),
    tag = "account"
)]
pub async fn update_2fa_setting(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<Update2FARequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&user.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let mut user_store = state.user_store.write().await;
    let mut user = user_store.get_user(&email).await.map_err(|_| AuthAPIError::InvalidCredentials)?;
    user.requires_2fa = payload.requires_2fa;
    if let Some(ref method) = payload.two_fa_method {
        user.two_fa_method = method.clone();
    }
    let two_fa_method = user.two_fa_method.clone();
    user_store.update_user(user).await.map_err(|_| AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to update user")))?;
    Ok(axum::Json(AccountSettingsResponse {
        requires_2fa: payload.requires_2fa,
        two_fa_method,
    }))
}
