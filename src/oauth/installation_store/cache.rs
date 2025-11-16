//! In-memory cache-based installation storage
//!
//! Provides fast, thread-safe in-memory storage for installations.

use crate::error::Result;
use crate::oauth::installation_store::InstallationStore;
use crate::oauth::models::{Bot, Installation};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Key for looking up installations
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct InstallationKey {
    enterprise_id: Option<String>,
    team_id: Option<String>,
    user_id: Option<String>,
}

impl InstallationKey {
    fn new(enterprise_id: Option<&str>, team_id: Option<&str>, user_id: Option<&str>) -> Self {
        Self {
            enterprise_id: enterprise_id.map(|s| s.to_string()),
            team_id: team_id.map(|s| s.to_string()),
            user_id: user_id.map(|s| s.to_string()),
        }
    }

    fn bot_key(enterprise_id: Option<&str>, team_id: Option<&str>) -> Self {
        Self::new(enterprise_id, team_id, None)
    }
}

/// In-memory installation store
///
/// Stores installations in memory using a thread-safe HashMap with RwLock.
/// Suitable for development, testing, or as a fast cache layer.
///
/// # Warning
///
/// All data is lost when the process terminates. For production use,
/// consider using FileInstallationStore or database-backed stores.
///
/// # Example
///
/// ```
/// use slack_rs::oauth::installation_store::{CacheInstallationStore, InstallationStore};
/// use slack_rs::oauth::models::Installation;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = CacheInstallationStore::new();
///
///     let installation = Installation::new("U12345")
///         .app_id("A12345")
///         .team_id("T12345")
///         .bot_token("xoxb-token")
///         .bot_id("B12345")
///         .bot_user_id("U67890");
///
///     store.save(installation).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CacheInstallationStore {
    installations: Arc<RwLock<HashMap<InstallationKey, Installation>>>,
    bots: Arc<RwLock<HashMap<InstallationKey, Bot>>>,
}

