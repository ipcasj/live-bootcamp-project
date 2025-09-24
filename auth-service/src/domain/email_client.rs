use async_trait::async_trait;

/// Trait for sending 2FA codes via email.
#[async_trait]
pub trait EmailClient: Send + Sync + 'static {
    /// Send a 2FA code to the given email address.
    async fn send_2fa_code(&self, email: &str, code: &str) -> color_eyre::Result<()>;
    /// Send an email with custom subject and body.
    async fn send_email(&self, email: &crate::domain::Email, subject: &str, body: &str) -> color_eyre::Result<()>;
}

#[derive(Debug, thiserror::Error)]
pub enum EmailClientError {
    #[error("Failed to send email: {0}")]
    SendError(String),
}
