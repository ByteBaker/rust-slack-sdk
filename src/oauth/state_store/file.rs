#![allow(missing_debug_implementations)]
//! File-based OAuth state storage

use crate::error::{Error, Result};
use crate::oauth::state_store::OAuthStateStore;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, warn};
use uuid::Uuid;

/// State entry with expiration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateEntry {
    expires_at: DateTime<Utc>,
}

/// File-based OAuth state store
///
/// Stores OAuth state values as individual JSON files in a directory.
/// Each state gets its own file named `{state}.json`.
///
/// # Example
///
/// ```
/// use slack_rs::oauth::state_store::{FileOAuthStateStore, OAuthStateStore};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = FileOAuthStateStore::new("/tmp/oauth-states");
///
///     let state = store.issue().await?;
///     let is_valid = store.consume(&state).await?;
///     assert!(is_valid);
///
///     Ok(())
/// }
/// ```
pub struct FileOAuthStateStore {
    base_dir: PathBuf,
    expiration_seconds: i64,
}

impl FileOAuthStateStore {
    /// Creates a new FileOAuthStateStore
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Directory where state files will be stored
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            expiration_seconds: 600, // 10 minutes
        }
    }

    /// Sets the expiration time for state values
    pub fn with_expiration_seconds(mut self, seconds: i64) -> Self {
        self.expiration_seconds = seconds;
        self
    }

    /// Ensures the base directory exists
    async fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.base_dir).await.map_err(|e| {
            Error::storage_error(format!(
                "Failed to create directory {:?}: {}",
                self.base_dir, e
            ))
        })
    }

    /// Gets the path for a state file
    fn get_state_path(&self, state: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", state))
    }

    /// Writes a state entry to disk
    async fn write_state(&self, state: &str, entry: &StateEntry) -> Result<()> {
        let path = self.get_state_path(state);
        let data = serde_json::to_vec(entry)
            .map_err(|e| Error::serialization_error(format!("Failed to serialize state: {}", e)))?;

        fs::write(&path, data).await.map_err(|e| {
            Error::storage_error(format!("Failed to write state file {:?}: {}", path, e))
        })
    }

    /// Reads a state entry from disk
    async fn read_state(&self, state: &str) -> Result<Option<StateEntry>> {
        let path = self.get_state_path(state);

        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read(&path).await.map_err(|e| {
            Error::storage_error(format!("Failed to read state file {:?}: {}", path, e))
        })?;

        let entry: StateEntry = serde_json::from_slice(&data).map_err(|e| {
            Error::deserialization_error(format!("Failed to deserialize state: {}", e))
        })?;

        Ok(Some(entry))
    }

    /// Deletes a state file
    async fn delete_state(&self, state: &str) -> Result<()> {
        let path = self.get_state_path(state);

        if path.exists() {
            fs::remove_file(&path).await.map_err(|e| {
                Error::storage_error(format!("Failed to delete state file {:?}: {}", path, e))
            })?;
        }

        Ok(())
    }

    /// Cleans up expired state files
    pub async fn cleanup_expired(&self) -> Result<()> {
        if !self.base_dir.exists() {
            return Ok(());
        }

        let now = Utc::now();
        let mut entries = fs::read_dir(&self.base_dir).await.map_err(|e| {
            Error::storage_error(format!(
                "Failed to read directory {:?}: {}",
                self.base_dir, e
            ))
        })?;

        let mut cleaned = 0;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::storage_error(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".json") {
                    if let Ok(data) = fs::read(&path).await {
                        if let Ok(state_entry) = serde_json::from_slice::<StateEntry>(&data) {
                            if state_entry.expires_at <= now {
                                if let Err(e) = fs::remove_file(&path).await {
                                    warn!("Failed to delete expired state file {:?}: {}", path, e);
                                } else {
                                    cleaned += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        if cleaned > 0 {
            debug!("Cleaned up {} expired state files", cleaned);
        }

        Ok(())
    }
}

#[async_trait]
impl OAuthStateStore for FileOAuthStateStore {
    async fn issue(&self) -> Result<String> {
        self.ensure_dir().await?;

        // Cleanup expired states
        if let Err(e) = self.cleanup_expired().await {
            warn!("Failed to cleanup expired states: {}", e);
        }

        let state = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::seconds(self.expiration_seconds);

        let entry = StateEntry { expires_at };
        self.write_state(&state, &entry).await?;

        debug!("Issued state {} (expires at {})", state, expires_at);

        Ok(state)
    }

    async fn consume(&self, state: &str) -> Result<bool> {
        // Cleanup expired states
        if let Err(e) = self.cleanup_expired().await {
            warn!("Failed to cleanup expired states: {}", e);
        }

        if let Some(entry) = self.read_state(state).await? {
            // Delete the state file first
            self.delete_state(state).await?;

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
    use tempfile::TempDir;
    use tokio::time::{sleep, Duration as TokioDuration};

    async fn create_test_store() -> (FileOAuthStateStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = FileOAuthStateStore::new(temp_dir.path());
        (store, temp_dir)
    }

    #[tokio::test]
    async fn test_issue_and_consume() {
        let (store, _temp_dir) = create_test_store().await;

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
        let (store, _temp_dir) = create_test_store().await;

        let is_valid = store.consume("invalid-state").await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_multiple_states() {
        let (store, _temp_dir) = create_test_store().await;

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
        let temp_dir = TempDir::new().unwrap();
        let store = FileOAuthStateStore::new(temp_dir.path()).with_expiration_seconds(1);

        let state = store.issue().await.unwrap();

        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;

        let is_valid = store.consume(&state).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let store = FileOAuthStateStore::new(temp_dir.path()).with_expiration_seconds(1);

        let _state1 = store.issue().await.unwrap();
        let _state2 = store.issue().await.unwrap();

        // Wait for expiration
        sleep(TokioDuration::from_secs(2)).await;

        // Cleanup should remove expired states
        store.cleanup_expired().await.unwrap();

        // Check that files are gone
        let mut entries = fs::read_dir(temp_dir.path()).await.unwrap();
        let mut count = 0;
        while let Some(_) = entries.next_entry().await.unwrap() {
            count += 1;
        }
        assert_eq!(count, 0);
    }
}
