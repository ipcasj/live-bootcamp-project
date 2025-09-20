use std::sync::Arc;
use tokio::sync::RwLock;
use crate::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use crate::services::data_stores::redis_two_fa_code_store::{RedisTwoFACodeStore, RedisPool};
use crate::app_state::TwoFACodeStoreType;

pub fn default_two_fa_code_store() -> TwoFACodeStoreType {
    Arc::new(RwLock::new(HashmapTwoFACodeStore::default()))
}

pub fn redis_two_fa_code_store(redis_pool: Arc<RedisPool>) -> TwoFACodeStoreType {
    Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_pool)))
}
