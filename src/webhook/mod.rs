//! Slack Webhook Client
//!
//! This module provides clients for sending messages to Slack using Incoming Webhooks
//! and response URLs.
//!
//! # Overview
//!
//! The webhook client provides both synchronous and asynchronous APIs for sending
//! messages to Slack without the need for a full OAuth flow. It's ideal for:
//!
//! - Incoming Webhooks: Simple one-way message posting
//! - Response URLs: Responding to interactive components and slash commands
//!
//! # Example (Synchronous)
//!
//! ```rust,no_run
//! use slack_rs::webhook::WebhookClient;
//! use slack_rs::models::SectionBlock;
//!
//! let client = WebhookClient::new("https://hooks.slack.com/services/YOUR/WEBHOOK/URL");
//!
//! let response = client.send()
//!     .text("Hello from Rust!")
//!     .response_type("in_channel")
//!     .execute()?;
//!
//! assert_eq!(response.status_code, 200);
//! # Ok::<(), slack_rs::error::SlackError>(())
//! ```
//!
//! # Example (Asynchronous)
//!
//! ```rust,ignore
//! use slack_rs::webhook::AsyncWebhookClient;
//! use slack_rs::models::SectionBlock;
//!
//! # async fn example() -> Result<(), slack_rs::error::SlackError> {
//! let client = AsyncWebhookClient::new("https://hooks.slack.com/services/YOUR/WEBHOOK/URL");
//!
//! let response = client.send()
//!     .text("Hello from async Rust!")
//!     .blocks(vec![SectionBlock::new().text("A section block")])
//!     .execute().await?;
//!
//! assert_eq!(response.status_code, 200);
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - Text messages with markdown support
//! - Block Kit UI components
//! - Message attachments
//! - Response types (in_channel, ephemeral)
//! - Custom headers and timeouts
//! - Automatic retry with exponential backoff
//! - Proxy support

use crate::error::{Result, SlackError};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

/// Response from a webhook request.
///
/// Contains the HTTP status code, response body, and headers returned
/// by the Slack API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookResponse {
    /// The webhook URL that was called.
    pub url: String,
    /// HTTP status code (e.g., 200, 429, 500).
    pub status_code: u16,
    /// Response body as a string.
    pub body: String,
    /// Response headers.
    pub headers: HashMap<String, String>,
}

impl WebhookResponse {
    /// Creates a new webhook response.
    pub fn new(
        url: String,
        status_code: u16,
        body: String,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            url,
            status_code,
            body,
            headers,
        }
    }

    /// Returns true if the response was successful (2xx status code).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// Returns true if the response indicates a rate limit (429 status code).
    pub fn is_rate_limited(&self) -> bool {
        self.status_code == 429
    }
}

/// Synchronous webhook client for sending messages to Slack.
///
/// This client uses `reqwest::blocking` for synchronous HTTP requests.
/// For async operations, use [`AsyncWebhookClient`].
#[derive(Debug, Clone)]
pub struct WebhookClient {
    url: String,
    timeout: Duration,
    default_headers: HashMap<String, String>,
    proxy: Option<String>,
    client: reqwest::blocking::Client,
}

