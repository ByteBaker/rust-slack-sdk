//! Socket Mode client for real-time Slack events.
//!
//! This module provides the main client for Socket Mode, which enables
//! real-time event handling without exposing a public HTTP endpoint.

use crate::error::{Result, SlackError};
use crate::socket_mode::connection::SocketModeConnection;
use crate::socket_mode::types::{SocketModeMessageType, SocketModeRequest, SocketModeResponse};
use crate::web::AsyncWebClient;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Type alias for event handler functions.
pub type EventHandler = Arc<dyn Fn(SocketModeRequest) -> Result<()> + Send + Sync>;

/// Socket Mode client for real-time event handling.
///
/// The Socket Mode client connects to Slack via WebSocket and receives events
/// in real-time. It automatically handles reconnection and acknowledgments.
///
/// # Examples
///
/// ```no_run
/// use slack_rs::socket_mode::SocketModeClient;
/// use slack_rs::error::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = SocketModeClient::new("xapp-1-A123-456-abc");
///
///     // Register event handlers
///     client.on_events_api(|req| {
///         println!("Received event: {:?}", req);
///         Ok(())
///     }).await;
///
///     // Connect and start processing events
///     client.connect().await?;
///
///     Ok(())
/// }
/// ```
pub struct SocketModeClient {
    /// App-level token for authentication (xapp-*)
    app_token: String,

    /// Web API client for making API calls
    web_client: AsyncWebClient,

    /// WebSocket connection
    connection: Arc<SocketModeConnection>,

    /// Event handlers by message type
    handlers: Arc<RwLock<HashMap<String, Vec<EventHandler>>>>,

    /// Whether to auto-acknowledge messages
    auto_acknowledge: Arc<RwLock<bool>>,

    /// Maximum reconnection attempts
    max_reconnect_attempts: usize,

    /// Whether the client is running
    running: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for SocketModeClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketModeClient")
            .field("app_token", &"<redacted>")
            .field("max_reconnect_attempts", &self.max_reconnect_attempts)
            .finish()
    }
}

