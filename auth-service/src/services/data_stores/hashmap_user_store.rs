// In-memory HashMap-based user store implementation for the auth-service.
/// In-memory user store using a HashMap.
use std::collections::HashMap;
use crate::domain::Email;

use crate::domain::{User, UserStore, UserStoreError};
use async_trait::async_trait;

#[derive(Default)]
pub struct HashmapUserStore {
    pub users: HashMap<Email, User>,
}

#[async_trait]
impl UserStore for HashmapUserStore {
    async fn update_password(&mut self, email: &crate::domain::Email, new_password: crate::domain::Password) -> Result<(), UserStoreError> {
        if let Some(user) = self.users.get_mut(email) {
            user.password = new_password;
            Ok(())
        } else {
            Err(UserStoreError::UserNotFound)
        }
    }
    async fn update_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let key = user.email.clone();
        if self.users.contains_key(&key) {
            self.users.insert(key, user);
            Ok(())
        } else {
            Err(UserStoreError::UserNotFound)
        }
    }

    async fn get_user_settings(&self, email: &crate::domain::Email) -> Result<(bool, crate::domain::user::TwoFAMethod), UserStoreError> {
        self.users.get(email)
            .map(|u| (u.requires_2fa, u.two_fa_method.clone()))
            .ok_or(UserStoreError::UserNotFound)
    }
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

    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError> {
        match self.users.get(email) {
            Some(user) => {
                // For hashmap store, use simple string comparison 
                // since Password now stores plaintext
                if user.password.as_ref() == password {
                    Ok(())
                } else {
                    Err(UserStoreError::InvalidCredentials)
                }
            },
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
        let password_plain = "password";
        let password = crate::domain::Password::parse(password_plain).unwrap();
        let user = User::new(email.clone(), password, false);
        store.add_user(user).await.unwrap();
        // Validate with correct password
        assert_eq!(store.validate_user(&email, password_plain).await, Ok(()));
        // Validate with wrong password
    assert_eq!(store.validate_user(&email, "wrongpassword").await, Err(UserStoreError::InvalidCredentials));
    }
}