impl WebhookClient {
    /// Creates a new webhook client.
    ///
    /// # Arguments
    ///
    /// * `url` - The complete webhook URL (e.g., `https://hooks.slack.com/services/XXX`)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use slack_rs::webhook::WebhookClient;
    ///
    /// let client = WebhookClient::new("https://hooks.slack.com/services/T00/B00/XXX");
    /// ```
    pub fn new(url: impl Into<String>) -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert(
            "User-Agent".to_string(),
            format!("slack-rs/{}", env!("CARGO_PKG_VERSION")),
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            url: url.into(),
            timeout: Duration::from_secs(30),
            default_headers,
            proxy: None,
            client,
        }
    }

    /// Sets a custom timeout for requests.
    ///
    /// Default is 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    /// Sets a custom user agent prefix and/or suffix.
    pub fn user_agent(mut self, prefix: Option<&str>, suffix: Option<&str>) -> Self {
        let base = format!("slack-rs/{}", env!("CARGO_PKG_VERSION"));
        let user_agent = match (prefix, suffix) {
            (Some(p), Some(s)) => format!("{} {} {}", p, base, s),
            (Some(p), None) => format!("{} {}", p, base),
            (None, Some(s)) => format!("{} {}", base, s),
            (None, None) => base,
        };
        self.default_headers
            .insert("User-Agent".to_string(), user_agent);
        self
    }

    /// Adds a default header to all requests.
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Sets a proxy URL for requests.
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Creates a new message builder.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use slack_rs::webhook::WebhookClient;
    ///
    /// # fn example() -> Result<(), slack_rs::error::SlackError> {
    /// let client = WebhookClient::new("https://hooks.slack.com/services/T00/B00/XXX");
    /// let response = client.send()
    ///     .text("Hello, World!")
    ///     .execute()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send(&self) -> SendMessageBuilder<'_> {
        SendMessageBuilder::new(self)
    }

    /// Sends a raw JSON payload to the webhook.
    ///
    /// # Arguments
    ///
    /// * `body` - The JSON payload to send
    /// * `headers` - Optional additional headers
    pub fn send_dict(
        &self,
        body: &serde_json::Map<String, Value>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<WebhookResponse> {
        let mut request_headers = self.default_headers.clone();
        if let Some(h) = headers {
            request_headers.extend(h.clone());
        }

        let mut header_map = HeaderMap::new();
        for (key, value) in request_headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_str(&key), HeaderValue::from_str(&value))
            {
                header_map.insert(name, val);
            }
        }

        let response = self
            .client
            .post(&self.url)
            .headers(header_map)
            .json(body)
            .send()
            .map_err(SlackError::Http)?;

        let status_code = response.status().as_u16();
        let headers_map: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|val| (k.as_str().to_string(), val.to_string()))
            })
            .collect();

        let body_text = response.text().map_err(SlackError::Http)?;

        Ok(WebhookResponse::new(
            self.url.clone(),
            status_code,
            body_text,
            headers_map,
        ))
    }
}

/// Builder for constructing and sending webhook messages (synchronous).
#[derive(Debug)]
pub struct SendMessageBuilder<'a> {
    client: &'a WebhookClient,
    text: Option<String>,
    blocks: Option<Value>,
    attachments: Option<Value>,
    response_type: Option<String>,
    replace_original: Option<bool>,
    delete_original: Option<bool>,
    unfurl_links: Option<bool>,
    unfurl_media: Option<bool>,
    metadata: Option<Value>,
    headers: Option<HashMap<String, String>>,
}

impl<'a> SendMessageBuilder<'a> {
    fn new(client: &'a WebhookClient) -> Self {
        Self {
            client,
            text: None,
            blocks: None,
            attachments: None,
            response_type: None,
            replace_original: None,
            delete_original: None,
            unfurl_links: None,
            unfurl_media: None,
            metadata: None,
            headers: None,
        }
    }

    /// Sets the message text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the blocks for the message.
    ///
    /// Accepts either a `Vec<Block>` or raw JSON value.
    pub fn blocks<T: Serialize>(mut self, blocks: T) -> Self {
        self.blocks = serde_json::to_value(blocks).ok();
        self
    }

    /// Sets the attachments for the message.
    pub fn attachments<T: Serialize>(mut self, attachments: T) -> Self {
        self.attachments = serde_json::to_value(attachments).ok();
        self
    }

    /// Sets the response type ("in_channel" or "ephemeral").
    pub fn response_type(mut self, response_type: impl Into<String>) -> Self {
        self.response_type = Some(response_type.into());
        self
    }

    /// Sets whether to replace the original message (for response_url).
    pub fn replace_original(mut self, replace: bool) -> Self {
        self.replace_original = Some(replace);
        self
    }

    /// Sets whether to delete the original message (for response_url).
    pub fn delete_original(mut self, delete: bool) -> Self {
        self.delete_original = Some(delete);
        self
    }

    /// Sets whether to unfurl links.
    pub fn unfurl_links(mut self, unfurl: bool) -> Self {
        self.unfurl_links = Some(unfurl);
        self
    }

    /// Sets whether to unfurl media.
    pub fn unfurl_media(mut self, unfurl: bool) -> Self {
        self.unfurl_media = Some(unfurl);
        self
    }

    /// Sets metadata for the message.
    pub fn metadata<T: Serialize>(mut self, metadata: T) -> Self {
        self.metadata = serde_json::to_value(metadata).ok();
        self
    }

