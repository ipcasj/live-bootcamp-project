use async_trait::async_trait;
use crate::domain::email_client::EmailClient;

/// Mock email client that logs 2FA codes to stdout for testing/development.
pub struct MockEmailClient;

#[async_trait]
impl EmailClient for MockEmailClient {
    async fn send_2fa_code(&self, email: &str, code: &str) -> color_eyre::Result<()> {
        tracing::debug!("[MOCK EMAIL] To: {email}, 2FA Code: {code}");
        Ok(())
    }

    async fn send_email(&self, email: &crate::domain::Email, subject: &str, body: &str) -> color_eyre::Result<()> {
        tracing::debug!("[MOCK EMAIL] To: {}, Subject: {}, Body: {}", email.as_ref(), subject, body);
        Ok(())
    }
}
