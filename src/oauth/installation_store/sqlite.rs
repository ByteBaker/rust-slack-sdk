#![allow(missing_debug_implementations)]
//! SQLite-based installation storage
//!
//! Stores installations in a SQLite database with proper indexing.

use crate::error::{Error, Result};
use crate::oauth::installation_store::InstallationStore;
use crate::oauth::models::{Bot, Installation};
use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// SQLite-based installation store
///
/// Stores installations in a SQLite database with tables for installations and bots.
/// Thread-safe and supports concurrent access.
///
/// # Example
///
/// ```no_run
/// use slack_rs::oauth::installation_store::{SqliteInstallationStore, InstallationStore};
/// use slack_rs::oauth::models::Installation;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let store = SqliteInstallationStore::new("/tmp/slack.db", "client_id_123").await?;
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
pub struct SqliteInstallationStore {
    conn: Arc<Mutex<Connection>>,
    client_id: String,
}

impl SqliteInstallationStore {
    /// Creates a new SqliteInstallationStore
    ///
    /// # Arguments
    ///
    /// * `database` - Path to the SQLite database file
    /// * `client_id` - OAuth client ID for this app
    pub async fn new(database: impl AsRef<Path>, client_id: impl Into<String>) -> Result<Self> {
        let database = database.as_ref().to_path_buf();
        let client_id = client_id.into();

        let conn = tokio::task::spawn_blocking(move || {
            let conn = Connection::open(&database).map_err(|e| {
                Error::storage_error(format!("Failed to open database {:?}: {}", database, e))
            })?;
            Ok::<_, Error>(conn)
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))??;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
            client_id,
        };

        store.create_tables().await?;

        Ok(store)
    }

    /// Creates the necessary tables if they don't exist
    async fn create_tables(&self) -> Result<()> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            // Check if tables exist
            let table_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='slack_installations'",
                    [],
                    |row| row.get(0),
                )
                .map(|count: i64| count > 0)
                .unwrap_or(false);

            if table_exists {
                return Ok::<_, Error>(());
            }

            conn.execute(
                r#"
                CREATE TABLE slack_installations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    client_id TEXT NOT NULL,
                    app_id TEXT NOT NULL,
                    enterprise_id TEXT NOT NULL DEFAULT '',
                    enterprise_name TEXT,
                    enterprise_url TEXT,
                    team_id TEXT NOT NULL DEFAULT '',
                    team_name TEXT,
                    bot_token TEXT,
                    bot_id TEXT,
                    bot_user_id TEXT,
                    bot_scopes TEXT,
                    bot_refresh_token TEXT,
                    bot_token_expires_at INTEGER,
                    user_id TEXT NOT NULL,
                    user_token TEXT,
                    user_scopes TEXT,
                    user_refresh_token TEXT,
                    user_token_expires_at INTEGER,
                    incoming_webhook_url TEXT,
                    incoming_webhook_channel TEXT,
                    incoming_webhook_channel_id TEXT,
                    incoming_webhook_configuration_url TEXT,
                    is_enterprise_install INTEGER NOT NULL DEFAULT 0,
                    token_type TEXT,
                    installed_at REAL NOT NULL
                )
                "#,
                [],
            )
            .map_err(|e| Error::storage_error(format!("Failed to create installations table: {}", e)))?;

            conn.execute(
                r#"
                CREATE INDEX slack_installations_idx ON slack_installations (
                    client_id,
                    enterprise_id,
                    team_id,
                    user_id,
                    installed_at
                )
                "#,
                [],
            )
            .map_err(|e| Error::storage_error(format!("Failed to create installations index: {}", e)))?;

            conn.execute(
                r#"
                CREATE TABLE slack_bots (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    client_id TEXT NOT NULL,
                    app_id TEXT NOT NULL,
                    enterprise_id TEXT NOT NULL DEFAULT '',
                    enterprise_name TEXT,
                    team_id TEXT NOT NULL DEFAULT '',
                    team_name TEXT,
                    bot_token TEXT NOT NULL,
                    bot_id TEXT NOT NULL,
                    bot_user_id TEXT NOT NULL,
                    bot_scopes TEXT,
                    bot_refresh_token TEXT,
                    bot_token_expires_at INTEGER,
                    is_enterprise_install INTEGER NOT NULL DEFAULT 0,
                    installed_at REAL NOT NULL
                )
                "#,
                [],
            )
            .map_err(|e| Error::storage_error(format!("Failed to create bots table: {}", e)))?;

            conn.execute(
                r#"
                CREATE INDEX slack_bots_idx ON slack_bots (
                    client_id,
                    enterprise_id,
                    team_id,
                    installed_at
                )
                "#,
                [],
            )
            .map_err(|e| Error::storage_error(format!("Failed to create bots index: {}", e)))?;

            debug!("SQLite tables created");
            Ok(())
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

