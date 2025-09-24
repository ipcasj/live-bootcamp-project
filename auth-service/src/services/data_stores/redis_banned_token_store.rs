use std::sync::Arc;

use bb8_redis::redis;
use bb8_redis::bb8::Pool;
use bb8_redis::RedisConnectionManager;
use async_trait::async_trait;
use color_eyre::eyre::Context;

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
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
        let token_key = get_key(token.as_str());

        let value = true;

        let ttl: u64 = self.config.auth.banned_token_ttl
            .try_into()
            .wrap_err("failed to cast banned_token_ttl to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        let mut conn = self.pool.get().await
            .wrap_err("failed to get Redis connection")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        let _: () = redis::cmd("SETEX")
            .arg(&token_key)
            .arg(ttl)
            .arg(value)
            .query_async(&mut *conn)
            .await
            .wrap_err("failed to set banned token in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let token_key = get_key(token);

        let mut conn = self.pool.get().await
            .wrap_err("failed to get Redis connection")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        let is_banned: bool = redis::cmd("EXISTS")
            .arg(&token_key)
            .query_async(&mut *conn)
            .await
            .wrap_err("failed to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;

        Ok(is_banned)
    }
}

fn get_key(token: &str) -> String {
    format!("banned_token:{}", token)
}