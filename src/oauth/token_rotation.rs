#![allow(missing_debug_implementations)]
//! Token rotation support for OAuth installations
//!
//! Handles automatic token refresh when token rotation is enabled.

use crate::error::{Error, Result};
use crate::oauth::installation_store::InstallationStore;
use crate::oauth::models::Installation;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

/// Token refresh response from Slack API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResponse {
    pub ok: bool,
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub team_id: Option<String>,
    pub user_id: Option<String>,
    pub error: Option<String>,
}

/// Token rotator for managing token refresh
///
/// Automatically refreshes OAuth tokens before they expire and updates
/// the installation store with new tokens.
///
/// # Example
///
/// ```no_run
/// use slack_rs::oauth::token_rotation::TokenRotator;
/// use slack_rs::oauth::installation_store::{CacheInstallationStore, InstallationStore};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = Arc::new(CacheInstallationStore::new());
///     let rotator = TokenRotator::new(store, "client_id", "client_secret");
///
///     // Rotator can check and refresh tokens automatically
///     Ok(())
/// }
/// ```
pub struct TokenRotator {
    http_client: Client,
    store: Arc<dyn InstallationStore>,
    client_id: String,
    client_secret: String,
}

impl TokenRotator {
    /// Creates a new TokenRotator
    ///
    /// # Arguments
    ///
    /// * `store` - Installation store for updating tokens
    /// * `client_id` - OAuth client ID
    /// * `client_secret` - OAuth client secret
    pub fn new(
        store: Arc<dyn InstallationStore>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            store,
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        }
    }

    /// Checks if a token needs rotation
    ///
    /// Returns true if the token expires within the threshold (default 2 hours).
    pub fn needs_rotation(&self, token_expires_at: Option<i64>, threshold_seconds: i64) -> bool {
        if let Some(expires_at) = token_expires_at {
            let now = Utc::now().timestamp();
            let time_until_expiry = expires_at - now;
            time_until_expiry < threshold_seconds
        } else {
            false
        }
    }

    /// Rotates a bot token
    ///
    /// Refreshes the bot token using the refresh token and updates the installation store.
    ///
    /// # Arguments
    ///
    /// * `installation` - The installation containing the refresh token
    ///
    /// # Returns
    ///
    /// The updated installation with new tokens
    pub async fn rotate_bot_token(&self, mut installation: Installation) -> Result<Installation> {
        let refresh_token = installation
            .bot_refresh_token
            .as_ref()
            .ok_or_else(|| Error::invalid_input("No bot refresh token available"))?;

        debug!(
            "Rotating bot token for team {}",
            installation.team_id.as_deref().unwrap_or("none")
        );

        let response = self.refresh_token(refresh_token).await?;

        if !response.ok {
            return Err(Error::api_error(format!(
                "Token refresh failed: {}",
                response
                    .error
                    .unwrap_or_else(|| "unknown error".to_string())
            )));
        }

        // Update installation with new token
        if let Some(access_token) = response.access_token {
            installation.bot_token = Some(access_token);
        }

        if let Some(refresh_token) = response.refresh_token {
            installation.bot_refresh_token = Some(refresh_token);
        }

        if let Some(expires_in) = response.expires_in {
            installation.bot_token_expires_at = Some(Utc::now().timestamp() + expires_in);
        }

        // Save updated installation
        self.store.save(installation.clone()).await?;

        debug!("Bot token rotated successfully");

        Ok(installation)
    }

    /// Rotates a user token
    ///
    /// Refreshes the user token using the refresh token and updates the installation store.
    ///
    /// # Arguments
    ///
    /// * `installation` - The installation containing the refresh token
    ///
    /// # Returns
    ///
    /// The updated installation with new tokens
    pub async fn rotate_user_token(&self, mut installation: Installation) -> Result<Installation> {
        let refresh_token = installation
            .user_refresh_token
            .as_ref()
            .ok_or_else(|| Error::invalid_input("No user refresh token available"))?;

        debug!("Rotating user token for user {}", installation.user_id);

        let response = self.refresh_token(refresh_token).await?;

        if !response.ok {
            return Err(Error::api_error(format!(
                "Token refresh failed: {}",
                response
                    .error
                    .unwrap_or_else(|| "unknown error".to_string())
            )));
        }

        // Update installation with new token
        if let Some(access_token) = response.access_token {
            installation.user_token = Some(access_token);
        }

        if let Some(refresh_token) = response.refresh_token {
            installation.user_refresh_token = Some(refresh_token);
        }

        if let Some(expires_in) = response.expires_in {
            installation.user_token_expires_at = Some(Utc::now().timestamp() + expires_in);
        }

        // Save updated installation
        self.store.save(installation.clone()).await?;

        debug!("User token rotated successfully");

        Ok(installation)
    }

    /// Calls the Slack API to refresh a token
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenRefreshResponse> {
        let params = serde_json::json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
        });

        // Make direct HTTP call since oauth.v2.access doesn't require auth
        let response = self
            .http_client
            .post("https://slack.com/api/oauth.v2.access")
            .form(&params)
            .send()
            .await?;

        let refresh_response: TokenRefreshResponse = response.json().await.map_err(|e| {
            Error::deserialization_error(format!("Failed to parse refresh response: {}", e))
        })?;

        Ok(refresh_response)
    }

    /// Checks if bot token needs rotation and rotates if necessary
    ///
    /// # Arguments
    ///
    /// * `installation` - The installation to check
    /// * `threshold_seconds` - Rotate if token expires within this many seconds (default 7200 = 2 hours)
    ///
    /// # Returns
    ///
    /// The installation, potentially with refreshed tokens
    pub async fn check_and_rotate_bot_token(
        &self,
        installation: Installation,
        threshold_seconds: Option<i64>,
    ) -> Result<Installation> {
        let threshold = threshold_seconds.unwrap_or(7200); // 2 hours

        if self.needs_rotation(installation.bot_token_expires_at, threshold) {
            self.rotate_bot_token(installation).await
        } else {
            Ok(installation)
        }
    }

    /// Checks if user token needs rotation and rotates if necessary
    ///
    /// # Arguments
    ///
    /// * `installation` - The installation to check
    /// * `threshold_seconds` - Rotate if token expires within this many seconds (default 7200 = 2 hours)
    ///
    /// # Returns
    ///
    /// The installation, potentially with refreshed tokens
    pub async fn check_and_rotate_user_token(
        &self,
        installation: Installation,
        threshold_seconds: Option<i64>,
    ) -> Result<Installation> {
        let threshold = threshold_seconds.unwrap_or(7200); // 2 hours

        if self.needs_rotation(installation.user_token_expires_at, threshold) {
            self.rotate_user_token(installation).await
        } else {
            Ok(installation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::installation_store::cache::CacheInstallationStore;

    #[test]
    fn test_needs_rotation() {
        let store = Arc::new(CacheInstallationStore::new());
        let rotator = TokenRotator::new(store, "client_id", "client_secret");

        // Token expires in 1 hour - should need rotation (threshold 2 hours)
        let expires_at = Utc::now().timestamp() + 3600;
        assert!(rotator.needs_rotation(Some(expires_at), 7200));

        // Token expires in 3 hours - should not need rotation
        let expires_at = Utc::now().timestamp() + 10800;
        assert!(!rotator.needs_rotation(Some(expires_at), 7200));

        // No expiration - should not need rotation
        assert!(!rotator.needs_rotation(None, 7200));

        // Expired token - should need rotation
        let expires_at = Utc::now().timestamp() - 3600;
        assert!(rotator.needs_rotation(Some(expires_at), 7200));
    }

    #[test]
    fn test_token_rotator_creation() {
        let store = Arc::new(CacheInstallationStore::new());
        let _rotator = TokenRotator::new(store, "client_id", "client_secret");
    }
}