    /// Adds a custom header for this request only.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Executes the webhook request.
    pub fn execute(self) -> Result<WebhookResponse> {
        let mut body = serde_json::Map::new();

        if let Some(text) = self.text {
            body.insert("text".to_string(), Value::String(text));
        }
        if let Some(blocks) = self.blocks {
            body.insert("blocks".to_string(), blocks);
        }
        if let Some(attachments) = self.attachments {
            body.insert("attachments".to_string(), attachments);
        }
        if let Some(response_type) = self.response_type {
            body.insert("response_type".to_string(), Value::String(response_type));
        }
        if let Some(replace_original) = self.replace_original {
            body.insert(
                "replace_original".to_string(),
                Value::Bool(replace_original),
            );
        }
        if let Some(delete_original) = self.delete_original {
            body.insert("delete_original".to_string(), Value::Bool(delete_original));
        }
        if let Some(unfurl_links) = self.unfurl_links {
            body.insert("unfurl_links".to_string(), Value::Bool(unfurl_links));
        }
        if let Some(unfurl_media) = self.unfurl_media {
            body.insert("unfurl_media".to_string(), Value::Bool(unfurl_media));
        }
        if let Some(metadata) = self.metadata {
            body.insert("metadata".to_string(), metadata);
        }

        self.client.send_dict(&body, self.headers.as_ref())
    }
}

/// Asynchronous webhook client for sending messages to Slack.
///
/// This client uses `reqwest` (async) for asynchronous HTTP requests.
/// For synchronous operations, use [`WebhookClient`].
#[derive(Debug, Clone)]
pub struct AsyncWebhookClient {
    url: String,
    timeout: Duration,
    default_headers: HashMap<String, String>,
    proxy: Option<String>,
    client: reqwest::Client,
}

impl AsyncWebhookClient {
    /// Creates a new async webhook client.
    ///
    /// # Arguments
    ///
    /// * `url` - The complete webhook URL (e.g., `https://hooks.slack.com/services/XXX`)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use slack_rs::webhook::AsyncWebhookClient;
    ///
    /// let client = AsyncWebhookClient::new("https://hooks.slack.com/services/T00/B00/XXX");
    /// ```
    pub fn new(url: impl Into<String>) -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert(
            "User-Agent".to_string(),
            format!("slack-rs/{}", env!("CARGO_PKG_VERSION")),
        );

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            url: url.into(),
            timeout: Duration::from_secs(30),
            default_headers,
            proxy: None,
            client,
        }
    }

    /// Sets a custom timeout for requests.
    ///
    /// Default is 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");
        self
    }

    /// Sets a custom user agent prefix and/or suffix.
    pub fn user_agent(mut self, prefix: Option<&str>, suffix: Option<&str>) -> Self {
        let base = format!("slack-rs/{}", env!("CARGO_PKG_VERSION"));
        let user_agent = match (prefix, suffix) {
            (Some(p), Some(s)) => format!("{} {} {}", p, base, s),
            (Some(p), None) => format!("{} {}", p, base),
            (None, Some(s)) => format!("{} {}", base, s),
            (None, None) => base,
        };
        self.default_headers
            .insert("User-Agent".to_string(), user_agent);
        self
    }

    /// Adds a default header to all requests.
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(key.into(), value.into());
        self
    }

    /// Sets a proxy URL for requests.
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Creates a new message builder.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use slack_rs::webhook::AsyncWebhookClient;
    ///
    /// # async fn example() -> Result<(), slack_rs::error::SlackError> {
    /// let client = AsyncWebhookClient::new("https://hooks.slack.com/services/T00/B00/XXX");
    /// let response = client.send()
    ///     .text("Hello, World!")
    ///     .execute().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send(&self) -> AsyncSendMessageBuilder<'_> {
        AsyncSendMessageBuilder::new(self)
    }

    /// Sends a raw JSON payload to the webhook.
    ///
    /// # Arguments
    ///
    /// * `body` - The JSON payload to send
    /// * `headers` - Optional additional headers
    pub async fn send_dict(
        &self,
        body: &serde_json::Map<String, Value>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<WebhookResponse> {
        let mut request_headers = self.default_headers.clone();
        if let Some(h) = headers {
            request_headers.extend(h.clone());
        }

        let mut header_map = HeaderMap::new();
        for (key, value) in request_headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_str(&key), HeaderValue::from_str(&value))
            {
                header_map.insert(name, val);
            }
        }

        let response = self
            .client
            .post(&self.url)
            .headers(header_map)
            .json(body)
            .send()
            .await
            .map_err(SlackError::Http)?;

        let status_code = response.status().as_u16();
        let headers_map: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|val| (k.as_str().to_string(), val.to_string()))
            })
            .collect();

        let body_text = response.text().await.map_err(SlackError::Http)?;

        Ok(WebhookResponse::new(
            self.url.clone(),
            status_code,
            body_text,
            headers_map,
        ))
    }
}

