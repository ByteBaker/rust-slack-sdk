//! In-memory cache-based OAuth state storage

use crate::error::Result;
use crate::oauth::state_store::OAuthStateStore;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;
use uuid::Uuid;

/// State entry with expiration
#[derive(Debug, Clone)]
struct StateEntry {
    expires_at: DateTime<Utc>,
}

/// In-memory OAuth state store
///
/// Stores OAuth state values in memory with automatic expiration.
/// Suitable for development, testing, or single-instance deployments.
///
/// # Warning
///
/// All data is lost when the process terminates. For production use with
/// multiple instances, consider using a shared storage backend.
///
/// # Example
///
/// ```
/// use slack_rs::oauth::state_store::{CacheOAuthStateStore, OAuthStateStore};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = CacheOAuthStateStore::new()
///         .with_expiration_seconds(600); // 10 minutes
///
///     let state = store.issue().await?;
///     let is_valid = store.consume(&state).await?;
///     assert!(is_valid);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CacheOAuthStateStore {
    states: Arc<RwLock<HashMap<String, StateEntry>>>,
    expiration_seconds: i64,
}

impl CacheOAuthStateStore {
    /// Creates a new CacheOAuthStateStore with default 10-minute expiration
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
            expiration_seconds: 600, // 10 minutes
        }
    }

    /// Sets the expiration time for state values
    pub fn with_expiration_seconds(mut self, seconds: i64) -> Self {
        self.expiration_seconds = seconds;
        self
    }

    /// Returns the number of stored states
    pub async fn state_count(&self) -> usize {
        self.states.read().await.len()
    }

    /// Clears all stored states
    pub async fn clear(&self) {
        self.states.write().await.clear();
    }

    /// Removes expired states
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        let mut states = self.states.write().await;

        let before = states.len();
        states.retain(|_, entry| entry.expires_at > now);
        let after = states.len();

        if before != after {
            debug!("Cleaned up {} expired states", before - after);
        }
    }
}

impl Default for CacheOAuthStateStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OAuthStateStore for CacheOAuthStateStore {
    async fn issue(&self) -> Result<String> {
        // Cleanup expired states before issuing new one
        self.cleanup_expired().await;

        let state = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::seconds(self.expiration_seconds);

        self.states
            .write()
            .await
            .insert(state.clone(), StateEntry { expires_at });

        debug!("Issued state {} (expires at {})", state, expires_at);

        Ok(state)
    }

    async fn consume(&self, state: &str) -> Result<bool> {
        // Cleanup expired states
        self.cleanup_expired().await;

        let mut states = self.states.write().await;

        if let Some(entry) = states.remove(state) {
            let now = Utc::now();
            if entry.expires_at > now {
                debug!("Consumed valid state {}", state);
                Ok(true)
            } else {
                debug!("State {} has expired", state);
                Ok(false)
            }
        } else {
            debug!("State {} not found or already consumed", state);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration as TokioDuration};

    #[tokio::test]
    async fn test_issue_and_consume() {
        let store = CacheOAuthStateStore::new();

        let state = store.issue().await.unwrap();
        assert!(!state.is_empty());

        let is_valid = store.consume(&state).await.unwrap();
        assert!(is_valid);

        // Second consumption should fail
        let is_valid = store.consume(&state).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_invalid_state() {
        let store = CacheOAuthStateStore::new();

        let is_valid = store.consume("invalid-state").await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_multiple_states() {
        let store = CacheOAuthStateStore::new();

        let state1 = store.issue().await.unwrap();
        let state2 = store.issue().await.unwrap();

        assert_ne!(state1, state2);

        let is_valid1 = store.consume(&state1).await.unwrap();
        let is_valid2 = store.consume(&state2).await.unwrap();

        assert!(is_valid1);
        assert!(is_valid2);
    }

    #[tokio::test]
    async fn test_expiration() {
        let store = CacheOAuthStateStore::new().with_expiration_seconds(1);

        let state = store.issue().await.unwrap();

        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;

        let is_valid = store.consume(&state).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let store = CacheOAuthStateStore::new().with_expiration_seconds(1);

        let _state1 = store.issue().await.unwrap();
        let _state2 = store.issue().await.unwrap();

        assert_eq!(store.state_count().await, 2);

        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;

        // Cleanup should remove expired states
        store.cleanup_expired().await;
        assert_eq!(store.state_count().await, 0);
    }

    #[tokio::test]
    async fn test_clear() {
        let store = CacheOAuthStateStore::new();

        store.issue().await.unwrap();
        store.issue().await.unwrap();

        assert_eq!(store.state_count().await, 2);

        store.clear().await;
        assert_eq!(store.state_count().await, 0);
    }
}
