//! Installation storage for OAuth installations
//!
//! This module provides traits and implementations for storing and retrieving
//! Slack app installation data.

use crate::error::Result;
use crate::oauth::models::{Bot, Installation};
use async_trait::async_trait;

pub mod cache;
pub mod file;

#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Trait for storing and retrieving Slack app installations
///
/// Implementations of this trait handle persistence of OAuth installation data,
/// including bot and user tokens, scopes, and refresh tokens.
///
/// The minimum required methods are:
/// - `save` - Store an installation
/// - `find_installation` - Retrieve an installation
///
/// For proper handling of app uninstallations and token revocations, implement:
/// - `delete_installation` - Remove a specific installation
/// - `delete_all` - Remove all installations for a workspace/org
///
/// If your app only needs bot scope installations, you can use:
/// - `save_bot` - Store bot data
/// - `find_bot` - Retrieve bot data
/// - `delete_bot` - Remove bot data
#[async_trait]
pub trait InstallationStore: Send + Sync {
    /// Saves an installation
    async fn save(&self, installation: Installation) -> Result<()>;

    /// Saves bot installation data
    async fn save_bot(&self, bot: Bot) -> Result<()>;

    /// Finds a bot scope installation per workspace/org
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - Enterprise Grid organization ID (None for non-Grid workspaces)
    /// * `team_id` - Workspace/team ID
    /// * `is_enterprise_install` - Whether this is an org-wide installation
    async fn find_bot(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Bot>>;

    /// Finds an installation for the given IDs
    ///
    /// If `user_id` is None, implementations may return the latest installation
    /// in the workspace/org.
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - Enterprise Grid organization ID (None for non-Grid workspaces)
    /// * `team_id` - Workspace/team ID
    /// * `user_id` - User ID who installed the app (None for any user)
    /// * `is_enterprise_install` - Whether this is an org-wide installation
    async fn find_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Installation>>;

    /// Deletes a bot scope installation per workspace/org
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - Enterprise Grid organization ID (None for non-Grid workspaces)
    /// * `team_id` - Workspace/team ID
    async fn delete_bot(&self, enterprise_id: Option<&str>, team_id: Option<&str>) -> Result<()>;

    /// Deletes an installation that matches the given IDs
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - Enterprise Grid organization ID (None for non-Grid workspaces)
    /// * `team_id` - Workspace/team ID
    /// * `user_id` - User ID who installed the app (None to delete all for workspace)
    async fn delete_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<()>;

    /// Deletes all installation data for the given workspace/org
    ///
    /// This is a convenience method that calls both delete_bot and delete_installation.
    ///
    /// # Arguments
    ///
    /// * `enterprise_id` - Enterprise Grid organization ID (None for non-Grid workspaces)
    /// * `team_id` - Workspace/team ID
    async fn delete_all(&self, enterprise_id: Option<&str>, team_id: Option<&str>) -> Result<()> {
        self.delete_bot(enterprise_id, team_id).await?;
        self.delete_installation(enterprise_id, team_id, None)
            .await?;
        Ok(())
    }
}
