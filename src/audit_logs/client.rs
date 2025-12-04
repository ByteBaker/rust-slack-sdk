//! Audit Logs API clients
//!
//! Provides both synchronous and asynchronous clients for accessing
//! Slack's Enterprise Grid Audit Logs API.

use crate::error::SlackError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{blocking::Client as BlockingClient, Client as AsyncClient};
use std::collections::HashMap;
use std::time::Duration;

use super::models::AuditLogsResponse;

const BASE_URL: &str = "https://api.slack.com/audit/v1/";
const DEFAULT_TIMEOUT: u64 = 30;

/// Get the SDK user agent string
fn get_user_agent() -> String {
    format!(
        "slack-rs/{} {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    )
}

/// Synchronous Audit Logs API client
///
/// # Example
///
/// ```no_run
/// use slack_rs::audit_logs::AuditLogsClient;
///
/// let client = AuditLogsClient::new("xoxp-your-token");
/// let response = client.logs(Some(100), Some("user_login"), None, None, None, None, None).unwrap();
/// ```
#[derive(Debug)]
pub struct AuditLogsClient {
    token: String,
    base_url: String,
    client: BlockingClient,
}

impl AuditLogsClient {
    /// Create a new AuditLogsClient
    ///
    /// # Arguments
    ///
    /// * `token` - Admin user token (starts with `xoxp-`)
    pub fn new(token: impl Into<String>) -> Self {
        Self::with_base_url(token, BASE_URL)
    }

    /// Create a new AuditLogsClient with a custom base URL
    ///
    /// # Arguments
    ///
    /// * `token` - Admin user token (starts with `xoxp-`)
    /// * `base_url` - Custom base URL for the API
    pub fn with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        let client = BlockingClient::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token: token.into(),
            base_url: base_url.into(),
            client,
        }
    }

    /// Returns information about the kind of objects which the Audit Logs API returns
    ///
    /// Authentication not required.
    pub fn schemas(&self) -> Result<AuditLogsResponse, SlackError> {
        self.api_call("GET", "schemas", &[])
    }

    /// Returns information about the kind of actions that the Audit Logs API returns
    ///
    /// Authentication not required.
    pub fn actions(&self) -> Result<AuditLogsResponse, SlackError> {
        self.api_call("GET", "actions", &[])
    }

    /// Retrieve audit events from your organization
    ///
    /// # Arguments
    ///
    /// * `limit` - Number of results to return (max 9999)
    /// * `action` - Name of the action to filter by
    /// * `actor` - User ID who initiated the action
    /// * `entity` - ID of the target entity
    /// * `oldest` - Unix timestamp of the least recent audit event
    /// * `latest` - Unix timestamp of the most recent audit event
    /// * `cursor` - Pagination cursor for next page
    pub fn logs(
        &self,
        limit: Option<u32>,
        action: Option<&str>,
        actor: Option<&str>,
        entity: Option<&str>,
        oldest: Option<i64>,
        latest: Option<i64>,
        cursor: Option<&str>,
    ) -> Result<AuditLogsResponse, SlackError> {
        let mut params = Vec::new();

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(action) = action {
            params.push(("action", action.to_string()));
        }
        if let Some(actor) = actor {
            params.push(("actor", actor.to_string()));
        }
        if let Some(entity) = entity {
            params.push(("entity", entity.to_string()));
        }
        if let Some(oldest) = oldest {
            params.push(("oldest", oldest.to_string()));
        }
        if let Some(latest) = latest {
            params.push(("latest", latest.to_string()));
        }
        if let Some(cursor) = cursor {
            params.push(("cursor", cursor.to_string()));
        }

        self.api_call("GET", "logs", &params)
    }

    /// Make an API call to the Audit Logs API
    fn api_call(
        &self,
        method: &str,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<AuditLogsResponse, SlackError> {
        let mut url = format!("{}{}", self.base_url, path);

        if !params.is_empty() {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&query_string);
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token)).map_err(|e| {
                SlackError::InvalidInput {
                    message: e.to_string(),
                }
            })?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&get_user_agent()).map_err(|e| SlackError::InvalidInput {
                message: e.to_string(),
            })?,
        );

        let request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            _ => {
                return Err(SlackError::InvalidInput {
                    message: format!("Unsupported method: {}", method),
                })
            }
        };

        let response = request.headers(headers.clone()).send()?;

        let status = response.status();
        let status_code = status.as_u16();

        // Extract headers
        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                response_headers.insert(key.to_string(), v.to_string());
            }
        }

        // Read body
        let body = response.text()?;

        Ok(AuditLogsResponse::new(
            url,
            status_code,
            response_headers,
            Some(body),
        ))
    }
}

