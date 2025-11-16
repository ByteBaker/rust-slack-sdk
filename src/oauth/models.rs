//! OAuth models for Slack installations
//!
//! This module provides types for managing Slack OAuth installations, including
//! bot and user tokens, scopes, and refresh tokens for token rotation.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete Slack app installation
///
/// Contains all information about an app installation, including bot and user tokens,
/// scopes, refresh tokens, and incoming webhook configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Installation {
    /// The app ID
    pub app_id: Option<String>,

    /// Enterprise Grid organization ID
    pub enterprise_id: Option<String>,

    /// Enterprise Grid organization name
    pub enterprise_name: Option<String>,

    /// Enterprise Grid organization URL
    pub enterprise_url: Option<String>,

    /// Workspace/team ID
    pub team_id: Option<String>,

    /// Workspace/team name
    pub team_name: Option<String>,

    /// Bot access token
    pub bot_token: Option<String>,

    /// Bot ID
    pub bot_id: Option<String>,

    /// Bot user ID
    pub bot_user_id: Option<String>,

    /// Bot OAuth scopes
    pub bot_scopes: Option<Vec<String>>,

    /// Bot refresh token (when token rotation is enabled)
    pub bot_refresh_token: Option<String>,

    /// Bot token expiration timestamp (when token rotation is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token_expires_at: Option<i64>,

    /// User ID who installed the app
    pub user_id: String,

    /// User access token
    pub user_token: Option<String>,

    /// User OAuth scopes
    pub user_scopes: Option<Vec<String>>,

    /// User refresh token (when token rotation is enabled)
    pub user_refresh_token: Option<String>,

    /// User token expiration timestamp (when token rotation is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_token_expires_at: Option<i64>,

    /// Incoming webhook URL
    pub incoming_webhook_url: Option<String>,

    /// Incoming webhook channel name
    pub incoming_webhook_channel: Option<String>,

    /// Incoming webhook channel ID
    pub incoming_webhook_channel_id: Option<String>,

    /// Incoming webhook configuration URL
    pub incoming_webhook_configuration_url: Option<String>,

    /// Whether this is an Enterprise Grid organization-wide install
    pub is_enterprise_install: bool,

    /// Token type (typically "bot")
    pub token_type: Option<String>,

    /// Installation timestamp (Unix timestamp in seconds)
    pub installed_at: f64,

    /// Custom values for application-specific data
    #[serde(flatten)]
    pub custom_values: HashMap<String, serde_json::Value>,
}

