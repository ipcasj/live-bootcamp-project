use async_trait::async_trait;

/// Trait for sending 2FA codes via email.
#[async_trait]
pub trait EmailClient: Send + Sync + 'static {
    /// Send a 2FA code to the given email address.
    async fn send_2fa_code(&self, email: &str, code: &str) -> Result<(), EmailClientError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EmailClientError {
    #[error("Failed to send email: {0}")]
    SendError(String),
}