impl CacheInstallationStore {
    /// Creates a new CacheInstallationStore
    pub fn new() -> Self {
        Self {
            installations: Arc::new(RwLock::new(HashMap::new())),
            bots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns the number of stored installations
    pub async fn installation_count(&self) -> usize {
        self.installations.read().await.len()
    }

    /// Returns the number of stored bots
    pub async fn bot_count(&self) -> usize {
        self.bots.read().await.len()
    }

    /// Clears all stored data
    pub async fn clear(&self) {
        self.installations.write().await.clear();
        self.bots.write().await.clear();
    }
}

impl Default for CacheInstallationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InstallationStore for CacheInstallationStore {
    async fn save(&self, installation: Installation) -> Result<()> {
        // Save bot data
        if let Some(bot) = installation.to_bot() {
            self.save_bot(bot).await?;
        }

        // Save installation with user-specific key
        let user_key = InstallationKey::new(
            installation.enterprise_id.as_deref(),
            installation.team_id.as_deref(),
            Some(&installation.user_id),
        );

        self.installations
            .write()
            .await
            .insert(user_key, installation.clone());

        // Save installation with workspace-level key (for finding latest)
        let workspace_key = InstallationKey::new(
            installation.enterprise_id.as_deref(),
            installation.team_id.as_deref(),
            None,
        );

        self.installations
            .write()
            .await
            .insert(workspace_key, installation.clone());

        debug!(
            "Saved installation for team {} user {}",
            installation.team_id.as_deref().unwrap_or("none"),
            installation.user_id
        );

        Ok(())
    }

    async fn save_bot(&self, bot: Bot) -> Result<()> {
        let key = InstallationKey::bot_key(bot.enterprise_id.as_deref(), bot.team_id.as_deref());

        self.bots.write().await.insert(key, bot.clone());

        debug!(
            "Saved bot for team {}",
            bot.team_id.as_deref().unwrap_or("none")
        );

        Ok(())
    }

    async fn find_bot(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Bot>> {
        let actual_team_id = if is_enterprise_install { None } else { team_id };
        let key = InstallationKey::bot_key(enterprise_id, actual_team_id);

        let bot = self.bots.read().await.get(&key).cloned();

        if bot.is_none() {
            debug!(
                "Bot not found for enterprise {} team {}",
                enterprise_id.unwrap_or("none"),
                team_id.unwrap_or("none")
            );
        }

        Ok(bot)
    }

    async fn find_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Installation>> {
        let actual_team_id = if is_enterprise_install { None } else { team_id };
        let key = InstallationKey::new(enterprise_id, actual_team_id, user_id);

        let mut installation = self.installations.read().await.get(&key).cloned();

        if installation.is_none() {
            debug!(
                "Installation not found for enterprise {} team {} user {}",
                enterprise_id.unwrap_or("none"),
                team_id.unwrap_or("none"),
                user_id.unwrap_or("any")
            );
            return Ok(None);
        }

        // If this is a user-specific installation or missing bot token,
        // try to get the latest bot token
        let should_find_bot =
            user_id.is_some() || installation.as_ref().unwrap().bot_token.is_none();
        if should_find_bot {
            if let Ok(Some(bot)) = self
                .find_bot(enterprise_id, team_id, is_enterprise_install)
                .await
            {
                if let Some(ref mut inst) = installation {
                    if inst.bot_token.as_ref() != Some(&bot.bot_token) {
                        inst.bot_id = Some(bot.bot_id.clone());
                        inst.bot_user_id = Some(bot.bot_user_id.clone());
                        inst.bot_token = Some(bot.bot_token.clone());
                        inst.bot_scopes = Some(bot.bot_scopes.clone());
                        inst.bot_refresh_token = bot.bot_refresh_token.clone();
                        inst.bot_token_expires_at = bot.bot_token_expires_at;
                    }
                }
            }
        }

        Ok(installation)
    }

    async fn delete_bot(&self, enterprise_id: Option<&str>, team_id: Option<&str>) -> Result<()> {
        let key = InstallationKey::bot_key(enterprise_id, team_id);
        self.bots.write().await.remove(&key);

        debug!(
            "Deleted bot for enterprise {} team {}",
            enterprise_id.unwrap_or("none"),
            team_id.unwrap_or("none")
        );

        Ok(())
    }

    async fn delete_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<()> {
        if let Some(user_id) = user_id {
            // Delete specific user installation
            let key = InstallationKey::new(enterprise_id, team_id, Some(user_id));
            self.installations.write().await.remove(&key);
        } else {
            // Delete all installations for the workspace
            let mut installations = self.installations.write().await;
            installations.retain(|key, _| {
                !(key.enterprise_id.as_deref() == enterprise_id
                    && key.team_id.as_deref() == team_id)
            });
        }

        debug!(
            "Deleted installation for enterprise {} team {} user {}",
            enterprise_id.unwrap_or("none"),
            team_id.unwrap_or("none"),
            user_id.unwrap_or("any")
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_save_and_find_installation() {
        let store = CacheInstallationStore::new();

        let installation = Installation::new("U12345")
            .app_id("A12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        store.save(installation.clone()).await.unwrap();

        let found = store
            .find_installation(None, Some("T12345"), Some("U12345"), false)
            .await
            .unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.user_id, "U12345");
        assert_eq!(found.team_id, Some("T12345".to_string()));
        assert_eq!(found.bot_token, Some("xoxb-token".to_string()));
    }

    #[tokio::test]
    async fn test_save_and_find_bot() {
        let store = CacheInstallationStore::new();

        let mut bot = Bot::new("xoxb-token", "B12345", "U67890");
        bot.team_id = Some("T12345".to_string());

        store.save_bot(bot.clone()).await.unwrap();

        let found = store.find_bot(None, Some("T12345"), false).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.bot_token, "xoxb-token");
        assert_eq!(found.bot_id, "B12345");
    }

    #[tokio::test]
    async fn test_find_nonexistent_installation() {
        let store = CacheInstallationStore::new();

        let found = store
            .find_installation(None, Some("T99999"), Some("U99999"), false)
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_installation() {
        let store = CacheInstallationStore::new();

        let installation = Installation::new("U12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        store.save(installation).await.unwrap();

        let found = store
            .find_installation(None, Some("T12345"), Some("U12345"), false)
            .await
            .unwrap();
        assert!(found.is_some());

        store
            .delete_installation(None, Some("T12345"), Some("U12345"))
            .await
            .unwrap();

        let found = store
            .find_installation(None, Some("T12345"), Some("U12345"), false)
            .await
            .unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_bot() {
        let store = CacheInstallationStore::new();

        let mut bot = Bot::new("xoxb-token", "B12345", "U67890");
        bot.team_id = Some("T12345".to_string());

        store.save_bot(bot).await.unwrap();

        let found = store.find_bot(None, Some("T12345"), false).await.unwrap();
        assert!(found.is_some());

        store.delete_bot(None, Some("T12345")).await.unwrap();

        let found = store.find_bot(None, Some("T12345"), false).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_all() {
        let store = CacheInstallationStore::new();

        let installation = Installation::new("U12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        store.save(installation).await.unwrap();

        store.delete_all(None, Some("T12345")).await.unwrap();

        let found = store
            .find_installation(None, Some("T12345"), Some("U12345"), false)
            .await
            .unwrap();
        assert!(found.is_none());

        let found_bot = store.find_bot(None, Some("T12345"), false).await.unwrap();
        assert!(found_bot.is_none());
    }

    #[tokio::test]
    async fn test_clear() {
        let store = CacheInstallationStore::new();

        let installation = Installation::new("U12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        store.save(installation).await.unwrap();
        assert_eq!(store.installation_count().await, 2); // user-specific + workspace-level

        store.clear().await;
        assert_eq!(store.installation_count().await, 0);
        assert_eq!(store.bot_count().await, 0);
    }

    #[tokio::test]
    async fn test_multiple_users() {
        let store = CacheInstallationStore::new();

        let installation1 = Installation::new("U12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        let installation2 = Installation::new("U54321")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        store.save(installation1).await.unwrap();
        store.save(installation2).await.unwrap();

        let found1 = store
            .find_installation(None, Some("T12345"), Some("U12345"), false)
            .await
            .unwrap();
        assert!(found1.is_some());

        let found2 = store
            .find_installation(None, Some("T12345"), Some("U54321"), false)
            .await
            .unwrap();
        assert!(found2.is_some());

        // Deleting one user shouldn't affect the other
        store
            .delete_installation(None, Some("T12345"), Some("U12345"))
            .await
            .unwrap();

        let found1 = store
            .find_installation(None, Some("T12345"), Some("U12345"), false)
            .await
            .unwrap();
        assert!(found1.is_none());

        let found2 = store
            .find_installation(None, Some("T12345"), Some("U54321"), false)
            .await
            .unwrap();
        assert!(found2.is_some());
    }
}
