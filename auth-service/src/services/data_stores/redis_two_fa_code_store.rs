use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bb8_redis::{bb8::Pool, redis, RedisConnectionManager};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::{
    domain::{
        data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
        Email,
    },
    config::AppConfig,
};

pub type RedisPool = Pool<RedisConnectionManager>;

#[derive(Clone)]
pub struct RedisTwoFACodeStore {
    pool: Arc<RedisPool>,
    config: Arc<AppConfig>,
}

impl RedisTwoFACodeStore {
    pub fn new(pool: Arc<RedisPool>, config: Arc<AppConfig>) -> Self {
        Self { pool, config }
    }
}

#[async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let mut conn = self.pool.get().await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let key = get_key(&email);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?
            .as_secs();

        let tuple = TwoFATuple(login_attempt_id.as_ref().to_string(), code.as_ref().to_string(), now);
        let serialized = serde_json::to_string(&tuple)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(self.config.auth.two_fa_code_expiration)
            .arg(serialized)
            .query_async(&mut *conn)
            .await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let mut conn = self.pool.get().await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let key = get_key(email);
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        let (login_attempt_id, code, _) = self.get_code_with_meta(email).await?;
        Ok((login_attempt_id, code))
    }

    async fn get_code_with_meta(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode, u64), TwoFACodeStoreError> {
        let mut conn = self.pool.get().await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let key = get_key(email);
        let result: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut *conn)
            .await
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let serialized = result.ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)?;
        
        let tuple: TwoFATuple = serde_json::from_str(&serialized)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        let login_attempt_id = LoginAttemptId::parse(tuple.0)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
        let code = TwoFACode::parse(tuple.1)
            .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

        Ok((login_attempt_id, code, tuple.2))
    }

    async fn record_failed_attempt(&mut self, email: &Email) {
        if let Ok(mut conn) = self.pool.get().await {
            let key = get_failed_attempts_key(email);
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Get current attempts count
            let current_count: u32 = redis::cmd("HGET")
                .arg(&key)
                .arg("count")
                .query_async(&mut *conn)
                .await
                .unwrap_or(0);

            // Increment and update
            let _: Result<(), _> = redis::cmd("HSET")
                .arg(&key)
                .arg("count")
                .arg(current_count + 1)
                .arg("last_failed")
                .arg(now)
                .query_async(&mut *conn)
                .await;

            // Set expiration (1 hour)
            let _: Result<(), _> = redis::cmd("EXPIRE")
                .arg(&key)
                .arg(3600u64)
                .query_async(&mut *conn)
                .await;
        }
    }

    async fn reset_failed_attempts(&mut self, email: &Email) {
        if let Ok(mut conn) = self.pool.get().await {
            let key = get_failed_attempts_key(email);
            let _: Result<(), _> = redis::cmd("DEL")
                .arg(&key)
                .query_async(&mut *conn)
                .await;
        }
    }

    async fn get_failed_attempts(&self, email: &Email) -> u32 {
        if let Ok(mut conn) = self.pool.get().await {
            let key = get_failed_attempts_key(email);
            redis::cmd("HGET")
                .arg(&key)
                .arg("count")
                .query_async(&mut *conn)
                .await
                .unwrap_or(0)
        } else {
            0
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String, pub u64);

const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";
const FAILED_ATTEMPTS_PREFIX: &str = "two_fa_failed:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}

fn get_failed_attempts_key(email: &Email) -> String {
    format!("{}{}", FAILED_ATTEMPTS_PREFIX, email.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Email;

    #[test]
    fn test_get_key() {
        let email = Email::parse("test@example.com").unwrap();
        let key = get_key(&email);
        assert_eq!(key, "two_fa_code:test@example.com");
    }

    #[test]
    fn test_get_failed_attempts_key() {
        let email = Email::parse("test@example.com").unwrap();
        let key = get_failed_attempts_key(&email);
        assert_eq!(key, "two_fa_failed:test@example.com");
    }

    #[test]
    fn test_tuple_serialization() {
        let tuple = TwoFATuple("test_id".to_string(), "123456".to_string(), 1234567890);
        let serialized = serde_json::to_string(&tuple).unwrap();
        let deserialized: TwoFATuple = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(tuple.0, deserialized.0);
        assert_eq!(tuple.1, deserialized.1);
        assert_eq!(tuple.2, deserialized.2);
    }
}