/// Asynchronous Audit Logs API client
///
/// # Example
///
/// ```no_run
/// use slack_rs::audit_logs::AsyncAuditLogsClient;
///
/// #[tokio::main]
/// async fn main() {
///     let client = AsyncAuditLogsClient::new("xoxp-your-token");
///     let response = client.logs(Some(100), Some("user_login"), None, None, None, None, None).await.unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct AsyncAuditLogsClient {
    token: String,
    base_url: String,
    client: AsyncClient,
}

impl AsyncAuditLogsClient {
    /// Create a new AsyncAuditLogsClient
    ///
    /// # Arguments
    ///
    /// * `token` - Admin user token (starts with `xoxp-`)
    pub fn new(token: impl Into<String>) -> Self {
        Self::with_base_url(token, BASE_URL)
    }

    /// Create a new AsyncAuditLogsClient with a custom base URL
    ///
    /// # Arguments
    ///
    /// * `token` - Admin user token (starts with `xoxp-`)
    /// * `base_url` - Custom base URL for the API
    pub fn with_base_url(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        let client = AsyncClient::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token: token.into(),
            base_url: base_url.into(),
            client,
        }
    }

    /// Returns information about the kind of objects which the Audit Logs API returns
    ///
    /// Authentication not required.
    pub async fn schemas(&self) -> Result<AuditLogsResponse, SlackError> {
        self.api_call("GET", "schemas", &[]).await
    }

    /// Returns information about the kind of actions that the Audit Logs API returns
    ///
    /// Authentication not required.
    pub async fn actions(&self) -> Result<AuditLogsResponse, SlackError> {
        self.api_call("GET", "actions", &[]).await
    }

    /// Retrieve audit events from your organization
    ///
    /// # Arguments
    ///
    /// * `limit` - Number of results to return (max 9999)
    /// * `action` - Name of the action to filter by
    /// * `actor` - User ID who initiated the action
    /// * `entity` - ID of the target entity
    /// * `oldest` - Unix timestamp of the least recent audit event
    /// * `latest` - Unix timestamp of the most recent audit event
    /// * `cursor` - Pagination cursor for next page
    pub async fn logs(
        &self,
        limit: Option<u32>,
        action: Option<&str>,
        actor: Option<&str>,
        entity: Option<&str>,
        oldest: Option<i64>,
        latest: Option<i64>,
        cursor: Option<&str>,
    ) -> Result<AuditLogsResponse, SlackError> {
        let mut params = Vec::new();

        if let Some(limit) = limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(action) = action {
            params.push(("action", action.to_string()));
        }
        if let Some(actor) = actor {
            params.push(("actor", actor.to_string()));
        }
        if let Some(entity) = entity {
            params.push(("entity", entity.to_string()));
        }
        if let Some(oldest) = oldest {
            params.push(("oldest", oldest.to_string()));
        }
        if let Some(latest) = latest {
            params.push(("latest", latest.to_string()));
        }
        if let Some(cursor) = cursor {
            params.push(("cursor", cursor.to_string()));
        }

        self.api_call("GET", "logs", &params).await
    }

    /// Make an API call to the Audit Logs API
    async fn api_call(
        &self,
        method: &str,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<AuditLogsResponse, SlackError> {
        let mut url = format!("{}{}", self.base_url, path);

        if !params.is_empty() {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            url.push('?');
            url.push_str(&query_string);
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token)).map_err(|e| {
                SlackError::InvalidInput {
                    message: e.to_string(),
                }
            })?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&get_user_agent()).map_err(|e| SlackError::InvalidInput {
                message: e.to_string(),
            })?,
        );

        let request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            _ => {
                return Err(SlackError::InvalidInput {
                    message: format!("Unsupported method: {}", method),
                })
            }
        };

        let response = request.headers(headers.clone()).send().await?;

        let status = response.status();
        let status_code = status.as_u16();

        // Extract headers
        let mut response_headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                response_headers.insert(key.to_string(), v.to_string());
            }
        }

        // Read body
        let body = response.text().await?;

        Ok(AuditLogsResponse::new(
            url,
            status_code,
            response_headers,
            Some(body),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AuditLogsClient::new("xoxp-test-token");
        assert_eq!(client.token, "xoxp-test-token");
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn test_client_with_custom_base_url() {
        let client = AuditLogsClient::with_base_url("xoxp-test-token", "http://localhost:8888/");
        assert_eq!(client.token, "xoxp-test-token");
        assert_eq!(client.base_url, "http://localhost:8888/");
    }

    #[test]
    fn test_async_client_creation() {
        let client = AsyncAuditLogsClient::new("xoxp-test-token");
        assert_eq!(client.token, "xoxp-test-token");
        assert_eq!(client.base_url, BASE_URL);
    }

    #[test]
    fn test_get_user_agent() {
        let ua = get_user_agent();
        assert!(ua.contains("slack-rs"));
        assert!(ua.contains(env!("CARGO_PKG_VERSION")));
    }
}