#[async_trait]
impl InstallationStore for SqliteInstallationStore {
    async fn save(&self, installation: Installation) -> Result<()> {
        // Save bot data first
        if let Some(bot) = installation.to_bot() {
            self.save_bot(bot).await?;
        }

        let conn = self.conn.clone();
        let client_id = self.client_id.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let bot_scopes = installation
                .bot_scopes
                .as_ref()
                .map(|s| s.join(","))
                .unwrap_or_default();
            let user_scopes = installation
                .user_scopes
                .as_ref()
                .map(|s| s.join(","))
                .unwrap_or_default();

            let team_id_str = installation.team_id.as_deref().unwrap_or("");
            let enterprise_id_str = installation.enterprise_id.as_deref().unwrap_or("");

            conn.execute(
                r#"
                INSERT INTO slack_installations (
                    client_id, app_id, enterprise_id, enterprise_name, enterprise_url,
                    team_id, team_name, bot_token, bot_id, bot_user_id, bot_scopes,
                    bot_refresh_token, bot_token_expires_at, user_id, user_token, user_scopes,
                    user_refresh_token, user_token_expires_at, incoming_webhook_url,
                    incoming_webhook_channel, incoming_webhook_channel_id,
                    incoming_webhook_configuration_url, is_enterprise_install, token_type,
                    installed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)
                "#,
                params![
                    client_id,
                    installation.app_id,
                    enterprise_id_str,
                    installation.enterprise_name,
                    installation.enterprise_url,
                    team_id_str,
                    installation.team_name,
                    installation.bot_token,
                    installation.bot_id,
                    installation.bot_user_id,
                    bot_scopes,
                    installation.bot_refresh_token,
                    installation.bot_token_expires_at,
                    installation.user_id,
                    installation.user_token,
                    user_scopes,
                    installation.user_refresh_token,
                    installation.user_token_expires_at,
                    installation.incoming_webhook_url,
                    installation.incoming_webhook_channel,
                    installation.incoming_webhook_channel_id,
                    installation.incoming_webhook_configuration_url,
                    if installation.is_enterprise_install { 1 } else { 0 },
                    installation.token_type,
                    installation.installed_at,
                ],
            )
            .map_err(|e| Error::storage_error(format!("Failed to insert installation: {}", e)))?;

            debug!(
                "Saved installation for team {} user {}",
                team_id_str,
                installation.user_id
            );

            Ok::<_, Error>(())
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn save_bot(&self, bot: Bot) -> Result<()> {
        let conn = self.conn.clone();
        let client_id = self.client_id.clone();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let bot_scopes = bot.bot_scopes.join(",");

            let team_id_str = bot.team_id.as_deref().unwrap_or("");
            let enterprise_id_str = bot.enterprise_id.as_deref().unwrap_or("");

            conn.execute(
                r#"
                INSERT INTO slack_bots (
                    client_id, app_id, enterprise_id, enterprise_name, team_id, team_name,
                    bot_token, bot_id, bot_user_id, bot_scopes, bot_refresh_token,
                    bot_token_expires_at, is_enterprise_install, installed_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                "#,
                params![
                    client_id,
                    bot.app_id,
                    enterprise_id_str,
                    bot.enterprise_name,
                    team_id_str,
                    bot.team_name,
                    bot.bot_token,
                    bot.bot_id,
                    bot.bot_user_id,
                    bot_scopes,
                    bot.bot_refresh_token,
                    bot.bot_token_expires_at,
                    if bot.is_enterprise_install { 1 } else { 0 },
                    bot.installed_at,
                ],
            )
            .map_err(|e| Error::storage_error(format!("Failed to insert bot: {}", e)))?;

            debug!("Saved bot for team {}", team_id_str);

            Ok::<_, Error>(())
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn find_bot(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Bot>> {
        let conn = self.conn.clone();
        let client_id = self.client_id.clone();
        let enterprise_id = enterprise_id.unwrap_or("").to_string();
        let team_id = if is_enterprise_install {
            String::new()
        } else {
            team_id.unwrap_or("").to_string()
        };

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let bot = conn
                .query_row(
                    r#"
                    SELECT app_id, enterprise_id, enterprise_name, team_id, team_name,
                           bot_token, bot_id, bot_user_id, bot_scopes, bot_refresh_token,
                           bot_token_expires_at, is_enterprise_install, installed_at
                    FROM slack_bots
                    WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3
                    ORDER BY installed_at DESC
                    LIMIT 1
                    "#,
                    params![client_id, enterprise_id, team_id],
                    |row| {
                        let bot_scopes: String = row.get(8)?;
                        let bot_scopes = if bot_scopes.is_empty() {
                            Vec::new()
                        } else {
                            bot_scopes.split(',').map(|s| s.to_string()).collect()
                        };

                        let enterprise_id: String = row.get(1)?;
                        let team_id: String = row.get(3)?;

                        Ok(Bot {
                            app_id: row.get(0)?,
                            enterprise_id: if enterprise_id.is_empty() {
                                None
                            } else {
                                Some(enterprise_id)
                            },
                            enterprise_name: row.get(2)?,
                            team_id: if team_id.is_empty() {
                                None
                            } else {
                                Some(team_id)
                            },
                            team_name: row.get(4)?,
                            bot_token: row.get(5)?,
                            bot_id: row.get(6)?,
                            bot_user_id: row.get(7)?,
                            bot_scopes,
                            bot_refresh_token: row.get(9)?,
                            bot_token_expires_at: row.get(10)?,
                            is_enterprise_install: row.get::<_, i32>(11)? != 0,
                            installed_at: row.get(12)?,
                            custom_values: Default::default(),
                        })
                    },
                )
                .optional()
                .map_err(|e| Error::storage_error(format!("Failed to query bot: {}", e)))?;

            if bot.is_none() {
                debug!(
                    "Bot not found for enterprise {} team {}",
                    enterprise_id, team_id
                );
            }

            Ok::<_, Error>(bot)
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))?
    }

    async fn find_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
        is_enterprise_install: bool,
    ) -> Result<Option<Installation>> {
        let conn = self.conn.clone();
        let client_id = self.client_id.clone();
        let enterprise_id_str = enterprise_id.unwrap_or("").to_string();
        let team_id_str = if is_enterprise_install {
            String::new()
        } else {
            team_id.unwrap_or("").to_string()
        };
        let user_id = user_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            let mut installation = if let Some(user_id) = &user_id {
                conn.query_row(
                    r#"
                    SELECT app_id, enterprise_id, enterprise_name, enterprise_url, team_id, team_name,
                           bot_token, bot_id, bot_user_id, bot_scopes, bot_refresh_token, bot_token_expires_at,
                           user_id, user_token, user_scopes, user_refresh_token, user_token_expires_at,
                           incoming_webhook_url, incoming_webhook_channel, incoming_webhook_channel_id,
                           incoming_webhook_configuration_url, is_enterprise_install, token_type, installed_at
                    FROM slack_installations
                    WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3 AND user_id = ?4
                    ORDER BY installed_at DESC
                    LIMIT 1
                    "#,
                    params![client_id, enterprise_id_str, team_id_str, user_id],
                    parse_installation_row,
                )
                .optional()
                .map_err(|e| Error::storage_error(format!("Failed to query installation: {}", e)))?
            } else {
                conn.query_row(
                    r#"
                    SELECT app_id, enterprise_id, enterprise_name, enterprise_url, team_id, team_name,
                           bot_token, bot_id, bot_user_id, bot_scopes, bot_refresh_token, bot_token_expires_at,
                           user_id, user_token, user_scopes, user_refresh_token, user_token_expires_at,
                           incoming_webhook_url, incoming_webhook_channel, incoming_webhook_channel_id,
                           incoming_webhook_configuration_url, is_enterprise_install, token_type, installed_at
                    FROM slack_installations
                    WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3
                    ORDER BY installed_at DESC
                    LIMIT 1
                    "#,
                    params![client_id, enterprise_id_str, team_id_str],
                    parse_installation_row,
                )
                .optional()
                .map_err(|e| Error::storage_error(format!("Failed to query installation: {}", e)))?
            };

            if installation.is_none() {
                debug!(
                    "Installation not found for enterprise {} team {} user {}",
                    enterprise_id_str,
                    team_id_str,
                    user_id.as_deref().unwrap_or("any")
                );
                return Ok(None);
            }

            // If this is a user-specific installation, get the latest bot token
            if user_id.is_some() {
                if let Some(ref mut inst) = installation {
                    let latest_bot = conn
                        .query_row(
                            r#"
                            SELECT bot_token, bot_id, bot_user_id, bot_scopes, bot_refresh_token, bot_token_expires_at
                            FROM slack_installations
                            WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3 AND bot_token IS NOT NULL
                            ORDER BY installed_at DESC
                            LIMIT 1
                            "#,
                            params![client_id, enterprise_id_str, team_id_str],
                            |row| {
                                let bot_scopes: Option<String> = row.get(3)?;
                                let bot_scopes = bot_scopes.map(|s| {
                                    if s.is_empty() {
                                        Vec::new()
                                    } else {
                                        s.split(',').map(|s| s.to_string()).collect()
                                    }
                                });

                                Ok((
                                    row.get::<_, Option<String>>(0)?,
                                    row.get::<_, Option<String>>(1)?,
                                    row.get::<_, Option<String>>(2)?,
                                    bot_scopes,
                                    row.get::<_, Option<String>>(4)?,
                                    row.get::<_, Option<i64>>(5)?,
                                ))
                            },
                        )
                        .optional()
                        .map_err(|e| Error::storage_error(format!("Failed to query latest bot: {}", e)))?;

                    if let Some((bot_token, bot_id, bot_user_id, bot_scopes, bot_refresh_token, bot_token_expires_at)) = latest_bot {
                        inst.bot_token = bot_token;
                        inst.bot_id = bot_id;
                        inst.bot_user_id = bot_user_id;
                        inst.bot_scopes = bot_scopes;
                        inst.bot_refresh_token = bot_refresh_token;
                        inst.bot_token_expires_at = bot_token_expires_at;
                    }
                }
            }

            Ok(installation)
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))?
    }

    async fn delete_bot(&self, enterprise_id: Option<&str>, team_id: Option<&str>) -> Result<()> {
        let conn = self.conn.clone();
        let client_id = self.client_id.clone();
        let enterprise_id = enterprise_id.unwrap_or("").to_string();
        let team_id = team_id.unwrap_or("").to_string();

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            conn.execute(
                "DELETE FROM slack_bots WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3",
                params![client_id, enterprise_id, team_id],
            )
            .map_err(|e| Error::storage_error(format!("Failed to delete bot: {}", e)))?;

            debug!("Deleted bot for enterprise {} team {}", enterprise_id, team_id);

            Ok::<_, Error>(())
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn delete_installation(
        &self,
        enterprise_id: Option<&str>,
        team_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.clone();
        let client_id = self.client_id.clone();
        let enterprise_id = enterprise_id.unwrap_or("").to_string();
        let team_id = team_id.unwrap_or("").to_string();
        let user_id = user_id.map(|s| s.to_string());

        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().unwrap();

            if let Some(user_id) = &user_id {
                conn.execute(
                    "DELETE FROM slack_installations WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3 AND user_id = ?4",
                    params![client_id, enterprise_id, team_id, user_id],
                )
                .map_err(|e| Error::storage_error(format!("Failed to delete installation: {}", e)))?;
            } else {
                conn.execute(
                    "DELETE FROM slack_installations WHERE client_id = ?1 AND enterprise_id = ?2 AND team_id = ?3",
                    params![client_id, enterprise_id, team_id],
                )
                .map_err(|e| Error::storage_error(format!("Failed to delete installation: {}", e)))?;
            }

            debug!(
                "Deleted installation for enterprise {} team {} user {}",
                enterprise_id,
                team_id,
                user_id.as_deref().unwrap_or("any")
            );

            Ok::<_, Error>(())
        })
        .await
        .map_err(|e| Error::storage_error(format!("Task join error: {}", e)))??;

        Ok(())
    }
}

