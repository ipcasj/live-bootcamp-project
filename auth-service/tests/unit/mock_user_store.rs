//! Mock implementation of the UserStore trait for unit testing handlers.
use async_trait::async_trait;
use auth_service::domain::{User, UserStore, UserStoreError};
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
pub struct MockUserStore {
    pub users: Arc<Mutex<Vec<User>>>,
    pub fail_add: bool,
    pub fail_get: bool,
    pub fail_validate: bool,
}

#[async_trait]
impl UserStore for MockUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.fail_add {
            return Err(UserStoreError::UnexpectedError);
        }
        let mut users = self.users.lock().unwrap();
        if users.iter().any(|u| u.email.as_ref() == user.email.as_ref()) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            users.push(user);
            Ok(())
        }
    }
    async fn get_user(&self, email: &auth_service::domain::Email) -> Result<User, UserStoreError> {
        if self.fail_get {
            return Err(UserStoreError::UnexpectedError);
        }
        let users = self.users.lock().unwrap();
        users.iter().find(|u| &u.email == email).cloned().ok_or(UserStoreError::UserNotFound)
    }
    async fn validate_user(&self, email: &auth_service::domain::Email, password: &auth_service::domain::Password) -> Result<(), UserStoreError> {
        if self.fail_validate {
            return Err(UserStoreError::UnexpectedError);
        }
        let users = self.users.lock().unwrap();
        match users.iter().find(|u| &u.email == email) {
            Some(u) if &u.password == password => Ok(()),
            Some(_) => Err(UserStoreError::InvalidCredentials),
            None => Err(UserStoreError::UserNotFound),
        }
    }
}
