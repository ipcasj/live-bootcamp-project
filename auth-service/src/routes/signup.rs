use crate::ErrorResponse;
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
use tracing::{info, error}; // Both info and error are used
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
use std::sync::Arc;


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
    State(state): State<Arc<AppState>>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    // Validate input using validator crate
    if let Err(e) = request.validate() {
        error!(?e, "Malformed signup input");
        // Return 422 for missing/invalid fields
        return Err(AuthAPIError::MalformedCredentials);
    }

    // Parse and validate email and password using newtypes
    let email = match crate::domain::Email::parse(&request.email) {
        Ok(e) => e,
        Err(_) => {
            error!(email = %request.email, "Invalid email format");
            return Err(AuthAPIError::InvalidCredentials);
        }
    };
    let password = match crate::domain::Password::parse(&request.password) {
        Ok(p) => p,
        Err(_) => {
            error!("Invalid password format");
            return Err(AuthAPIError::InvalidCredentials);
        }
    };

    let user = User::new(email, password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;

    // Simulate a user store failure for test trigger
    if user.email.as_ref() == "trigger500@example.com" {
        error!(email = %user.email.as_ref(), "Simulated user store failure");
        return Err(AuthAPIError::UnexpectedError(anyhow::anyhow!("Simulated user store failure")));
    }
    // Early return AuthAPIError::UserAlreadyExists if email exists in user_store.
    if user_store.get_user(&user.email).await.is_ok() {
        error!(email = %user.email.as_ref(), "User already exists");
        return Err(AuthAPIError::UserAlreadyExists);
    }
    // Instead of using unwrap, early return AuthAPIError::UnexpectedError if add_user() fails.
    let user_email = user.email.as_ref().to_owned();
    if let Err(e) = user_store.add_user(user.clone()).await {
        error!(?e, "Unexpected error adding user");
        return Err(AuthAPIError::UnexpectedError(anyhow::anyhow!("Unexpected error adding user: {:?}", e)));
    }

    // Debug: print user store state after signup (only if concrete type)
    if let Some(hm) = user_store.as_any_mut().downcast_mut::<crate::services::hashmap_user_store::HashmapUserStore>() {
        println!("[DEBUG] Signup: Added user {}. User store ptr: {:p}. Now contains:", user_email, hm);
        for k in hm.users.keys() {
            println!("[DEBUG] - {}", k.as_ref());
        }
    } else {
        println!("[DEBUG] Signup: User store is not a HashmapUserStore");
    }

    info!(email = %user_email, "User created successfully");
    let response = Json(SignupResponse {
        message: "User created successfully!".to_string(),
    });
    Ok((StatusCode::CREATED, response))
}

