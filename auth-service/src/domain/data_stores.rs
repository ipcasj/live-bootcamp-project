#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;

    // New: get code with timestamp for expiration
    async fn get_code_with_meta(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode, u64), TwoFACodeStoreError>;

    // New: failed attempt tracking
    async fn record_failed_attempt(&mut self, email: &Email);
    async fn reset_failed_attempts(&mut self, email: &Email);
    async fn get_failed_attempts(&self, email: &Email) -> u32;
}

#[derive(Debug, thiserror::Error)]
pub enum TwoFACodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] color_eyre::Report),
}

impl PartialEq for TwoFACodeStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::LoginAttemptIdNotFound, Self::LoginAttemptIdNotFound)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

#[derive(Debug, Clone)]
pub struct LoginAttemptId(secrecy::Secret<String>);

impl PartialEq for LoginAttemptId {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret;
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl LoginAttemptId {
    pub fn parse(id: String) -> color_eyre::Result<Self> {
        let parsed_id = uuid::Uuid::parse_str(&id)
            .wrap_err("Invalid login attempt id")?;
        Ok(Self(secrecy::Secret::new(parsed_id.to_string())))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(secrecy::Secret::new(uuid::Uuid::new_v4().to_string()))
    }
}

impl AsRef<secrecy::Secret<String>> for LoginAttemptId {
    fn as_ref(&self) -> &secrecy::Secret<String> {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct TwoFACode(secrecy::Secret<String>);

impl PartialEq for TwoFACode {
    fn eq(&self, other: &Self) -> bool {
        use secrecy::ExposeSecret;
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl TwoFACode {
    pub fn parse(code: String) -> color_eyre::Result<Self> {
        // 2FA codes should be exactly 6 digits
        if code.len() != 6 {
            return Err(color_eyre::eyre::eyre!("2FA code must be exactly 6 digits"));
        }
        
        // Check if all characters are digits
        if !code.chars().all(|c| c.is_ascii_digit()) {
            return Err(color_eyre::eyre::eyre!("2FA code must contain only digits"));
        }

        Ok(Self(secrecy::Secret::new(code)))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let code: u32 = rand::random::<u32>() % 1_000_000;
        TwoFACode(secrecy::Secret::new(format!("{:06}", code)))
    }
}

impl AsRef<secrecy::Secret<String>> for TwoFACode {
    fn as_ref(&self) -> &secrecy::Secret<String> {
        &self.0
    }
}
use async_trait::async_trait;
use color_eyre::eyre::Context;
use secrecy::Secret;

#[async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError>;
}

#[derive(Debug, thiserror::Error)]
pub enum BannedTokenStoreError {
    #[error("Unexpected error")]
    UnexpectedError(#[source] color_eyre::Report),
}
// Data store abstractions and errors for user storage in the auth-service.
/// Trait for async user store implementations.
/// Error type for user store operations.
use super::{User, Email};
use std::any::Any;

#[async_trait::async_trait]
pub trait UserStore: Send + Sync + Any {
    async fn update_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user_settings(&self, email: &Email) -> Result<(bool, crate::domain::user::TwoFAMethod), UserStoreError>;
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError>;
    async fn delete_user(&mut self, email: &Email) -> Result<(), UserStoreError>;
    async fn update_password(&mut self, email: &Email, new_password: crate::domain::Password) -> Result<(), UserStoreError>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Debug, thiserror::Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] color_eyre::Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UserStoreError::UserAlreadyExists, UserStoreError::UserAlreadyExists) => true,
            (UserStoreError::UserNotFound, UserStoreError::UserNotFound) => true,
            (UserStoreError::InvalidCredentials, UserStoreError::InvalidCredentials) => true,
            (UserStoreError::UnexpectedError(_), UserStoreError::UnexpectedError(_)) => true,
            _ => false,
        }
    }
}
