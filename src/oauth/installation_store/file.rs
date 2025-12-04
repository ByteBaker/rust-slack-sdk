#![allow(missing_debug_implementations)]
//! File-based installation storage
//!
//! Stores installations as JSON files in a directory structure.

use crate::error::{Error, Result};
use crate::oauth::installation_store::InstallationStore;
use crate::oauth::models::{Bot, Installation};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

/// File-based installation store
///
/// Stores installations in a directory structure:
/// `{base_dir}/{enterprise_id or "none"}-{team_id or "none"}/`
///
/// Files created:
/// - `bot-latest` - Latest bot installation
/// - `bot-{timestamp}` - Historical bot installations (if enabled)
/// - `installer-latest` - Latest installation for any user
/// - `installer-{user_id}-latest` - Latest installation for specific user
/// - `installer-{user_id}-{timestamp}` - Historical installations (if enabled)
///
/// # Example
///
/// ```
/// use slack_rs::oauth::installation_store::{FileInstallationStore, InstallationStore};
/// use slack_rs::oauth::models::Installation;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = FileInstallationStore::new("/tmp/slack-installations");
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
pub struct FileInstallationStore {
    base_dir: PathBuf,
    historical_data_enabled: bool,
    client_id: Option<String>,
}

impl FileInstallationStore {
    /// Creates a new FileInstallationStore
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for storing installations
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            historical_data_enabled: true,
            client_id: None,
        }
    }

    /// Sets whether to store historical data
    ///
    /// When enabled, each save creates both a "latest" file and a timestamped file.
    /// When disabled, only "latest" files are kept.
    pub fn with_historical_data(mut self, enabled: bool) -> Self {
        self.historical_data_enabled = enabled;
        self
    }

    /// Sets the client ID
    ///
    /// When set, installations are stored in a subdirectory named after the client ID.
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Gets the base directory for this store
    fn get_base_dir(&self) -> PathBuf {
        if let Some(client_id) = &self.client_id {
            self.base_dir.join(client_id)
        } else {
            self.base_dir.clone()
        }
    }

    /// Gets the directory for a specific team/enterprise
    fn get_team_dir(&self, enterprise_id: Option<&str>, team_id: Option<&str>) -> PathBuf {
        let e_id = enterprise_id.unwrap_or("none");
        let t_id = team_id.unwrap_or("none");
        self.get_base_dir().join(format!("{}-{}", e_id, t_id))
    }

    /// Ensures a directory exists
    async fn ensure_dir(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).await.map_err(|e| {
            Error::storage_error(format!("Failed to create directory {:?}: {}", path, e))
        })
    }

    /// Writes data to a file
    async fn write_file(&self, path: &Path, data: &[u8]) -> Result<()> {
        fs::write(path, data)
            .await
            .map_err(|e| Error::storage_error(format!("Failed to write file {:?}: {}", path, e)))
    }

    /// Reads data from a file
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        fs::read(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::not_found(format!("Installation file not found: {:?}", path))
            } else {
                Error::storage_error(format!("Failed to read file {:?}: {}", path, e))
            }
        })
    }

    /// Deletes files matching a pattern
    async fn delete_files_by_pattern(&self, dir: &Path, pattern: &str) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            Error::storage_error(format!("Failed to read directory {:?}: {}", dir, e))
        })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::storage_error(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with(pattern) {
                        if let Err(e) = fs::remove_file(&path).await {
                            warn!("Failed to delete file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl InstallationStore for FileInstallationStore {
    async fn save(&self, installation: Installation) -> Result<()> {
        let actual_team_id = if installation.is_enterprise_install {
            None
        } else {
            installation.team_id.as_deref()
        };
        let team_dir = self.get_team_dir(installation.enterprise_id.as_deref(), actual_team_id);
        self.ensure_dir(&team_dir).await?;

        // Save bot data
        if let Some(bot) = installation.to_bot() {
            self.save_bot(bot).await?;
        }

        let data = serde_json::to_vec(&installation).map_err(|e| {
            Error::serialization_error(format!("Failed to serialize installation: {}", e))
        })?;

        if self.historical_data_enabled {
            let history_version = installation.installed_at.to_string();

            // Save workspace-level installation
            let latest_path = team_dir.join("installer-latest");
            self.write_file(&latest_path, &data).await?;

            let history_path = team_dir.join(format!("installer-{}", history_version));
            self.write_file(&history_path, &data).await?;

            // Save user-specific installation
            let user_id = &installation.user_id;
            let user_latest_path = team_dir.join(format!("installer-{}-latest", user_id));
            self.write_file(&user_latest_path, &data).await?;

            let user_history_path =
                team_dir.join(format!("installer-{}-{}", user_id, history_version));
            self.write_file(&user_history_path, &data).await?;
        } else {
            // Save only latest user-specific installation
            let user_id = &installation.user_id;
            let user_latest_path = team_dir.join(format!("installer-{}-latest", user_id));
            self.write_file(&user_latest_path, &data).await?;
        }

        debug!(
            "Saved installation for team {} user {}",
            installation.team_id.as_deref().unwrap_or("none"),
            installation.user_id
        );

        Ok(())
    }

    async fn save_bot(&self, bot: Bot) -> Result<()> {
        let actual_team_id = if bot.is_enterprise_install {
            None
        } else {
            bot.team_id.as_deref()
        };
        let team_dir = self.get_team_dir(bot.enterprise_id.as_deref(), actual_team_id);
        self.ensure_dir(&team_dir).await?;

        let data = serde_json::to_vec(&bot)
            .map_err(|e| Error::serialization_error(format!("Failed to serialize bot: {}", e)))?;

        if self.historical_data_enabled {
            let history_version = bot.installed_at.to_string();

            let latest_path = team_dir.join("bot-latest");
            self.write_file(&latest_path, &data).await?;

            let history_path = team_dir.join(format!("bot-{}", history_version));
            self.write_file(&history_path, &data).await?;
        } else {
            let latest_path = team_dir.join("bot-latest");
            self.write_file(&latest_path, &data).await?;
        }

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
        let team_dir = self.get_team_dir(enterprise_id, actual_team_id);
        let bot_path = team_dir.join("bot-latest");

        if !bot_path.exists() {
            debug!(
                "Bot not found for enterprise {} team {}",
                enterprise_id.unwrap_or("none"),
                team_id.unwrap_or("none")
            );
            return Ok(None);
        }

        let data = self.read_file(&bot_path).await?;
        let bot: Bot = serde_json::from_slice(&data).map_err(|e| {
            Error::deserialization_error(format!("Failed to deserialize bot: {}", e))
        })?;

        Ok(Some(bot))
    }

    async fn find_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Installation>> {
        let actual_team_id = if is_enterprise_install { None } else { team_id };
        let team_dir = self.get_team_dir(enterprise_id, actual_team_id);

        let installation_path = if let Some(user_id) = user_id {
            team_dir.join(format!("installer-{}-latest", user_id))
        } else {
            team_dir.join("installer-latest")
        };

        if !installation_path.exists() {
            debug!(
                "Installation not found for enterprise {} team {} user {}",
                enterprise_id.unwrap_or("none"),
                team_id.unwrap_or("none"),
                user_id.unwrap_or("any")
            );
            return Ok(None);
        }

        let data = self.read_file(&installation_path).await?;
        let mut installation: Installation = serde_json::from_slice(&data).map_err(|e| {
            Error::deserialization_error(format!("Failed to deserialize installation: {}", e))
        })?;

        // If this is a user-specific installation or missing bot token,
        // try to get the latest bot token
        let should_find_bot = user_id.is_some() || installation.bot_token.is_none();
        if should_find_bot {
            if let Ok(Some(bot)) = self
                .find_bot(enterprise_id, team_id, is_enterprise_install)
                .await
            {
                if installation.bot_token.as_ref() != Some(&bot.bot_token) {
                    installation.bot_id = Some(bot.bot_id);
                    installation.bot_user_id = Some(bot.bot_user_id);
                    installation.bot_token = Some(bot.bot_token);
                    installation.bot_scopes = Some(bot.bot_scopes);
                    installation.bot_refresh_token = bot.bot_refresh_token;
                    installation.bot_token_expires_at = bot.bot_token_expires_at;
                }
            }
        }

        Ok(Some(installation))
    }

    async fn delete_bot(&self, enterprise_id: Option<&str>, team_id: Option<&str>) -> Result<()> {
        let team_dir = self.get_team_dir(enterprise_id, team_id);
        self.delete_files_by_pattern(&team_dir, "bot-").await?;

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
        let team_dir = self.get_team_dir(enterprise_id, team_id);

        if let Some(user_id) = user_id {
            let pattern = format!("installer-{}-", user_id);
            self.delete_files_by_pattern(&team_dir, &pattern).await?;
        } else {
            self.delete_files_by_pattern(&team_dir, "installer-")
                .await?;
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
    use tempfile::TempDir;

    async fn create_test_store() -> (FileInstallationStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = FileInstallationStore::new(temp_dir.path());
        (store, temp_dir)
    }

    #[tokio::test]
    async fn test_save_and_find_installation() {
        let (store, _temp_dir) = create_test_store().await;

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
        let (store, _temp_dir) = create_test_store().await;

        let bot = Bot::new("xoxb-token", "B12345", "U67890");
        let mut bot = bot;
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
        let (store, _temp_dir) = create_test_store().await;

        let found = store
            .find_installation(None, Some("T99999"), Some("U99999"), false)
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_installation() {
        let (store, _temp_dir) = create_test_store().await;

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
        let (store, _temp_dir) = create_test_store().await;

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
        let (store, _temp_dir) = create_test_store().await;

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
    async fn test_enterprise_install() {
        let (store, _temp_dir) = create_test_store().await;

        let installation = Installation::new("U12345")
            .enterprise_id("E12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890")
            .is_enterprise_install(true);

        store.save(installation).await.unwrap();

        let found = store
            .find_installation(Some("E12345"), Some("T12345"), Some("U12345"), true)
            .await
            .unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.enterprise_id, Some("E12345".to_string()));
        assert!(found.is_enterprise_install);
    }

    #[tokio::test]
    async fn test_historical_data_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let store = FileInstallationStore::new(temp_dir.path()).with_historical_data(false);

        let installation = Installation::new("U12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890");

        store.save(installation).await.unwrap();

        // Check that only latest file exists
        let team_dir = store.get_team_dir(None, Some("T12345"));
        let mut entries = fs::read_dir(&team_dir).await.unwrap();
        let mut file_count = 0;

        while let Some(entry) = entries.next_entry().await.unwrap() {
            let name = entry.file_name();
            let name_str = name.to_str().unwrap();
            assert!(
                name_str.contains("latest"),
                "Found non-latest file: {}",
                name_str
            );
            file_count += 1;
        }

        assert_eq!(file_count, 2); // bot-latest and installer-U12345-latest
    }
}
