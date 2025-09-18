use std::any::Any;
use std::error::Error;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use sqlx::PgPool;

use crate::domain::{
    data_stores::{UserStore, UserStoreError},
    user::TwoFAMethod,
    Email, Password, User,
};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(&user.password.as_ref())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref(),
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_pkey") => {
                UserStoreError::UserAlreadyExists
            }
            _ => UserStoreError::UnexpectedError,
        })?;

        Ok(())
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        let row = sqlx::query!(
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
            _ => UserStoreError::UnexpectedError,
        })?;

        let email = Email::parse(&row.email).map_err(|_| UserStoreError::UnexpectedError)?;
        let password = Password::from_hash(row.password_hash);

        Ok(User {
            email,
            password,
            requires_2fa: row.requires_2fa,
            two_fa_method: TwoFAMethod::default(),
        })
    }

    async fn validate_user(&self, email: &Email, password: &str) -> Result<(), UserStoreError> {
        tracing::debug!("PostgreSQL validate_user called for email: {}", email.as_ref());
        
        let row = sqlx::query!(
            "SELECT password_hash FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::debug!("Database query error: {:?}", e);
            match e {
                sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
                _ => UserStoreError::UnexpectedError,
            }
        })?;

        tracing::debug!("Found user, verifying password");
        
        verify_password_hash(&row.password_hash, password)
            .await
            .map_err(|e| {
                tracing::debug!("Password verification failed: {:?}", e);
                UserStoreError::InvalidCredentials
            })?;

        Ok(())
    }

    async fn delete_user(&mut self, email: &Email) -> Result<(), UserStoreError> {
        let result = sqlx::query!("DELETE FROM users WHERE email = $1", email.as_ref())
            .execute(&self.pool)
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }

    async fn update_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(&user.password.as_ref())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let result = sqlx::query!(
            "UPDATE users SET password_hash = $1, requires_2fa = $2 WHERE email = $3",
            password_hash,
            user.requires_2fa,
            user.email.as_ref()
        )
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }

    async fn get_user_settings(&self, email: &Email) -> Result<(bool, TwoFAMethod), UserStoreError> {
        let row = sqlx::query!(
            "SELECT requires_2fa FROM users WHERE email = $1",
            email.as_ref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
            _ => UserStoreError::UnexpectedError,
        })?;

        Ok((row.requires_2fa, TwoFAMethod::default()))
    }

    async fn update_password(&mut self, email: &Email, new_password: Password) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(&new_password.as_ref())
            .await
            .map_err(|_| UserStoreError::UnexpectedError)?;

        let result = sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE email = $2",
            password_hash,
            email.as_ref()
        )
        .execute(&self.pool)
        .await
        .map_err(|_| UserStoreError::UnexpectedError)?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// Helper function to verify if a given password matches an expected hash
// Updated to use spawn_blocking for CPU-intensive operations
async fn verify_password_hash(
    expected_password_hash: &str,
    password_candidate: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing::debug!("Verifying password hash");
    tracing::debug!("Expected hash starts with: {}", &expected_password_hash[..20]);
    tracing::debug!("Password candidate length: {}", password_candidate.len());
    
    let expected_password_hash = expected_password_hash.to_string();
    let password_candidate = password_candidate.to_string();

    tokio::task::spawn_blocking(move || {
        let expected_password_hash: PasswordHash<'_> = PasswordHash::new(&expected_password_hash)?;

        let result = Argon2::default()
            .verify_password(password_candidate.as_bytes(), &expected_password_hash)
            .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) });
            
        match &result {
            Ok(_) => tracing::debug!("Password verification succeeded"),
            Err(e) => tracing::debug!("Password verification failed: {:?}", e),
        }
        
        result
    })
    .await
    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
}

// Helper function to hash passwords before persisting them in the database.
// Updated to use spawn_blocking for CPU-intensive operations
async fn compute_password_hash(password: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let password = password.to_string();

    tokio::task::spawn_blocking(move || {
        let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?,
        )
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
        .to_string();

        Ok(password_hash)
    })
    .await
    .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?
}