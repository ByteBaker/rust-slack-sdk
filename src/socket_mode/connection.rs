//! WebSocket connection management for Socket Mode.
//!
//! This module handles the WebSocket connection lifecycle, including
//! connecting, reconnecting, and managing the connection state.

use crate::error::{Result, SlackError};
use crate::socket_mode::types::{SocketModeRequest, SocketModeResponse};
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

/// WebSocket connection wrapper for Socket Mode.
#[derive(Clone, Debug)]
pub struct SocketModeConnection {
    /// The WebSocket stream
    stream: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,

    /// The WebSocket URL to connect to
    url: String,

    /// Whether the connection is active
    connected: Arc<Mutex<bool>>,
}

impl SocketModeConnection {
    /// Creates a new connection with the given WebSocket URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
            url: url.into(),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Connects to the WebSocket server.
    pub async fn connect(&self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| SlackError::SocketMode(format!("Failed to connect: {}", e)))?;

        let mut stream = self.stream.lock().await;
        *stream = Some(ws_stream);

        let mut connected = self.connected.lock().await;
        *connected = true;

        Ok(())
    }

    /// Disconnects from the WebSocket server.
    pub async fn disconnect(&self) -> Result<()> {
        let mut stream = self.stream.lock().await;
        if let Some(ws) = stream.take() {
            drop(ws); // Drops the connection
        }

        let mut connected = self.connected.lock().await;
        *connected = false;

        Ok(())
    }

    /// Checks if the connection is active.
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Receives the next message from the WebSocket.
    pub async fn receive_message(&self) -> Result<Option<SocketModeRequest>> {
        let mut stream = self.stream.lock().await;

        if let Some(ws) = stream.as_mut() {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let value: Value = serde_json::from_str(&text).map_err(|e| {
                        SlackError::SocketMode(format!("Failed to parse message: {}", e))
                    })?;

                    let request: SocketModeRequest =
                        serde_json::from_value(value).map_err(|e| {
                            SlackError::SocketMode(format!("Failed to deserialize request: {}", e))
                        })?;

                    Ok(Some(request))
                }
                Some(Ok(Message::Close(_))) => {
                    let mut connected = self.connected.lock().await;
                    *connected = false;
                    Ok(None)
                }
                Some(Ok(Message::Ping(data))) => {
                    // Respond to ping with pong
                    ws.send(Message::Pong(data)).await.map_err(|e| {
                        SlackError::SocketMode(format!("Failed to send pong: {}", e))
                    })?;
                    // Recursively wait for next message
                    drop(stream);
                    Box::pin(self.receive_message()).await
                }
                Some(Ok(_)) => {
                    // Skip other message types
                    drop(stream);
                    Box::pin(self.receive_message()).await
                }
                Some(Err(e)) => Err(SlackError::SocketMode(format!("WebSocket error: {}", e))),
                None => {
                    // Connection closed
                    let mut connected = self.connected.lock().await;
                    *connected = false;
                    Ok(None)
                }
            }
        } else {
            Err(SlackError::NotConnected)
        }
    }

    /// Sends an acknowledgment message.
    pub async fn send_acknowledgment(&self, response: &SocketModeResponse) -> Result<()> {
        let json = serde_json::to_string(response)
            .map_err(|e| SlackError::SocketMode(format!("Failed to serialize response: {}", e)))?;

        let mut stream = self.stream.lock().await;

        if let Some(ws) = stream.as_mut() {
            ws.send(Message::Text(json))
                .await
                .map_err(|e| SlackError::SocketMode(format!("Failed to send message: {}", e)))?;
            Ok(())
        } else {
            Err(SlackError::NotConnected)
        }
    }

    /// Returns a clone of the connection URL.
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_new() {
        let conn = SocketModeConnection::new("wss://wss-primary.slack.com/link");
        assert_eq!(conn.url(), "wss://wss-primary.slack.com/link");
    }

    #[tokio::test]
    async fn test_connection_not_connected_initially() {
        let conn = SocketModeConnection::new("wss://test.example.com");
        assert!(!conn.is_connected().await);
    }

    #[tokio::test]
    async fn test_receive_message_without_connection() {
        let conn = SocketModeConnection::new("wss://test.example.com");
        let result = conn.receive_message().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::NotConnected));
    }

    #[tokio::test]
    async fn test_send_acknowledgment_without_connection() {
        let conn = SocketModeConnection::new("wss://test.example.com");
        let response = SocketModeResponse::new("test-envelope");
        let result = conn.send_acknowledgment(&response).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::NotConnected));
    }
}
