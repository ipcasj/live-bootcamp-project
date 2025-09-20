use std::sync::Arc;

use bb8_redis::redis;
use bb8_redis::bb8::Pool;
use bb8_redis::RedisConnectionManager;
use async_trait::async_trait;

use crate::{
    domain::data_stores::BannedTokenStore,
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

#[async_trait::async_trait]
#[async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn ban_token(&self, token: String) {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        let key = format!("banned_token:{}", token);
        let _: Result<(), _> = redis::cmd("SETEX")
            .arg(&key)
            .arg(self.config.auth.banned_token_ttl)
            .arg("banned")
            .query_async(&mut *conn)
            .await;
    }

    async fn is_banned(&self, token: &str) -> bool {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("Failed to get Redis connection: {}", e);
                return false;
            }
        };

        let key = format!("banned_token:{}", token);
        let exists: Result<bool, _> = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut *conn)
            .await;
        exists.unwrap_or(false)
    }
}