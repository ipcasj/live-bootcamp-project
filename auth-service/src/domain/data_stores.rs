use async_trait::async_trait;

#[async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn ban_token(&self, token: String);
    async fn is_banned(&self, token: &str) -> bool;
}
// Data store abstractions and errors for user storage in the auth-service.
/// Trait for async user store implementations.
/// Error type for user store operations.
use super::{User, Email, Password};
use std::any::Any;

#[async_trait::async_trait]
pub trait UserStore: Send + Sync + Any {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError>;
    async fn delete_user(&mut self, email: &Email) -> Result<(), UserStoreError>;
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
