//! OAuth authorize URL generation
//!
//! This module provides utilities for generating OAuth authorization URLs
//! for the Slack OAuth flow.

use url::Url;

/// Generates OAuth v2 authorization URLs
///
/// Used to create the URL that users visit to authorize your Slack app.
///
/// # Example
///
/// ```
/// use slack_rs::oauth::AuthorizeUrlGenerator;
///
/// let generator = AuthorizeUrlGenerator::new("client_id_123")
///     .scopes(vec!["chat:write".to_string(), "channels:read".to_string()])
///     .redirect_uri("https://example.com/oauth/callback");
///
/// let url = generator.generate("random_state_string", None);
/// ```
#[derive(Debug, Clone)]
pub struct AuthorizeUrlGenerator {
    client_id: String,
    redirect_uri: Option<String>,
    scopes: Vec<String>,
    user_scopes: Vec<String>,
    authorization_url: String,
}

impl AuthorizeUrlGenerator {
    /// Creates a new AuthorizeUrlGenerator
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            redirect_uri: None,
            scopes: Vec::new(),
            user_scopes: Vec::new(),
            authorization_url: "https://slack.com/oauth/v2/authorize".to_string(),
        }
    }

    /// Sets the redirect URI
    pub fn redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    /// Sets the bot scopes
    pub fn scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Sets the user scopes
    pub fn user_scopes(mut self, user_scopes: Vec<String>) -> Self {
        self.user_scopes = user_scopes;
        self
    }

    /// Sets a custom authorization URL (for testing or custom Slack instances)
    pub fn authorization_url(mut self, url: impl Into<String>) -> Self {
        self.authorization_url = url.into();
        self
    }

    /// Generates the authorization URL
    ///
    /// # Arguments
    ///
    /// * `state` - A random string to prevent CSRF attacks
    /// * `team` - Optional team ID to pre-select a workspace
    pub fn generate(&self, state: impl Into<String>, team: Option<String>) -> String {
        let mut url = Url::parse(&self.authorization_url).expect("Invalid authorization URL");

        let scopes = self.scopes.join(",");
        let user_scopes = self.user_scopes.join(",");

        url.query_pairs_mut()
            .append_pair("state", &state.into())
            .append_pair("client_id", &self.client_id)
            .append_pair("scope", &scopes)
            .append_pair("user_scope", &user_scopes);

        if let Some(redirect_uri) = &self.redirect_uri {
            url.query_pairs_mut()
                .append_pair("redirect_uri", redirect_uri);
        }

        if let Some(team) = team {
            url.query_pairs_mut().append_pair("team", &team);
        }

        url.to_string()
    }
}

/// Generates OpenID Connect authorization URLs
///
/// Used for OpenID Connect authentication with Slack.
///
/// # Example
///
/// ```
/// use slack_rs::oauth::OpenIDConnectAuthorizeUrlGenerator;
///
/// let generator = OpenIDConnectAuthorizeUrlGenerator::new(
///     "client_id_123",
///     "https://example.com/oauth/callback"
/// )
/// .scopes(vec!["openid".to_string(), "profile".to_string()]);
///
/// let url = generator.generate("random_state_string", Some("nonce_value"), None);
/// ```
#[derive(Debug, Clone)]
pub struct OpenIDConnectAuthorizeUrlGenerator {
    client_id: String,
    redirect_uri: String,
    scopes: Vec<String>,
    authorization_url: String,
}

