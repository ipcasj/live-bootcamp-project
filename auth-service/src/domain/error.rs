use color_eyre::eyre::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthAPIError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Incorrect credentials")]
    IncorrectCredentials,
    #[error("Missing token")]
    MissingToken,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl AuthAPIError {
    pub fn code(&self) -> &'static str {
        match self {
            AuthAPIError::UserAlreadyExists => "user_already_exists",
            AuthAPIError::InvalidCredentials => "invalid_credentials",
            AuthAPIError::IncorrectCredentials => "incorrect_credentials",
            AuthAPIError::MissingToken => "missing_token",
            AuthAPIError::InvalidToken => "invalid_token",
            AuthAPIError::UnexpectedError(_) => "internal_server_error",
        }
    }
}