fn parse_installation_row(row: &rusqlite::Row) -> rusqlite::Result<Installation> {
    let bot_scopes: Option<String> = row.get(9)?;
    let bot_scopes = bot_scopes.map(|s| {
        if s.is_empty() {
            Vec::new()
        } else {
            s.split(',').map(|s| s.to_string()).collect()
        }
    });

    let user_scopes: Option<String> = row.get(14)?;
    let user_scopes = user_scopes.map(|s| {
        if s.is_empty() {
            Vec::new()
        } else {
            s.split(',').map(|s| s.to_string()).collect()
        }
    });

    let enterprise_id: String = row.get(1)?;
    let team_id: String = row.get(4)?;

    Ok(Installation {
        app_id: row.get(0)?,
        enterprise_id: if enterprise_id.is_empty() {
            None
        } else {
            Some(enterprise_id)
        },
        enterprise_name: row.get(2)?,
        enterprise_url: row.get(3)?,
        team_id: if team_id.is_empty() {
            None
        } else {
            Some(team_id)
        },
        team_name: row.get(5)?,
        bot_token: row.get(6)?,
        bot_id: row.get(7)?,
        bot_user_id: row.get(8)?,
        bot_scopes,
        bot_refresh_token: row.get(10)?,
        bot_token_expires_at: row.get(11)?,
        user_id: row.get(12)?,
        user_token: row.get(13)?,
        user_scopes,
        user_refresh_token: row.get(15)?,
        user_token_expires_at: row.get(16)?,
        incoming_webhook_url: row.get(17)?,
        incoming_webhook_channel: row.get(18)?,
        incoming_webhook_channel_id: row.get(19)?,
        incoming_webhook_configuration_url: row.get(20)?,
        is_enterprise_install: row.get::<_, i32>(21)? != 0,
        token_type: row.get(22)?,
        installed_at: row.get(23)?,
        custom_values: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_store() -> SqliteInstallationStore {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = SqliteInstallationStore::new(&db_path, "client_123")
            .await
            .unwrap();
        // Don't drop temp_dir yet - store it in a leaked Box to keep it alive
        Box::leak(Box::new(temp_dir));
        store
    }

    #[tokio::test]
    async fn test_save_and_find_installation() {
        let store = create_test_store().await;

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
        let store = create_test_store().await;

        let mut bot = Bot::new("xoxb-token", "B12345", "U67890");
        bot.team_id = Some("T12345".to_string());
        bot.app_id = Some("A12345".to_string());

        store.save_bot(bot.clone()).await.unwrap();

        let found = store.find_bot(None, Some("T12345"), false).await.unwrap();

        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.bot_token, "xoxb-token");
        assert_eq!(found.bot_id, "B12345");
    }

    #[tokio::test]
    async fn test_delete_installation() {
        let store = create_test_store().await;

        let installation = Installation::new("U12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890")
            .app_id("A12345");

        store.save(installation).await.unwrap();

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
        let store = create_test_store().await;

        let mut bot = Bot::new("xoxb-token", "B12345", "U67890");
        bot.team_id = Some("T12345".to_string());
        bot.app_id = Some("A12345".to_string());

        store.save_bot(bot).await.unwrap();

        store.delete_bot(None, Some("T12345")).await.unwrap();

        let found = store.find_bot(None, Some("T12345"), false).await.unwrap();
        assert!(found.is_none());
    }
}