impl SocketModeClient {
    /// Creates a new Socket Mode client with the given app-level token.
    ///
    /// The token must be an app-level token (starting with `xapp-`).
    pub fn new(app_token: impl Into<String>) -> Self {
        let app_token = app_token.into();
        let web_client = AsyncWebClient::new(&app_token);

        Self {
            app_token,
            web_client,
            connection: Arc::new(SocketModeConnection::new("")),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            auto_acknowledge: Arc::new(RwLock::new(true)),
            max_reconnect_attempts: 5,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Sets whether to automatically acknowledge messages.
    pub async fn set_auto_acknowledge(&self, auto: bool) {
        let mut auto_ack = self.auto_acknowledge.write().await;
        *auto_ack = auto;
    }

    /// Sets the maximum number of reconnection attempts.
    pub fn with_max_reconnect_attempts(mut self, max: usize) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    /// Connects to Slack via Socket Mode.
    ///
    /// This method obtains a WebSocket URL from the Slack API and establishes
    /// the WebSocket connection.
    pub async fn connect(&self) -> Result<()> {
        // Call apps.connections.open to get the WebSocket URL
        let response = self
            .web_client
            .apps_connections_open(Some(json!({})))
            .await?;

        // Extract the WebSocket URL from the response
        let url = response["url"]
            .as_str()
            .ok_or_else(|| SlackError::SocketMode("No WebSocket URL in response".to_string()))?;

        // Create a new connection with the URL
        let connection = Arc::new(SocketModeConnection::new(url));
        connection.connect().await?;

        // Replace the connection
        let old_connection = std::mem::replace(
            &mut *Arc::get_mut(&mut self.connection.clone())
                .ok_or_else(|| SlackError::SocketMode("Failed to update connection".to_string()))?,
            (*connection).clone(),
        );
        drop(old_connection);

        Ok(())
    }

    /// Disconnects from the WebSocket server.
    pub async fn disconnect(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;

        self.connection.disconnect().await
    }

    /// Checks if the client is connected.
    pub async fn is_connected(&self) -> bool {
        self.connection.is_connected().await
    }

    /// Registers an event handler for Events API messages.
    pub async fn on_events_api<F>(&self, handler: F)
    where
        F: Fn(SocketModeRequest) -> Result<()> + Send + Sync + 'static,
    {
        self.register_handler("events_api", Arc::new(handler)).await;
    }

    /// Registers an event handler for slash commands.
    pub async fn on_slash_commands<F>(&self, handler: F)
    where
        F: Fn(SocketModeRequest) -> Result<()> + Send + Sync + 'static,
    {
        self.register_handler("slash_commands", Arc::new(handler))
            .await;
    }

    /// Registers an event handler for interactive components.
    pub async fn on_interactive<F>(&self, handler: F)
    where
        F: Fn(SocketModeRequest) -> Result<()> + Send + Sync + 'static,
    {
        self.register_handler("interactive", Arc::new(handler))
            .await;
    }

    /// Registers a handler for a specific message type.
    async fn register_handler(&self, message_type: &str, handler: EventHandler) {
        let mut handlers = self.handlers.write().await;
        handlers
            .entry(message_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Starts processing events from the WebSocket connection.
    ///
    /// This method runs in a loop, receiving messages and calling the appropriate
    /// handlers. It will automatically reconnect if the connection is lost.
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        let mut reconnect_attempts = 0;

        loop {
            // Check if we should stop
            if !*self.running.read().await {
                break;
            }

            // Ensure we're connected
            if !self.is_connected().await {
                if reconnect_attempts >= self.max_reconnect_attempts {
                    return Err(SlackError::SocketMode(
                        "Max reconnection attempts reached".to_string(),
                    ));
                }

                // Try to reconnect
                match self.connect().await {
                    Ok(_) => {
                        reconnect_attempts = 0;
                    }
                    Err(_e) => {
                        reconnect_attempts += 1;
                        tokio::time::sleep(tokio::time::Duration::from_secs(
                            2_u64.pow(reconnect_attempts.min(5) as u32),
                        ))
                        .await;
                        continue;
                    }
                }
            }

            // Receive the next message
            match self.connection.receive_message().await {
                Ok(Some(request)) => {
                    // Handle special message types
                    let msg_type = SocketModeMessageType::from_string(&request.message_type);

                    match msg_type {
                        SocketModeMessageType::Disconnect => {
                            // Server requested disconnect, try to reconnect
                            let _ = self.connection.disconnect().await;
                            continue;
                        }
                        SocketModeMessageType::Hello => {
                            // Connection established successfully
                            continue;
                        }
                        _ => {
                            // Process the request
                            self.process_request(request).await?;
                        }
                    }
                }
                Ok(None) => {
                    // Connection closed, will reconnect on next iteration
                    continue;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Processes a single Socket Mode request.
    async fn process_request(&self, request: SocketModeRequest) -> Result<()> {
        let envelope_id = request.envelope_id.clone();

        // Call the handlers
        let handlers = self.handlers.read().await;
        if let Some(handler_list) = handlers.get(&request.message_type) {
            for handler in handler_list {
                if let Err(e) = handler(request.clone()) {
                    eprintln!("Handler error: {}", e);
                }
            }
        }

        // Auto-acknowledge if enabled
        if *self.auto_acknowledge.read().await {
            let response = SocketModeResponse::new(envelope_id);
            self.connection.send_acknowledgment(&response).await?;
        }

        Ok(())
    }

    /// Manually acknowledges a Socket Mode request.
    ///
    /// This is useful when auto-acknowledge is disabled.
    pub async fn acknowledge(&self, envelope_id: impl Into<String>) -> Result<()> {
        let response = SocketModeResponse::new(envelope_id);
        self.connection.send_acknowledgment(&response).await
    }

    /// Acknowledges a Socket Mode request with a payload.
    pub async fn acknowledge_with_payload(
        &self,
        envelope_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Result<()> {
        let response = SocketModeResponse::with_payload(envelope_id, payload);
        self.connection.send_acknowledgment(&response).await
    }
}

// Clone implementation to allow sharing the client
impl Clone for SocketModeClient {
    fn clone(&self) -> Self {
        Self {
            app_token: self.app_token.clone(),
            web_client: self.web_client.clone(),
            connection: Arc::clone(&self.connection),
            handlers: Arc::clone(&self.handlers),
            auto_acknowledge: Arc::clone(&self.auto_acknowledge),
            max_reconnect_attempts: self.max_reconnect_attempts,
            running: Arc::clone(&self.running),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_new() {
        let client = SocketModeClient::new("xapp-1-A123-456-abc");
        assert_eq!(client.app_token, "xapp-1-A123-456-abc");
    }

    #[tokio::test]
    async fn test_client_auto_acknowledge() {
        let client = SocketModeClient::new("xapp-test");
        assert!(*client.auto_acknowledge.read().await);

        client.set_auto_acknowledge(false).await;
        assert!(!*client.auto_acknowledge.read().await);
    }

    #[test]
    fn test_client_with_max_reconnect_attempts() {
        let client = SocketModeClient::new("xapp-test").with_max_reconnect_attempts(10);
        assert_eq!(client.max_reconnect_attempts, 10);
    }

    #[tokio::test]
    async fn test_client_register_handlers() {
        let client = SocketModeClient::new("xapp-test");

        client
            .on_events_api(|_req| {
                // Handler logic
                Ok(())
            })
            .await;

        client
            .on_slash_commands(|_req| {
                // Handler logic
                Ok(())
            })
            .await;

        let handlers = client.handlers.read().await;
        assert!(handlers.contains_key("events_api"));
        assert!(handlers.contains_key("slash_commands"));
    }

    #[tokio::test]
    async fn test_client_not_connected_initially() {
        let client = SocketModeClient::new("xapp-test");
        assert!(!client.is_connected().await);
    }

    #[test]
    fn test_client_clone() {
        let client1 = SocketModeClient::new("xapp-test");
        let client2 = client1.clone();

        assert_eq!(client1.app_token, client2.app_token);
        assert_eq!(
            client1.max_reconnect_attempts,
            client2.max_reconnect_attempts
        );
    }
}