impl Installation {
    /// Creates a new Installation with required fields
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            app_id: None,
            enterprise_id: None,
            enterprise_name: None,
            enterprise_url: None,
            team_id: None,
            team_name: None,
            bot_token: None,
            bot_id: None,
            bot_user_id: None,
            bot_scopes: None,
            bot_refresh_token: None,
            bot_token_expires_at: None,
            user_id: user_id.into(),
            user_token: None,
            user_scopes: None,
            user_refresh_token: None,
            user_token_expires_at: None,
            incoming_webhook_url: None,
            incoming_webhook_channel: None,
            incoming_webhook_channel_id: None,
            incoming_webhook_configuration_url: None,
            is_enterprise_install: false,
            token_type: None,
            installed_at: Utc::now().timestamp() as f64,
            custom_values: HashMap::new(),
        }
    }

    /// Builder method to set app_id
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// Builder method to set enterprise_id
    pub fn enterprise_id(mut self, enterprise_id: impl Into<String>) -> Self {
        self.enterprise_id = Some(enterprise_id.into());
        self
    }

    /// Builder method to set team_id
    pub fn team_id(mut self, team_id: impl Into<String>) -> Self {
        self.team_id = Some(team_id.into());
        self
    }

    /// Builder method to set bot_token
    pub fn bot_token(mut self, bot_token: impl Into<String>) -> Self {
        self.bot_token = Some(bot_token.into());
        self
    }

    /// Builder method to set bot_id
    pub fn bot_id(mut self, bot_id: impl Into<String>) -> Self {
        self.bot_id = Some(bot_id.into());
        self
    }

    /// Builder method to set bot_user_id
    pub fn bot_user_id(mut self, bot_user_id: impl Into<String>) -> Self {
        self.bot_user_id = Some(bot_user_id.into());
        self
    }

    /// Builder method to set bot_scopes
    pub fn bot_scopes(mut self, scopes: Vec<String>) -> Self {
        self.bot_scopes = Some(scopes);
        self
    }

    /// Builder method to set user_token
    pub fn user_token(mut self, user_token: impl Into<String>) -> Self {
        self.user_token = Some(user_token.into());
        self
    }

    /// Builder method to set is_enterprise_install
    pub fn is_enterprise_install(mut self, is_enterprise_install: bool) -> Self {
        self.is_enterprise_install = is_enterprise_install;
        self
    }

    /// Converts this Installation to a Bot
    pub fn to_bot(&self) -> Option<Bot> {
        if self.bot_token.is_none() || self.bot_id.is_none() || self.bot_user_id.is_none() {
            return None;
        }

        Some(Bot {
            app_id: self.app_id.clone(),
            enterprise_id: self.enterprise_id.clone(),
            enterprise_name: self.enterprise_name.clone(),
            team_id: self.team_id.clone(),
            team_name: self.team_name.clone(),
            bot_token: self.bot_token.clone().unwrap(),
            bot_id: self.bot_id.clone().unwrap(),
            bot_user_id: self.bot_user_id.clone().unwrap(),
            bot_scopes: self.bot_scopes.clone().unwrap_or_default(),
            bot_refresh_token: self.bot_refresh_token.clone(),
            bot_token_expires_at: self.bot_token_expires_at,
            is_enterprise_install: self.is_enterprise_install,
            installed_at: self.installed_at,
            custom_values: self.custom_values.clone(),
        })
    }

    /// Set a custom value
    pub fn set_custom_value(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.custom_values.insert(name.into(), value);
    }

    /// Get a custom value
    pub fn get_custom_value(&self, name: &str) -> Option<&serde_json::Value> {
        self.custom_values.get(name)
    }
}

/// Represents bot-specific installation data
///
/// A subset of Installation containing only bot-related information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bot {
    /// The app ID
    pub app_id: Option<String>,

    /// Enterprise Grid organization ID
    pub enterprise_id: Option<String>,

    /// Enterprise Grid organization name
    pub enterprise_name: Option<String>,

    /// Workspace/team ID
    pub team_id: Option<String>,

    /// Workspace/team name
    pub team_name: Option<String>,

    /// Bot access token
    pub bot_token: String,

    /// Bot ID
    pub bot_id: String,

    /// Bot user ID
    pub bot_user_id: String,

    /// Bot OAuth scopes
    pub bot_scopes: Vec<String>,

    /// Bot refresh token (when token rotation is enabled)
    pub bot_refresh_token: Option<String>,

    /// Bot token expiration timestamp (when token rotation is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_token_expires_at: Option<i64>,

    /// Whether this is an Enterprise Grid organization-wide install
    pub is_enterprise_install: bool,

    /// Installation timestamp (Unix timestamp in seconds)
    pub installed_at: f64,

    /// Custom values for application-specific data
    #[serde(flatten)]
    pub custom_values: HashMap<String, serde_json::Value>,
}

impl Bot {
    /// Creates a new Bot with required fields
    pub fn new(
        bot_token: impl Into<String>,
        bot_id: impl Into<String>,
        bot_user_id: impl Into<String>,
    ) -> Self {
        Self {
            app_id: None,
            enterprise_id: None,
            enterprise_name: None,
            team_id: None,
            team_name: None,
            bot_token: bot_token.into(),
            bot_id: bot_id.into(),
            bot_user_id: bot_user_id.into(),
            bot_scopes: Vec::new(),
            bot_refresh_token: None,
            bot_token_expires_at: None,
            is_enterprise_install: false,
            installed_at: Utc::now().timestamp() as f64,
            custom_values: HashMap::new(),
        }
    }

