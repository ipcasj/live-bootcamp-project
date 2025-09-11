use axum::{extract::FromRequestParts, http::{request::Parts, StatusCode}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub email: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Demo: extract email from header 'x-user-email'. In production, use session/JWT.
        if let Some(email) = parts.headers.get("x-user-email") {
            let email = email.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid email header".to_string()))?.to_string();
            Ok(AuthenticatedUser { email })
        } else {
            Err((StatusCode::UNAUTHORIZED, "Missing authentication header".to_string()))
        }
    }
}
