use std::sync::Arc;
use tokio::sync::RwLock;
use crate::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;

pub fn default_two_fa_code_store() -> Arc<RwLock<HashmapTwoFACodeStore>> {
    Arc::new(RwLock::new(HashmapTwoFACodeStore::default()))
}
