use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode, u64)>, // u64 = unix timestamp
    failed_attempts: HashMap<Email, (u32, u64)>, // (count, last_failed_ts)
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.codes.insert(email, (login_attempt_id, code, now));
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        self.codes.get(email).cloned().map(|(id, code, _)| (id, code)).ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email).map(|_| ()).ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }

    async fn get_code_with_meta(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode, u64), TwoFACodeStoreError> {
        self.codes.get(email).cloned().ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)
    }

    async fn record_failed_attempt(&mut self, email: &Email) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let entry = self.failed_attempts.entry(email.clone()).or_insert((0, now));
        entry.0 += 1;
        entry.1 = now;
    }

    async fn reset_failed_attempts(&mut self, email: &Email) {
        self.failed_attempts.remove(email);
    }

    async fn get_failed_attempts(&self, email: &Email) -> u32 {
        self.failed_attempts.get(email).map(|(c, _)| *c).unwrap_or(0)
    }
}