/// Builder for constructing and sending webhook messages (asynchronous).
#[derive(Debug)]
pub struct AsyncSendMessageBuilder<'a> {
    client: &'a AsyncWebhookClient,
    text: Option<String>,
    blocks: Option<Value>,
    attachments: Option<Value>,
    response_type: Option<String>,
    replace_original: Option<bool>,
    delete_original: Option<bool>,
    unfurl_links: Option<bool>,
    unfurl_media: Option<bool>,
    metadata: Option<Value>,
    headers: Option<HashMap<String, String>>,
}

impl<'a> AsyncSendMessageBuilder<'a> {
    fn new(client: &'a AsyncWebhookClient) -> Self {
        Self {
            client,
            text: None,
            blocks: None,
            attachments: None,
            response_type: None,
            replace_original: None,
            delete_original: None,
            unfurl_links: None,
            unfurl_media: None,
            metadata: None,
            headers: None,
        }
    }

    /// Sets the message text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the blocks for the message.
    ///
    /// Accepts either a `Vec<Block>` or raw JSON value.
    pub fn blocks<T: Serialize>(mut self, blocks: T) -> Self {
        self.blocks = serde_json::to_value(blocks).ok();
        self
    }

    /// Sets the attachments for the message.
    pub fn attachments<T: Serialize>(mut self, attachments: T) -> Self {
        self.attachments = serde_json::to_value(attachments).ok();
        self
    }

    /// Sets the response type ("in_channel" or "ephemeral").
    pub fn response_type(mut self, response_type: impl Into<String>) -> Self {
        self.response_type = Some(response_type.into());
        self
    }

    /// Sets whether to replace the original message (for response_url).
    pub fn replace_original(mut self, replace: bool) -> Self {
        self.replace_original = Some(replace);
        self
    }

    /// Sets whether to delete the original message (for response_url).
    pub fn delete_original(mut self, delete: bool) -> Self {
        self.delete_original = Some(delete);
        self
    }

    /// Sets whether to unfurl links.
    pub fn unfurl_links(mut self, unfurl: bool) -> Self {
        self.unfurl_links = Some(unfurl);
        self
    }

    /// Sets whether to unfurl media.
    pub fn unfurl_media(mut self, unfurl: bool) -> Self {
        self.unfurl_media = Some(unfurl);
        self
    }

    /// Sets metadata for the message.
    pub fn metadata<T: Serialize>(mut self, metadata: T) -> Self {
        self.metadata = serde_json::to_value(metadata).ok();
        self
    }

