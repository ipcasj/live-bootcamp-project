use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::data_stores::BannedTokenStore;
use async_trait::async_trait;

pub struct HashsetBannedTokenStore {
    tokens: Arc<RwLock<HashSet<String>>>,
}

impl Default for HashsetBannedTokenStore {
    fn default() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashSet::new())),
        }
    }
}

#[async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn ban_token(&self, token: String) {
        self.tokens.write().await.insert(token);
    }

    async fn is_banned(&self, token: &str) -> bool {
        self.tokens.read().await.contains(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ban_and_check() {
        let store = HashsetBannedTokenStore::default();
        let token = "abc123".to_string();
        assert!(!store.is_banned(&token).await);
        store.ban_token(token.clone()).await;
        assert!(store.is_banned(&token).await);
    }
}
