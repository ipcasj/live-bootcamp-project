use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::config::AppConfig;
use crate::domain::email::Email;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user email)
    pub exp: usize,  // Expiration time
}

/// Errors that can occur during token generation
#[derive(Debug, thiserror::Error)]
pub enum GenerateTokenError {
    #[error("JWT token error: {0}")]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Unexpected error occurred")]
    UnexpectedError,
}

/// Errors that can occur during token validation
#[derive(Debug, thiserror::Error)]
pub enum ValidateTokenError {
    #[error("JWT token error: {0}")]
    TokenError(#[from] jsonwebtoken::errors::Error),
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Unexpected error occurred")]
    UnexpectedError,
}

/// Configuration-aware JWT authentication utilities
pub struct AuthService {
    config: Arc<AppConfig>,
}

impl AuthService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    /// Generate a JWT token for the given email
    pub fn generate_jwt_token(&self, email: &Email) -> Result<String, GenerateTokenError> {
        let delta = Duration::try_seconds(self.config.auth.jwt_expiration as i64)
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
            &EncodingKey::from_secret(self.config.auth.jwt_secret.as_bytes()),
        ).map_err(GenerateTokenError::TokenError)
    }

    /// Generate a refresh token for the given email
    pub fn generate_refresh_token(&self, email: &Email) -> Result<String, GenerateTokenError> {
        let delta = Duration::try_seconds(self.config.auth.refresh_token_expiration as i64)
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
            &EncodingKey::from_secret(self.config.auth.refresh_token_secret.as_bytes()),
        ).map_err(GenerateTokenError::TokenError)
    }

    /// Generate refresh token from string email
    pub fn generate_refresh_token_from_str(&self, email: &str) -> Result<String, GenerateTokenError> {
        let email = Email::parse(email).map_err(|_| GenerateTokenError::UnexpectedError)?;
        self.generate_refresh_token(&email)
    }

    /// Validate a JWT token and return the claims
    pub fn validate_jwt_token(&self, token: &str) -> Result<Claims, ValidateTokenError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.auth.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(ValidateTokenError::TokenError)
    }

    /// Validate a refresh token and return the claims
    pub fn validate_refresh_token(&self, token: &str) -> Result<Claims, ValidateTokenError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.auth.refresh_token_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(ValidateTokenError::TokenError)
    }

    /// Generate a JWT cookie for the given token
    pub fn generate_jwt_cookie(&self, token: String) -> Cookie<'static> {
        Cookie::build(self.config.auth.jwt_cookie_name.clone(), token)
            .path("/")
            .max_age(time::Duration::seconds(self.config.auth.jwt_expiration as i64))
            .same_site(SameSite::Lax)
            .http_only(true)
            .finish()
    }

    /// Get the JWT cookie name from configuration
    pub fn jwt_cookie_name(&self) -> &str {
        &self.config.auth.jwt_cookie_name
    }

    /// Get 2FA code expiration time
    pub fn two_fa_code_expiration(&self) -> u64 {
        self.config.auth.two_fa_code_expiration
    }
}

// Legacy function wrappers for backward compatibility during migration
// These should be removed once all code is migrated to use AuthService

use once_cell::sync::Lazy;

static LEGACY_CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    AppConfig::load().expect("Failed to load configuration for legacy functions")
});

static LEGACY_AUTH_SERVICE: Lazy<AuthService> = Lazy::new(|| {
    AuthService::new(Arc::new(LEGACY_CONFIG.clone()))
});

/// Legacy function - use AuthService::generate_jwt_token instead
pub fn generate_auth_cookie(email: &Email) -> Result<Cookie, GenerateTokenError> {
    let token = LEGACY_AUTH_SERVICE.generate_jwt_token(email)?;
    Ok(LEGACY_AUTH_SERVICE.generate_jwt_cookie(token))
}

/// Legacy function - use AuthService::validate_jwt_token instead
pub fn validate_token(token: &str) -> Result<Claims, ValidateTokenError> {
    LEGACY_AUTH_SERVICE.validate_jwt_token(token)
}

/// Legacy function - use AuthService::generate_refresh_token instead
pub fn generate_refresh_token(email: &Email) -> Result<String, GenerateTokenError> {
    LEGACY_AUTH_SERVICE.generate_refresh_token(email)
}

/// Legacy function - use AuthService::generate_refresh_token_from_str instead
pub fn generate_refresh_token_from_str(email: &str) -> Result<String, GenerateTokenError> {
    LEGACY_AUTH_SERVICE.generate_refresh_token_from_str(email)
}

/// Legacy function - use AuthService::validate_refresh_token instead
pub fn validate_refresh_token(token: &str) -> Result<Claims, ValidateTokenError> {
    LEGACY_AUTH_SERVICE.validate_refresh_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_config() -> AppConfig {
        env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
        env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_validation");
        AppConfig::load().expect("Failed to load test config")
    }

    #[test]
    fn test_auth_service_jwt_generation() {
        let config = Arc::new(setup_test_config());
        let auth_service = AuthService::new(config);
        let email = Email::parse("test@example.com").unwrap();
        
        let token = auth_service.generate_jwt_token(&email).expect("Should generate token");
        assert!(!token.is_empty());
        
        let claims = auth_service.validate_jwt_token(&token).expect("Should validate token");
        assert_eq!(claims.sub, "test@example.com");
    }

    #[test]
    fn test_auth_service_refresh_token() {
        let config = Arc::new(setup_test_config());
        let auth_service = AuthService::new(config);
        let email = Email::parse("test@example.com").unwrap();
        
        let token = auth_service.generate_refresh_token(&email).expect("Should generate refresh token");
        assert!(!token.is_empty());
        
        let claims = auth_service.validate_refresh_token(&token).expect("Should validate refresh token");
        assert_eq!(claims.sub, "test@example.com");
    }

    #[test]
    fn test_jwt_cookie_generation() {
        let config = Arc::new(setup_test_config());
        let auth_service = AuthService::new(config);
        
        let cookie = auth_service.generate_jwt_cookie("test_token".to_string());
        assert_eq!(cookie.name(), &auth_service.config.auth.jwt_cookie_name);
        assert_eq!(cookie.value(), "test_token");
        assert!(cookie.http_only().unwrap_or(false));
    }
}