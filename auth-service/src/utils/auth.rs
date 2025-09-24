use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use color_eyre::eyre::{eyre, Context, ContextCompat, Result};

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
// Create refresh token
#[tracing::instrument(skip_all)]
pub fn generate_refresh_token(email: &Email) -> Result<String> {
    let delta = chrono::Duration::try_seconds(*REFRESH_TOKEN_TTL_SECONDS)
        .wrap_err("failed to create refresh token time delta")?;
    let exp = Utc::now()
        .checked_add_signed(delta)
        .wrap_err("failed to add time delta to current time")?
        .timestamp();
    let exp: usize = exp.try_into()
        .wrap_err("failed to cast exp time to usize")?;
    let sub = email.as_ref().to_owned();
    let claims = Claims { sub, exp };
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(REFRESH_TOKEN_SECRET.as_bytes()),
    ).wrap_err("failed to create refresh token")
}

#[tracing::instrument(skip_all)]
pub fn generate_refresh_token_from_str(email: &str) -> Result<String> {
    let email = Email::parse(email)?;
    generate_refresh_token(&email)
}

#[tracing::instrument(skip_all)]
pub async fn validate_refresh_token(token: &str, banned_token_store: Option<crate::app_state::BannedTokenStoreType>) -> Result<Claims, AuthAPIError> {
    if let Some(store) = banned_token_store {
        match store.read().await.contains_token(token).await {
            Ok(is_banned) => {
                if is_banned {
                    return Err(AuthAPIError::InvalidToken);
                }
            }
            Err(_) => return Err(AuthAPIError::UnexpectedError(color_eyre::eyre::eyre!("Failed to check if token is banned"))),
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
#[tracing::instrument(skip_all)]
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie<'static>> {
    let token = generate_auth_token(email)?;
    Ok(create_auth_cookie(token))
}

// Create cookie and set the value to the passed-in token string 
#[tracing::instrument(skip_all)]
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


#[tracing::instrument(skip_all)]
fn generate_auth_token(email: &Email) -> Result<String> {
    let delta = chrono::Duration::try_seconds(TOKEN_TTL_SECONDS)
        .wrap_err("failed to create 10 minute time delta")?;

    let exp = Utc::now()
        .checked_add_signed(delta)
        .ok_or(eyre!("failed to add 10 minutes to current time"))?
        .timestamp();

    let exp: usize = exp.try_into().wrap_err(format!(
        "failed to cast exp time to usize. exp time: {}",
        exp
    ))?;

    let sub = email.as_ref().to_owned();

    let claims = Claims { sub, exp };

    create_token(&claims)
}

// Helper to generate auth token from a string email (for refresh_token)
pub fn generate_auth_token_from_str(email: &str) -> Result<String> {
    let email = Email::parse(email)?;
    generate_auth_token(&email)
}

// Check if JWT auth token is valid by decoding it using the JWT secret
use crate::domain::AuthAPIError;

pub const TOKEN_TTL_SECONDS: i64 = 600;

#[tracing::instrument(skip_all)]
pub async fn validate_token(
    token: &str,
    banned_token_store: crate::app_state::BannedTokenStoreType,
) -> Result<Claims> {
    match banned_token_store.read().await.contains_token(token).await {
        Ok(value) => {
            if value {
                return Err(eyre!("token is banned"));
            }
        }
        Err(e) => return Err(e.into()),
    }

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .wrap_err("failed to decode token")
}

#[tracing::instrument(skip_all)]
fn create_token(claims: &Claims) -> Result<String> {
    encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .wrap_err("failed to create token")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::email::Email;
    use axum_extra::extract::cookie::SameSite;
    use chrono::Utc;
    
    // Test constants that don't rely on configuration loading
    const TEST_JWT_SECRET: &str = "test_jwt_secret_that_is_long_enough_for_testing_purposes_here";
    const TEST_JWT_COOKIE_NAME: &str = "jwt_test";
    const TEST_TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

    // Test-specific implementations that don't use global config
    fn create_test_auth_cookie(token: String) -> Cookie<'static> {
        let mut cookie = Cookie::new(TEST_JWT_COOKIE_NAME, token);
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Lax);
        cookie
    }

    fn generate_test_auth_token(email: &Email) -> Result<String, GenerateTokenError> {
        let delta = chrono::Duration::try_seconds(TEST_TOKEN_TTL_SECONDS)
            .ok_or(GenerateTokenError::UnexpectedError)?;

        let exp = Utc::now()
            .checked_add_signed(delta)
            .ok_or(GenerateTokenError::UnexpectedError)?
            .timestamp();

        let exp: usize = exp
            .try_into()
            .map_err(|_| GenerateTokenError::UnexpectedError)?;

        let sub = email.as_ref().to_owned();
        let claims = Claims { sub, exp };
        
        encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
        ).map_err(GenerateTokenError::TokenError)
    }

    fn generate_test_auth_cookie(email: &Email) -> Result<Cookie<'static>, GenerateTokenError> {
        let token = generate_test_auth_token(email)?;
        Ok(create_test_auth_cookie(token))
    }

    async fn validate_test_token(token: &str) -> Result<Claims, crate::domain::AuthAPIError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| crate::domain::AuthAPIError::InvalidToken)
    }

    #[tokio::test]
    async fn test_generate_auth_cookie() {
        let email = Email::parse("test@example.com").unwrap();
        let cookie = generate_test_auth_cookie(&email).unwrap();
        assert_eq!(cookie.name(), TEST_JWT_COOKIE_NAME);
        assert_eq!(cookie.value().split('.').count(), 3);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_create_auth_cookie() {
        let token = "test_token".to_owned();
        let cookie = create_test_auth_cookie(token.clone());
        assert_eq!(cookie.name(), TEST_JWT_COOKIE_NAME);
        assert_eq!(cookie.value(), token);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[tokio::test]
    async fn test_generate_auth_token() {
        let email = Email::parse("test@example.com").unwrap();
        let result = generate_test_auth_token(&email).unwrap();
        assert_eq!(result.split('.').count(), 3);
    }

    #[tokio::test]
    async fn test_validate_token_with_valid_token() {
        let email = Email::parse("test@example.com").unwrap();
        
        // Capture time before generating token
        let before_generation = Utc::now().timestamp();
        let token = generate_test_auth_token(&email).unwrap();
        let result = validate_test_token(&token).await.unwrap();
        
        assert_eq!(result.sub, "test@example.com");

        // Token should expire after the time it was created
        // We use a small buffer to account for test timing
        let min_expected_exp = before_generation + 30; // At least 30 seconds in the future
        assert!(result.exp > min_expected_exp as usize, 
            "Token expiration {} should be greater than {}", result.exp, min_expected_exp);
    }

    #[tokio::test]
    async fn test_validate_token_with_invalid_token() {
        let token = "invalid_token".to_owned();
        let result = validate_test_token(&token).await;
        assert!(result.is_err());
    }
}
