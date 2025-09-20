use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::sync::Arc;

use crate::domain::email::Email;
use crate::config::AppConfig;

// Lazy-loaded configuration for legacy compatibility
static LEGACY_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig::load().expect("Failed to load configuration for legacy auth functions")
});

// Legacy constants using configuration
pub static JWT_COOKIE_NAME: Lazy<String> = Lazy::new(|| LEGACY_CONFIG.auth.jwt_cookie_name.clone());
static JWT_SECRET: Lazy<String> = Lazy::new(|| LEGACY_CONFIG.auth.jwt_secret.clone());
static REFRESH_TOKEN_SECRET: Lazy<String> = Lazy::new(|| LEGACY_CONFIG.auth.refresh_token_secret.clone());
static REFRESH_TOKEN_TTL_SECONDS: Lazy<i64> = Lazy::new(|| LEGACY_CONFIG.auth.refresh_token_expiration as i64);
static TOKEN_TTL_SECONDS: Lazy<i64> = Lazy::new(|| LEGACY_CONFIG.auth.jwt_expiration as i64);
// Create refresh token
pub fn generate_refresh_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(*REFRESH_TOKEN_TTL_SECONDS)
        .ok_or(GenerateTokenError::UnexpectedError)?;
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();
    let exp: usize = exp.try_into().map_err(|_| GenerateTokenError::UnexpectedError)?;
    let sub = email.as_ref().to_owned();
    let claims = Claims { sub, exp };
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(REFRESH_TOKEN_SECRET.as_bytes()),
    ).map_err(GenerateTokenError::TokenError)
}

pub fn generate_refresh_token_from_str(email: &str) -> Result<String, GenerateTokenError> {
    let email = Email::parse(email).map_err(|_| GenerateTokenError::UnexpectedError)?;
    generate_refresh_token(&email)
}

pub async fn validate_refresh_token(token: &str, banned_token_store: Option<Arc<dyn BannedTokenStore>>) -> Result<Claims, AuthAPIError> {
    if let Some(store) = banned_token_store {
        if store.is_banned(token).await {
            return Err(AuthAPIError::BannedToken);
        }
    }
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(REFRESH_TOKEN_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthAPIError::InvalidToken)
}

// Create cookie with a new JWT auth token
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

// Create cookie and set the value to the passed-in token string 
fn create_auth_cookie(token: String) -> Cookie<'static> {
    let mut cookie = Cookie::new(&*JWT_COOKIE_NAME, token);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::Lax);
    cookie
}

#[derive(Debug)]
pub enum GenerateTokenError {
    TokenError(jsonwebtoken::errors::Error),
    UnexpectedError,
}


// Create JWT auth token
fn generate_auth_token(email: &Email) -> Result<String, GenerateTokenError> {
    let delta = chrono::Duration::try_seconds(*TOKEN_TTL_SECONDS)
        .ok_or(GenerateTokenError::UnexpectedError)?;

    // Create JWT expiration time
    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(GenerateTokenError::UnexpectedError)?
        .timestamp();

    // Cast exp to a usize, which is what Claims expects
    let exp: usize = exp
        .try_into()
        .map_err(|_| GenerateTokenError::UnexpectedError)?;

    let sub = email.as_ref().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims).map_err(GenerateTokenError::TokenError)
}

// Helper to generate auth token from a string email (for refresh_token)
pub fn generate_auth_token_from_str(email: &str) -> Result<String, GenerateTokenError> {
    let email = Email::parse(email).map_err(|_| GenerateTokenError::UnexpectedError)?;
    generate_auth_token(&email)
}

// Check if JWT auth token is valid by decoding it using the JWT secret
use crate::domain::data_stores::BannedTokenStore;
use crate::domain::AuthAPIError;

pub async fn validate_token(token: &str, banned_token_store: Option<Arc<dyn BannedTokenStore>>) -> Result<Claims, AuthAPIError> {
    if let Some(store) = banned_token_store {
        if store.is_banned(token).await {
            return Err(AuthAPIError::BannedToken);
        }
    }
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthAPIError::InvalidToken)
}

// Create JWT auth token by encoding claims using the JWT secret
fn create_token(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_auth_cookie() {
    let email = Email::parse("test@example.com").unwrap();
        let cookie = generate_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), &*JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_auth_cookie(token.clone());
        assert_eq!(cookie.name(), &*JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
    let email = Email::parse("test@example.com").unwrap();
        let result = generate_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
    let email = Email::parse("test@example.com").unwrap();
        let token = generate_auth_token(&email).unwrap();
    let result = validate_token(&token, None).await.unwrap();
        assert_eq!(result.sub, "test@example.com");

        let exp = Utc::now()
            .checked_add_signed(chrono::Duration::try_minutes(9).expect("valid duration"))
            .expect("valid timestamp")
            .timestamp();

        assert!(result.exp > exp as usize);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
    let result = validate_token(&token, None).await;
        assert!(result.is_err());
    }
}
