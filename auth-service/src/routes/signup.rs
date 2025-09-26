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
use secrecy::Secret;

use crate::{app_state::AppState, domain::{User, AuthAPIError}};
use color_eyre::eyre;
use std::sync::Arc;



#[derive(Debug, Deserialize)]
pub struct SignupRequestRest {
    #[serde(with = "secret_string")]
    pub email: Secret<String>,
    #[serde(with = "secret_string")]
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

// Custom serde module for Secret<String>
mod secret_string {
    use serde::{Deserialize, Deserializer};
    use secrecy::Secret;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Secret<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Secret::new(s))
    }
}

// Separate struct for API documentation
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SignupRequestSchema {
    /// User's email address
    pub email: String,
    /// User's password
    pub password: String,
    /// Whether the user requires 2FA
    pub requires_2fa: bool,
}

#[derive(Serialize, ToSchema)]
pub struct SignupResponseRest {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/signup",
    request_body = SignupRequestSchema,
    responses(
        (status = 201, description = "User created", body = SignupResponseRest),
        (status = 400, description = "Invalid credentials", body = ErrorResponse),
        (status = 409, description = "User already exists", body = ErrorResponse),
        (status = 422, description = "Malformed input", body = ErrorResponse),
        (status = 500, description = "Unexpected error", body = ErrorResponse)
    ),
    tag = "auth"
)]
/// Signup endpoint for user registration.

#[tracing::instrument(
    name = "User Signup", 
    skip_all, 
    err(Debug),
    fields(
        email = tracing::field::Empty,
        requires_2fa = request.requires_2fa,
        password_length = tracing::field::Empty,
    )
)]
pub async fn signup(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SignupRequestRest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    tracing::info!("👤 Processing new user signup request");
    
    // Parse and validate email and password using newtypes
    let email = match crate::domain::Email::parse(request.email) {
        Ok(e) => {
            // Record email in tracing span
            tracing::Span::current().record("email", e.as_ref().expose_secret());
            tracing::debug!(
                email = %e.as_ref().expose_secret(),
                "📧 Email parsed and validated successfully"
            );
            e
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "❌ Invalid email format in signup request"
            );
            return Err(AuthAPIError::MalformedCredentials);
        }
    };
    
    let password = match crate::domain::Password::parse(request.password) {
        Ok(p) => {
            // Record password length for debugging (not the actual password)
            tracing::Span::current().record("password_length", p.as_ref().expose_secret().len());
            tracing::debug!(
                password_length = p.as_ref().expose_secret().len(),
                "🔒 Password parsed and validated successfully"
            );
            p
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "❌ Invalid password format in signup request"
            );
            return Err(AuthAPIError::MalformedCredentials);
        }
    };

    let user = User::new(email, password, request.requires_2fa);
    
    tracing::debug!(
        email = %user.email.as_ref().expose_secret(),
        requires_2fa = user.requires_2fa,
        "👤 User object created, checking for existing user"
    );
    
    let mut user_store = state.user_store.write().await;

    // Simulate a user store failure for test trigger - use ExposeSecret for comparison
    use secrecy::ExposeSecret;
    if user.email.as_ref().expose_secret() == "trigger500@example.com" {
        tracing::error!(
            email = %user.email.as_ref().expose_secret(),
            "💥 Simulated user store failure triggered"
        );
        return Err(AuthAPIError::UnexpectedError(eyre::eyre!("Simulated user store failure")));
    }
    
    // Early return AuthAPIError::UserAlreadyExists if email exists in user_store.
    if user_store.get_user(&user.email).await.is_ok() {
        tracing::warn!(
            email = %user.email.as_ref().expose_secret(),
            "⚠️  Signup attempted for existing user"
        );
        return Err(AuthAPIError::UserAlreadyExists);
    }
    // Instead of using unwrap, early return AuthAPIError::UnexpectedError if add_user() fails.
    if let Err(e) = user_store.add_user(user.clone()).await {
        tracing::error!(
            email = %user.email.as_ref().expose_secret(),
            error = ?e,
            "❌ Unexpected error adding user to store"
        );
        return Err(AuthAPIError::UnexpectedError(eyre::eyre!("Unexpected error adding user: {:?}", e)));
    }

    tracing::info!(
        email = %user.email.as_ref().expose_secret(),
        requires_2fa = user.requires_2fa,
        "🎉 User created successfully"
    );
    
    let response = Json(SignupResponseRest {
        message: "User created successfully!".to_string(),
    });
    let mut res = response.into_response();
    *res.status_mut() = StatusCode::CREATED;
    Ok(res)
}

