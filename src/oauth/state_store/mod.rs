//! OAuth state storage for CSRF protection
//!
//! This module provides traits and implementations for managing OAuth state parameters.

use crate::error::Result;
use async_trait::async_trait;

pub mod cache;
pub mod file;

/// Trait for storing and validating OAuth state parameters
///
/// OAuth state parameters are used to prevent CSRF attacks during the OAuth flow.
/// Implementations should:
/// - Generate unique state values with `issue()`
/// - Validate and consume state values with `consume()`
/// - Support TTL (time-to-live) for automatic cleanup
///
/// # Example
///
/// ```no_run
/// use slack_rs::oauth::state_store::{CacheOAuthStateStore, OAuthStateStore};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = CacheOAuthStateStore::new();
///
///     // Issue a new state value
///     let state = store.issue().await?;
///
///     // Later, validate and consume the state
///     let is_valid = store.consume(&state).await?;
///     assert!(is_valid);
///
///     // Second consumption should fail
///     let is_valid = store.consume(&state).await?;
///     assert!(!is_valid);
///
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait OAuthStateStore: Send + Sync {
    /// Issues a new OAuth state value
    ///
    /// Generates and stores a unique state string that can be validated later.
    /// The state should be cryptographically random and unique.
    ///
    /// # Returns
    ///
    /// A unique state string to include in the OAuth authorization URL
    async fn issue(&self) -> Result<String>;

    /// Consumes and validates an OAuth state value
    ///
    /// Checks if the state exists and is valid, then removes it to prevent reuse.
    /// This is a one-time operation - subsequent calls with the same state should fail.
    ///
    /// # Arguments
    ///
    /// * `state` - The state string to validate
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the state is valid and consumed
    /// `Ok(false)` if the state is invalid or already consumed
    async fn consume(&self, state: &str) -> Result<bool>;
}
