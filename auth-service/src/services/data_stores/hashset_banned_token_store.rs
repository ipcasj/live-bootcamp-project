use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use secrecy::{Secret, ExposeSecret};
use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
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
    async fn add_token(&mut self, token: Secret<String>) -> Result<(), BannedTokenStoreError> {
        let mut tokens = self.tokens.write().await;
        tokens.insert(token.expose_secret().clone());
        Ok(())
    }

    async fn contains_token(&self, token: &Secret<String>) -> Result<bool, BannedTokenStoreError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.contains(token.expose_secret()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_check() {
        let mut store = HashsetBannedTokenStore::default();
        let token = Secret::new("abc123".to_string());
        assert!(!store.contains_token(&token).await.unwrap());
        store.add_token(token.clone()).await.unwrap();
        assert!(store.contains_token(&token).await.unwrap());
    }
}
