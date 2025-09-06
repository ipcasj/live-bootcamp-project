//! In-memory HashMap-based user store implementation for the auth-service.
/// In-memory user store using a HashMap.
use std::collections::HashMap;
use crate::domain::{Email, Password};

use crate::domain::{User, UserStore, UserStoreError};
use async_trait::async_trait;


// TODO: Create a new struct called `HashmapUserStore` containing a `users` field
// which stores a `HashMap` of email `String`s mapped to `User` objects.
// Derive the `Default` trait for `HashmapUserStore`.

#[derive(Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}


#[async_trait]
impl UserStore for HashmapUserStore {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let key = user.email.clone();
        if self.users.contains_key(&key) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(key, user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.users.get(email).cloned().ok_or(UserStoreError::UserNotFound)
    }

    async fn validate_user(&self, email: &Email, password: &Password) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) if &user.password == password => Ok(()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    async fn delete_user(&mut self, email: &Email) -> Result<(), UserStoreError> {
        if self.users.remove(email).is_some() {
            Ok(())
        } else {
            Err(UserStoreError::UserNotFound)
        }
    }
}

// TODO: Add unit tests for your `HashmapUserStore` implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_delete_user() {
        let mut store = HashmapUserStore::default();
        let email = crate::domain::Email::parse("test@example.com").unwrap();
        let password = crate::domain::Password::parse("password").unwrap();
        let user = User::new(email.clone(), password, false);
        store.add_user(user).await.unwrap();
        assert_eq!(store.delete_user(&email).await, Ok(()));
        assert_eq!(store.get_user(&email).await, Err(UserStoreError::UserNotFound));
    }

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let email = crate::domain::Email::parse("test@example.com").unwrap();
        let password = crate::domain::Password::parse("password").unwrap();
        let user = User::new(email.clone(), password, false);
        assert_eq!(store.add_user(user).await, Ok(()));
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let email = crate::domain::Email::parse("test@example.com").unwrap();
        let password = crate::domain::Password::parse("password").unwrap();
        let user = User::new(email.clone(), password, false);
        store.add_user(user.clone()).await.unwrap();
        assert_eq!(store.get_user(&email).await, Ok(user));
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let email = crate::domain::Email::parse("test@example.com").unwrap();
        let password = crate::domain::Password::parse("password").unwrap();
        let user = User::new(email.clone(), password.clone(), false);
        store.add_user(user).await.unwrap();
        assert_eq!(store.validate_user(&email, &password).await, Ok(()));
        let wrong_password = crate::domain::Password::parse("wrongpassword").unwrap();
        assert_eq!(store.validate_user(&email, &wrong_password).await, Err(UserStoreError::InvalidCredentials));
    }
}
