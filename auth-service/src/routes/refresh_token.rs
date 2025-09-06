use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use std::sync::Arc;
use crate::app_state::AppState;
use crate::utils::auth::{validate_refresh_token, generate_auth_token_from_str, generate_refresh_token_from_str};
use crate::domain::AuthAPIError;

#[derive(Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate the refresh token (signature, expiry, revocation)
    let claims = validate_refresh_token(&payload.refresh_token, Some(state.banned_token_store.clone())).await?;
    // Revoke (ban) the used refresh token
    state.banned_token_store.ban_token(payload.refresh_token.clone()).await;
    // Issue new access and refresh tokens
    let email = claims.sub;
    let new_access_token = generate_auth_token_from_str(&email)
        .map_err(|_| AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to generate access token")))?;
    let new_refresh_token = generate_refresh_token_from_str(&email)
        .map_err(|_| AuthAPIError::UnexpectedError(anyhow::anyhow!("Failed to generate refresh token")))?;
    Ok(Json(serde_json::json!({
        "access_token": new_access_token,
        "refresh_token": new_refresh_token
    })))
}