    /// Set a custom value
    pub fn set_custom_value(&mut self, name: impl Into<String>, value: serde_json::Value) {
        self.custom_values.insert(name.into(), value);
    }

    /// Get a custom value
    pub fn get_custom_value(&self, name: &str) -> Option<&serde_json::Value> {
        self.custom_values.get(name)
    }
}

/// Response from OAuth v2 authorization
///
/// Contains the response data from completing the OAuth flow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthV2Response {
    /// Whether the request was successful
    pub ok: bool,

    /// Access token (for token rotation: bot or user token)
    pub access_token: Option<String>,

    /// Token type (typically "bot")
    pub token_type: Option<String>,

    /// Scopes granted to the access token
    pub scope: Option<String>,

    /// Bot user ID
    pub bot_user_id: Option<String>,

    /// App ID
    pub app_id: Option<String>,

    /// Team information
    pub team: Option<OAuthV2Team>,

    /// Enterprise information
    pub enterprise: Option<OAuthV2Enterprise>,

    /// Authed user information
    pub authed_user: Option<OAuthV2AuthedUser>,

    /// Incoming webhook information
    pub incoming_webhook: Option<OAuthV2IncomingWebhook>,

    /// Refresh token (when token rotation is enabled)
    pub refresh_token: Option<String>,

    /// Token expiration time in seconds (when token rotation is enabled)
    pub expires_in: Option<i64>,

    /// Whether this is an Enterprise Grid organization-wide install
    pub is_enterprise_install: Option<bool>,

    /// Error message if ok is false
    pub error: Option<String>,
}

/// Team information in OAuth response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthV2Team {
    pub id: String,
    pub name: String,
}

/// Enterprise information in OAuth response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthV2Enterprise {
    pub id: String,
    pub name: String,
}

/// Authenticated user information in OAuth response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthV2AuthedUser {
    pub id: String,
    pub scope: Option<String>,
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
}

/// Incoming webhook information in OAuth response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OAuthV2IncomingWebhook {
    pub channel: String,
    pub channel_id: String,
    pub configuration_url: String,
    pub url: String,
}

