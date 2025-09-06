use axum::{extract::State, response::IntoResponse, Json};
use crate::{app_state::AppState, domain::{AuthAPIError}, ErrorResponse};
use std::sync::Arc;
use crate::auth_middleware::AuthenticatedUser;

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct DeleteAccountResponse {
	pub message: String,
}

#[utoipa::path(
	delete,
	path = "/delete-account",
	responses(
		(status = 200, description = "Account deleted", body = DeleteAccountResponse),
		(status = 401, description = "Unauthorized", body = ErrorResponse),
		(status = 404, description = "Account not found", body = ErrorResponse),
		(status = 500, description = "Unexpected error", body = ErrorResponse)
	),
	tag = "auth"
)]
pub async fn delete_account(
	State(state): State<Arc<AppState>>,
	user: AuthenticatedUser,
) -> Result<impl IntoResponse, AuthAPIError> {
	let mut user_store = state.user_store.write().await;
	let email = match crate::domain::Email::parse(&user.email) {
		Ok(e) => e,
		Err(_) => return Err(AuthAPIError::InvalidCredentials),
	};
	user_store.delete_user(&email).await.map_err(|e| match e {
		crate::domain::UserStoreError::UserNotFound => AuthAPIError::InvalidCredentials,
		_ => AuthAPIError::UnexpectedError(anyhow::anyhow!("Unexpected error deleting user")),
	})?;
	Ok(Json(DeleteAccountResponse { message: "Account deleted".to_string() }))
}