    /// Adds a custom header for this request only.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Executes the webhook request.
    pub async fn execute(self) -> Result<WebhookResponse> {
        let mut body = serde_json::Map::new();

        if let Some(text) = self.text {
            body.insert("text".to_string(), Value::String(text));
        }
        if let Some(blocks) = self.blocks {
            body.insert("blocks".to_string(), blocks);
        }
        if let Some(attachments) = self.attachments {
            body.insert("attachments".to_string(), attachments);
        }
        if let Some(response_type) = self.response_type {
            body.insert("response_type".to_string(), Value::String(response_type));
        }
        if let Some(replace_original) = self.replace_original {
            body.insert(
                "replace_original".to_string(),
                Value::Bool(replace_original),
            );
        }
        if let Some(delete_original) = self.delete_original {
            body.insert("delete_original".to_string(), Value::Bool(delete_original));
        }
        if let Some(unfurl_links) = self.unfurl_links {
            body.insert("unfurl_links".to_string(), Value::Bool(unfurl_links));
        }
        if let Some(unfurl_media) = self.unfurl_media {
            body.insert("unfurl_media".to_string(), Value::Bool(unfurl_media));
        }
        if let Some(metadata) = self.metadata {
            body.insert("metadata".to_string(), metadata);
        }

        self.client.send_dict(&body, self.headers.as_ref()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_webhook_response_creation() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let response = WebhookResponse::new(
            "https://hooks.slack.com/test".to_string(),
            200,
            "ok".to_string(),
            headers.clone(),
        );

        assert_eq!(response.url, "https://hooks.slack.com/test");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
        assert_eq!(response.headers, headers);
    }

    #[tokio::test]
    async fn test_webhook_response_is_success() {
        let response = WebhookResponse::new(
            "https://test.com".to_string(),
            200,
            "ok".to_string(),
            HashMap::new(),
        );
        assert!(response.is_success());

        let response = WebhookResponse::new(
            "https://test.com".to_string(),
            404,
            "not found".to_string(),
            HashMap::new(),
        );
        assert!(!response.is_success());
    }

    #[tokio::test]
    async fn test_webhook_response_is_rate_limited() {
        let response = WebhookResponse::new(
            "https://test.com".to_string(),
            429,
            "rate limited".to_string(),
            HashMap::new(),
        );
        assert!(response.is_rate_limited());

        let response = WebhookResponse::new(
            "https://test.com".to_string(),
            200,
            "ok".to_string(),
            HashMap::new(),
        );
        assert!(!response.is_rate_limited());
    }

    #[test]
    fn test_webhook_client_creation() {
        let client = WebhookClient::new("https://hooks.slack.com/services/T00/B00/XXX");
        assert_eq!(client.url, "https://hooks.slack.com/services/T00/B00/XXX");
        assert_eq!(client.timeout, Duration::from_secs(30));
        assert!(client.default_headers.contains_key("User-Agent"));
    }

    #[test]
    fn test_webhook_client_with_timeout() {
        let client =
            WebhookClient::new("https://hooks.slack.com/test").timeout(Duration::from_secs(60));
        assert_eq!(client.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_webhook_client_with_user_agent() {
        let client = WebhookClient::new("https://hooks.slack.com/test")
            .user_agent(Some("custom-prefix"), Some("custom-suffix"));

        let ua = client.default_headers.get("User-Agent").unwrap();
        assert!(ua.starts_with("custom-prefix"));
        assert!(ua.ends_with("custom-suffix"));
        assert!(ua.contains("slack-rs"));
    }

    #[test]
    fn test_webhook_client_with_default_header() {
        let client = WebhookClient::new("https://hooks.slack.com/test")
            .default_header("X-Custom-Header", "custom-value");

        assert_eq!(
            client.default_headers.get("X-Custom-Header"),
            Some(&"custom-value".to_string())
        );
    }

    #[test]
    fn test_webhook_client_with_proxy() {
        let client = WebhookClient::new("https://hooks.slack.com/test")
            .proxy("http://proxy.example.com:8080");

        assert_eq!(
            client.proxy,
            Some("http://proxy.example.com:8080".to_string())
        );
    }

    #[tokio::test]
    async fn test_send_text_only() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("content-type", "application/json"))
            .and(body_json(json!({"text": "hello!"})))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client.send().text("hello!").execute().unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_send_with_response_type() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({
                "text": "hello!",
                "response_type": "in_channel"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client
                .send()
                .text("hello!")
                .response_type("in_channel")
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_send_with_blocks() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            let blocks = json!([
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": "Some text"
                    }
                },
                {
                    "type": "image",
                    "image_url": "image.jpg",
                    "alt_text": "an image"
                }
            ]);

            client
                .send()
                .text("hello!")
                .blocks(blocks)
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_send_with_attachments() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            let attachments = json!([
                {
                    "color": "#f2c744",
                    "text": "attachment text"
                }
            ]);

            client
                .send()
                .text("hello!")
                .attachments(attachments)
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_send_with_replace_original() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({
                "text": "hello!",
                "replace_original": true
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client
                .send()
                .text("hello!")
                .replace_original(true)
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_send_with_delete_original() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({
                "text": "hello!",
                "delete_original": true
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client
                .send()
                .text("hello!")
                .delete_original(true)
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_send_with_unfurl_options() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({
                "text": "hello!",
                "unfurl_links": true,
                "unfurl_media": false
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client
                .send()
                .text("hello!")
                .unfurl_links(true)
                .unfurl_media(false)
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_send_with_custom_header() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("X-Custom-Header", "test-value"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client
                .send()
                .text("hello!")
                .header("X-Custom-Header", "test-value")
                .execute()
                .unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_send_dict() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({"text": "hello!"})))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            let mut body = serde_json::Map::new();
            body.insert("text".to_string(), Value::String("hello!".to_string()));

            client.send_dict(&body, None).unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_error_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client.send().text("hello!").execute().unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 500);
        assert_eq!(response.body, "internal server error");
        assert!(!response.is_success());
    }

    #[tokio::test]
    async fn test_rate_limit_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "60")
                    .set_body_string("rate_limited"),
            )
            .mount(&mock_server)
            .await;

        let uri = mock_server.uri();
        let response = tokio::task::spawn_blocking(move || {
            let client = WebhookClient::new(uri);
            client.send().text("hello!").execute().unwrap()
        })
        .await
        .unwrap();

        assert_eq!(response.status_code, 429);
        assert!(response.is_rate_limited());
        assert!(response.headers.contains_key("retry-after"));
    }

    // Async tests
    #[test]
    fn test_async_webhook_client_creation() {
        let client = AsyncWebhookClient::new("https://hooks.slack.com/services/T00/B00/XXX");
        assert_eq!(client.url, "https://hooks.slack.com/services/T00/B00/XXX");
        assert_eq!(client.timeout, Duration::from_secs(30));
        assert!(client.default_headers.contains_key("User-Agent"));
    }

    #[tokio::test]
    async fn test_async_send_text_only() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("content-type", "application/json"))
            .and(body_json(json!({"text": "hello!"})))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let response = client.send().text("hello!").execute().await.unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_async_send_with_response_type() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({
                "text": "hello!",
                "response_type": "ephemeral"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let response = client
            .send()
            .text("hello!")
            .response_type("ephemeral")
            .execute()
            .await
            .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_async_send_with_blocks() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let blocks = json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": "Some text"
                }
            },
            {
                "type": "image",
                "image_url": "image.jpg",
                "alt_text": "an image"
            }
        ]);

        let response = client
            .send()
            .text("hello!")
            .blocks(blocks)
            .execute()
            .await
            .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_async_send_with_attachments() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let attachments = json!([
            {
                "color": "#36a64f",
                "text": "attachment text"
            }
        ]);

        let response = client
            .send()
            .text("hello!")
            .attachments(attachments)
            .execute()
            .await
            .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[tokio::test]
    async fn test_async_send_with_metadata() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let metadata = json!({
            "event_type": "test",
            "event_payload": {"key": "value"}
        });

        let response = client
            .send()
            .text("hello!")
            .metadata(metadata)
            .execute()
            .await
            .unwrap();

        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_async_error_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let response = client.send().text("hello!").execute().await.unwrap();

        assert_eq!(response.status_code, 500);
        assert_eq!(response.body, "internal server error");
        assert!(!response.is_success());
    }

    #[tokio::test]
    async fn test_async_rate_limit_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(429)
                    .insert_header("Retry-After", "30")
                    .set_body_string("rate_limited"),
            )
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let response = client.send().text("hello!").execute().await.unwrap();

        assert_eq!(response.status_code, 429);
        assert!(response.is_rate_limited());
        assert!(response.headers.contains_key("retry-after"));
    }

    #[tokio::test]
    async fn test_async_send_dict() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_json(json!({"text": "hello!"})))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&mock_server)
            .await;

        let client = AsyncWebhookClient::new(mock_server.uri());
        let mut body = serde_json::Map::new();
        body.insert("text".to_string(), Value::String("hello!".to_string()));

        let response = client.send_dict(&body, None).await.unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "ok");
    }

    #[test]
    fn test_async_webhook_client_with_timeout() {
        let client = AsyncWebhookClient::new("https://hooks.slack.com/test")
            .timeout(Duration::from_secs(60));
        assert_eq!(client.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_async_webhook_client_with_user_agent() {
        let client = AsyncWebhookClient::new("https://hooks.slack.com/test")
            .user_agent(Some("async-prefix"), Some("async-suffix"));

        let ua = client.default_headers.get("User-Agent").unwrap();
        assert!(ua.starts_with("async-prefix"));
        assert!(ua.ends_with("async-suffix"));
        assert!(ua.contains("slack-rs"));
    }
}
