use std::any::Any;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use secrecy::{Secret, ExposeSecret};

use sqlx::PgPool;
use color_eyre::{Result, eyre::Context};

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
    #[tracing::instrument(name = "Adding user to PostgreSQL", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        tracing::info!("Attempting to add new user to database");
        
        let password_hash = compute_password_hash(user.password.as_ref().expose_secret().clone())
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Failed to compute password hash");
                UserStoreError::UnexpectedError(e)
            })?;

        let result = sqlx::query!(
            "INSERT INTO users (email, password_hash, requires_2fa) VALUES ($1, $2, $3)",
            user.email.as_ref().expose_secret(),
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.constraint() == Some("users_pkey") => {
                tracing::warn!("User already exists in database");
                UserStoreError::UserAlreadyExists
            }
            _ => {
                tracing::error!(error = ?e, "Unexpected database error during user insertion");
                UserStoreError::UnexpectedError(e.into())
            }
        })?;

        tracing::info!(rows_affected = result.rows_affected(), "User successfully added to database");
        Ok(())
    }

    #[tracing::instrument(name = "Retrieving user from PostgreSQL", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        tracing::debug!("Querying database for user");
        
        let row = sqlx::query!(
            "SELECT email, password_hash, requires_2fa FROM users WHERE email = $1",
            email.as_ref().expose_secret()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                tracing::warn!("User not found in database");
                UserStoreError::UserNotFound
            }
            _ => {
                tracing::error!(error = ?e, "Unexpected database error during user retrieval");
                UserStoreError::UnexpectedError(e.into())
            }
        })?;

        let email = Email::parse(Secret::new(row.email)).map_err(|e| {
            tracing::error!(error = ?e, "Failed to parse email from database");
            UserStoreError::UnexpectedError(e.into())
        })?;
        let password = Password::from_hash(row.password_hash);

        tracing::info!("User successfully retrieved from database");
        Ok(User {
            email,
            password,
            requires_2fa: row.requires_2fa,
            two_fa_method: TwoFAMethod::default(),
        })
    }

    #[tracing::instrument(name = "Validating user credentials in PostgreSQL", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &str,
    ) -> Result<(), UserStoreError> {
        tracing::debug!("Querying database for user password hash");
        
        let result = sqlx::query!(
            "SELECT password_hash FROM users WHERE email = $1",
            email.as_ref().expose_secret()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "Database error during password hash retrieval");
            UserStoreError::UnexpectedError(e.into())
        })?;

        if let Some(row) = result {
            tracing::debug!("User found, verifying password hash");
            verify_password_hash(row.password_hash, password.to_string())
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, "Error during password verification");
                    UserStoreError::UnexpectedError(e)
                })?;

            tracing::info!("Password validation successful");
            Ok(())
        } else {
            tracing::warn!("User not found during validation");
            Err(UserStoreError::UserNotFound)
        }
    }

    async fn delete_user(&mut self, email: &Email) -> Result<(), UserStoreError> {
        let result = sqlx::query!("DELETE FROM users WHERE email = $1", email.as_ref().expose_secret())
            .execute(&self.pool)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }

    async fn update_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(user.password.as_ref().expose_secret().clone())
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e))?;

        let result = sqlx::query!(
            "UPDATE users SET password_hash = $1, requires_2fa = $2 WHERE email = $3",
            password_hash,
            user.requires_2fa,
            user.email.as_ref().expose_secret()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(UserStoreError::UserNotFound);
        }

        Ok(())
    }

    async fn get_user_settings(&self, email: &Email) -> Result<(bool, TwoFAMethod), UserStoreError> {
        let row = sqlx::query!(
            "SELECT requires_2fa FROM users WHERE email = $1",
            email.as_ref().expose_secret()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
            _ => UserStoreError::UnexpectedError(e.into()),
        })?;

        Ok((row.requires_2fa, TwoFAMethod::default()))
    }

    async fn update_password(&mut self, email: &Email, new_password: Password) -> Result<(), UserStoreError> {
        let password_hash = compute_password_hash(new_password.as_ref().expose_secret().clone())
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e))?;

        let result = sqlx::query!(
            "UPDATE users SET password_hash = $1 WHERE email = $2",
            password_hash,
            email.as_ref().expose_secret()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

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
#[tracing::instrument(name = "Verify password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> Result<()> {
    let current_span: tracing::Span = tracing::Span::current();
    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;

            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .wrap_err("failed to verify password hash")
        })
    })
    .await;

    result?
}

// Helper function to hash passwords before persisting them in the database.
// Updated to use spawn_blocking for CPU-intensive operations
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> Result<String> {
    let current_span: tracing::Span = tracing::Span::current();

    let result = tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await;

    result?
}