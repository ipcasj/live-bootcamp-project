use thiserror::Error;
use anyhow::Error as AnyError;

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("User already exists")] 
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Malformed credentials")]
    MalformedCredentials,
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Unexpected error: {0}")]
    UnexpectedError(AnyError),
}

impl AuthAPIError {
    pub fn code(&self) -> &'static str {
        match self {
            AuthAPIError::UserAlreadyExists => "user_already_exists",
            AuthAPIError::InvalidCredentials => "invalid_credentials",
            AuthAPIError::MalformedCredentials => "malformed_credentials",
            AuthAPIError::IncorrectCredentials => "incorrect_credentials",
            AuthAPIError::UnexpectedError(_) => "internal_server_error",
        }
    }
}
