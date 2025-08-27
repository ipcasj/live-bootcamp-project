//! In-memory HashMap-based user store implementation for the auth-service.
/// In-memory user store using a HashMap.
use std::collections::HashMap;

use crate::domain::{User, UserStore, UserStoreError};
use async_trait::async_trait;


// TODO: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap`` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.

#[derive(Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}


#[async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
        self.users.get(email).cloned().ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) if user.password == password => Ok(()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@example.com".into(), "password".into(), false);
        assert_eq!(store.add_user(user).await, Ok(()));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@example.com".into(), "password".into(), false);
        store.add_user(user.clone()).await.unwrap();
        assert_eq!(store.get_user("test@example.com").await, Ok(user));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("test@example.com".into(), "password".into(), false);
        store.add_user(user).await.unwrap();
        assert_eq!(store.validate_user("test@example.com", "password").await, Ok(()));
        assert_eq!(store.validate_user("test@example.com", "wrongpassword").await, Err(UserStoreError::InvalidCredentials));
    }
}