impl OpenIDConnectAuthorizeUrlGenerator {
    /// Creates a new OpenIDConnectAuthorizeUrlGenerator
    pub fn new(client_id: impl Into<String>, redirect_uri: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            redirect_uri: redirect_uri.into(),
            scopes: Vec::new(),
            authorization_url: "https://slack.com/openid/connect/authorize".to_string(),
        }
    }

    /// Sets the scopes
    pub fn scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Sets a custom authorization URL (for testing or custom Slack instances)
    pub fn authorization_url(mut self, url: impl Into<String>) -> Self {
        self.authorization_url = url.into();
        self
    }

    /// Generates the authorization URL
    ///
    /// # Arguments
    ///
    /// * `state` - A random string to prevent CSRF attacks
    /// * `nonce` - Optional nonce for additional security
    /// * `team` - Optional team ID to pre-select a workspace
    pub fn generate(
        &self,
        state: impl Into<String>,
        nonce: Option<String>,
        team: Option<String>,
    ) -> String {
        let mut url = Url::parse(&self.authorization_url).expect("Invalid authorization URL");

        let scopes = self.scopes.join(",");

        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("state", &state.into())
            .append_pair("client_id", &self.client_id)
            .append_pair("scope", &scopes)
            .append_pair("redirect_uri", &self.redirect_uri);

        if let Some(team) = team {
            url.query_pairs_mut().append_pair("team", &team);
        }

        if let Some(nonce) = nonce {
            url.query_pairs_mut().append_pair("nonce", &nonce);
        }

        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authorize_url_generator_basic() {
        let generator = AuthorizeUrlGenerator::new("client_123");
        let url = generator.generate("state_456", None);

        assert!(url.contains("client_id=client_123"));
        assert!(url.contains("state=state_456"));
        assert!(url.starts_with("https://slack.com/oauth/v2/authorize"));
    }

    #[test]
    fn test_authorize_url_generator_with_scopes() {
        let generator = AuthorizeUrlGenerator::new("client_123")
            .scopes(vec!["chat:write".to_string(), "channels:read".to_string()]);

        let url = generator.generate("state_456", None);

        assert!(url.contains("scope=chat%3Awrite%2Cchannels%3Aread"));
    }

    #[test]
    fn test_authorize_url_generator_with_user_scopes() {
        let generator =
            AuthorizeUrlGenerator::new("client_123").user_scopes(vec!["search:read".to_string()]);

        let url = generator.generate("state_456", None);

        assert!(url.contains("user_scope=search%3Aread"));
    }

    #[test]
    fn test_authorize_url_generator_with_redirect_uri() {
        let generator =
            AuthorizeUrlGenerator::new("client_123").redirect_uri("https://example.com/callback");

        let url = generator.generate("state_456", None);

        assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
    }

    #[test]
    fn test_authorize_url_generator_with_team() {
        let generator = AuthorizeUrlGenerator::new("client_123");
        let url = generator.generate("state_456", Some("T12345".to_string()));

        assert!(url.contains("team=T12345"));
    }

    #[test]
    fn test_authorize_url_generator_custom_url() {
        let generator = AuthorizeUrlGenerator::new("client_123")
            .authorization_url("https://custom.slack.com/oauth/authorize");

        let url = generator.generate("state_456", None);

        assert!(url.starts_with("https://custom.slack.com/oauth/authorize"));
    }

    #[test]
    fn test_openid_connect_generator_basic() {
        let generator =
            OpenIDConnectAuthorizeUrlGenerator::new("client_123", "https://example.com/callback");

        let url = generator.generate("state_456", None, None);

        assert!(url.contains("response_type=code"));
        assert!(url.contains("client_id=client_123"));
        assert!(url.contains("state=state_456"));
        assert!(url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback"));
        assert!(url.starts_with("https://slack.com/openid/connect/authorize"));
    }

    #[test]
    fn test_openid_connect_generator_with_scopes() {
        let generator =
            OpenIDConnectAuthorizeUrlGenerator::new("client_123", "https://example.com/callback")
                .scopes(vec!["openid".to_string(), "profile".to_string()]);

        let url = generator.generate("state_456", None, None);

        assert!(url.contains("scope=openid%2Cprofile"));
    }

    #[test]
    fn test_openid_connect_generator_with_nonce() {
        let generator =
            OpenIDConnectAuthorizeUrlGenerator::new("client_123", "https://example.com/callback");

        let url = generator.generate("state_456", Some("nonce_789".to_string()), None);

        assert!(url.contains("nonce=nonce_789"));
    }

    #[test]
    fn test_openid_connect_generator_with_team() {
        let generator =
            OpenIDConnectAuthorizeUrlGenerator::new("client_123", "https://example.com/callback");

        let url = generator.generate("state_456", None, Some("T12345".to_string()));

        assert!(url.contains("team=T12345"));
    }
}
