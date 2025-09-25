use std::sync::Arc;

use bb8_redis::redis::AsyncCommands;
use bb8_redis::bb8::Pool;
use bb8_redis::RedisConnectionManager;
use async_trait::async_trait;
use secrecy::{Secret, ExposeSecret};

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    config::AppConfig,
};

pub type RedisPool = Pool<RedisConnectionManager>;

#[derive(Clone)]
pub struct RedisBannedTokenStore {
    pool: Arc<RedisPool>,
    config: Arc<AppConfig>,
}

impl RedisBannedTokenStore {
    pub fn new(pool: Arc<RedisPool>, config: Arc<AppConfig>) -> Self {
        Self { pool, config }
    }
}

#[async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(skip_all)]
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.pool
            .get()
            .await
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        let _: () = conn
            .set_ex(&get_key(token.expose_secret()), 1, self.config.auth.banned_token_ttl)
            .await
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn contains_token(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        let mut conn = self.pool
            .get()
            .await
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        let exists: bool = conn
            .exists(&get_key(token.expose_secret()))
            .await
            .map_err(|e| BannedTokenStoreError::UnexpectedError(e.into()))?;

        Ok(exists)
    }
}

fn get_key(token: &str) -> String {
    format!("banned_token:{}", token)
}