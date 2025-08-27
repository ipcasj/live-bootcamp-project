/// Health check endpoint for the auth-service.
#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service is healthy")),
    tag = "health"
)]
pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}
/// Signup route handler and types for user registration in the auth-service.
use utoipa::ToSchema;
use tracing::{info, error};
/// Signup route handler for user registration.
///
/// - Accepts POST requests with JSON body: { "email": String, "password": String, "requires2FA": bool }
/// - Validates email (must not be empty and must contain '@') and password (min 8 chars)
/// - Returns 201 and success message on success
/// - Returns 400 with error message for invalid credentials
/// - Returns 409 with error message if user already exists
/// - Returns 422 for malformed input (missing required fields)
/// - Returns 500 for unexpected errors
///
/// See also: AuthAPIError, SignupRequest, SignupResponse
/// # Example
/// 
/// 
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{app_state::AppState, domain::{User, AuthAPIError}};


#[derive(Deserialize, Validate, ToSchema)]
pub struct SignupRequest {
    #[validate(email, length(min = 1))]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}


#[derive(Serialize, ToSchema)]
pub struct SignupResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "User created", body = SignupResponse),
        (status = 400, description = "Invalid credentials", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse),
        (status = 422, description = "Malformed input", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    tag = "auth"
)]
/// Signup endpoint for user registration.

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate input using validator crate
    if let Err(e) = request.validate() {
        error!(?e, "Invalid signup credentials");
        return Err(AuthAPIError::InvalidCredentials);
    }

    let email = request.email;
    let password = request.password;

    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;

    // Early return AuthAPIError::UserAlreadyExists if email exists in user_store.
    if user_store.get_user(&user.email).await.is_ok() {
        error!(email = %user.email, "User already exists");
        return Err(AuthAPIError::UserAlreadyExists);
    }

    // Instead of using unwrap, early return AuthAPIError::UnexpectedError if add_user() fails.
    let user_email = user.email.clone();
    if let Err(e) = user_store.add_user(user).await {
        error!(?e, "Unexpected error adding user");
        return Err(AuthAPIError::UnexpectedError);
    }

    info!(email = %user_email, "User created successfully");
    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });
    Ok((StatusCode::CREATED, response))
}

// Add a stub handler for /verify-2fa
use axum::response::Json as AxumJson;
pub async fn verify_2fa(_body: AxumJson<serde_json::Value>) -> impl IntoResponse {
    StatusCode::OK
}
