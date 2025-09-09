use async_trait::async_trait;
use crate::domain::email_client::{EmailClient, EmailClientError};

/// Mock email client that logs 2FA codes to stdout for testing/development.
pub struct MockEmailClient;

#[async_trait]
impl EmailClient for MockEmailClient {
    async fn send_2fa_code(&self, email: &str, code: &str) -> Result<(), EmailClientError> {
        println!("[MOCK EMAIL] To: {email}, 2FA Code: {code}");
        Ok(())
    }
}