impl OAuthV2Response {
    /// Converts this OAuth response to an Installation
    pub fn to_installation(&self) -> Option<Installation> {
        let authed_user = self.authed_user.as_ref()?;

        let mut installation = Installation::new(authed_user.id.clone());

        installation.app_id = self.app_id.clone();
        installation.token_type = self.token_type.clone();
        installation.is_enterprise_install = self.is_enterprise_install.unwrap_or(false);

        // Set enterprise info
        if let Some(enterprise) = &self.enterprise {
            installation.enterprise_id = Some(enterprise.id.clone());
            installation.enterprise_name = Some(enterprise.name.clone());
        }

        // Set team info
        if let Some(team) = &self.team {
            installation.team_id = Some(team.id.clone());
            installation.team_name = Some(team.name.clone());
        }

        // Set bot info
        if let Some(access_token) = &self.access_token {
            installation.bot_token = Some(access_token.clone());
            installation.bot_user_id = self.bot_user_id.clone();

            if let Some(scope) = &self.scope {
                installation.bot_scopes =
                    Some(scope.split(',').map(|s| s.trim().to_string()).collect());
            }

            installation.bot_refresh_token = self.refresh_token.clone();
            if let Some(expires_in) = self.expires_in {
                installation.bot_token_expires_at = Some(Utc::now().timestamp() + expires_in);
            }
        }

        // Set user info
        installation.user_token = authed_user.access_token.clone();
        if let Some(scope) = &authed_user.scope {
            installation.user_scopes =
                Some(scope.split(',').map(|s| s.trim().to_string()).collect());
        }
        installation.user_refresh_token = authed_user.refresh_token.clone();
        if let Some(expires_in) = authed_user.expires_in {
            installation.user_token_expires_at = Some(Utc::now().timestamp() + expires_in);
        }

        // Set incoming webhook info
        if let Some(webhook) = &self.incoming_webhook {
            installation.incoming_webhook_url = Some(webhook.url.clone());
            installation.incoming_webhook_channel = Some(webhook.channel.clone());
            installation.incoming_webhook_channel_id = Some(webhook.channel_id.clone());
            installation.incoming_webhook_configuration_url =
                Some(webhook.configuration_url.clone());
        }

        Some(installation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installation_new() {
        let installation = Installation::new("U12345");
        assert_eq!(installation.user_id, "U12345");
        assert!(!installation.is_enterprise_install);
        assert!(installation.installed_at > 0.0);
    }

    #[test]
    fn test_installation_builder() {
        let installation = Installation::new("U12345")
            .app_id("A12345")
            .team_id("T12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890")
            .is_enterprise_install(false);

        assert_eq!(installation.app_id, Some("A12345".to_string()));
        assert_eq!(installation.team_id, Some("T12345".to_string()));
        assert_eq!(installation.bot_token, Some("xoxb-token".to_string()));
    }

    #[test]
    fn test_installation_to_bot() {
        let installation = Installation::new("U12345")
            .bot_token("xoxb-token")
            .bot_id("B12345")
            .bot_user_id("U67890")
            .bot_scopes(vec!["chat:write".to_string()]);

        let bot = installation.to_bot();
        assert!(bot.is_some());

        let bot = bot.unwrap();
        assert_eq!(bot.bot_token, "xoxb-token");
        assert_eq!(bot.bot_id, "B12345");
        assert_eq!(bot.bot_scopes, vec!["chat:write"]);
    }

    #[test]
    fn test_installation_to_bot_missing_fields() {
        let installation = Installation::new("U12345");
        assert!(installation.to_bot().is_none());
    }

    #[test]
    fn test_installation_custom_values() {
        let mut installation = Installation::new("U12345");
        installation.set_custom_value("key1", serde_json::json!("value1"));

        assert_eq!(
            installation.get_custom_value("key1"),
            Some(&serde_json::json!("value1"))
        );
    }

    #[test]
    fn test_bot_new() {
        let bot = Bot::new("xoxb-token", "B12345", "U67890");
        assert_eq!(bot.bot_token, "xoxb-token");
        assert_eq!(bot.bot_id, "B12345");
        assert_eq!(bot.bot_user_id, "U67890");
        assert!(bot.installed_at > 0.0);
    }

    #[test]
    fn test_oauth_response_to_installation() {
        let response = OAuthV2Response {
            ok: true,
            access_token: Some("xoxb-token".to_string()),
            token_type: Some("bot".to_string()),
            scope: Some("chat:write,channels:read".to_string()),
            bot_user_id: Some("U67890".to_string()),
            app_id: Some("A12345".to_string()),
            team: Some(OAuthV2Team {
                id: "T12345".to_string(),
                name: "My Team".to_string(),
            }),
            enterprise: None,
            authed_user: Some(OAuthV2AuthedUser {
                id: "U12345".to_string(),
                scope: Some("search:read".to_string()),
                access_token: Some("xoxp-token".to_string()),
                token_type: Some("user".to_string()),
                refresh_token: None,
                expires_in: None,
            }),
            incoming_webhook: None,
            refresh_token: None,
            expires_in: None,
            is_enterprise_install: Some(false),
            error: None,
        };

        let installation = response.to_installation();
        assert!(installation.is_some());

        let installation = installation.unwrap();
        assert_eq!(installation.user_id, "U12345");
        assert_eq!(installation.bot_token, Some("xoxb-token".to_string()));
        assert_eq!(installation.team_id, Some("T12345".to_string()));
        assert_eq!(
            installation.bot_scopes,
            Some(vec!["chat:write".to_string(), "channels:read".to_string()])
        );
    }

    #[test]
    fn test_installation_serialization() {
        let installation = Installation::new("U12345")
            .app_id("A12345")
            .team_id("T12345")
            .bot_token("xoxb-token");

        let json = serde_json::to_string(&installation).unwrap();
        let deserialized: Installation = serde_json::from_str(&json).unwrap();

        assert_eq!(installation.user_id, deserialized.user_id);
        assert_eq!(installation.app_id, deserialized.app_id);
        assert_eq!(installation.team_id, deserialized.team_id);
    }
}
