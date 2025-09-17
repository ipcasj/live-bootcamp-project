#[async_trait::async_trait]
pub trait TwoFACodeStore {
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

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        uuid::Uuid::parse_str(&id)
            .map(|_| LoginAttemptId(id))
            .map_err(|_| "Invalid UUID".to_string())
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId(uuid::Uuid::new_v4().to_string())
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
            Ok(TwoFACode(code))
        } else {
            Err("Invalid 2FA code".to_string())
        }
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let code: u32 = rand::random::<u32>() % 1_000_000;
        TwoFACode(format!("{:06}", code))
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
use async_trait::async_trait;

#[async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn ban_token(&self, token: String);
    async fn is_banned(&self, token: &str) -> bool;
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

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}
