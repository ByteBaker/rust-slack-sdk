//! Socket Mode message types.
//!
//! This module defines the request and response types used in Socket Mode communication.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A Socket Mode request envelope received from Slack.
///
/// Socket Mode wraps all events in an envelope that includes metadata
/// about the message type and requires acknowledgment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocketModeRequest {
    /// The type of the message (e.g., "events_api", "slash_commands", "interactive")
    #[serde(rename = "type")]
    pub message_type: String,

    /// The unique envelope ID for acknowledgment
    pub envelope_id: String,

    /// The actual payload data
    pub payload: Value,

    /// Whether this request accepts a response payload
    #[serde(default)]
    pub accepts_response_payload: bool,

    /// Retry attempt number (for events_api)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_attempt: Option<u32>,

    /// Retry reason (for events_api)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_reason: Option<String>,
}

impl SocketModeRequest {
    /// Creates a new Socket Mode request.
    pub fn new(
        message_type: impl Into<String>,
        envelope_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            message_type: message_type.into(),
            envelope_id: envelope_id.into(),
            payload,
            accepts_response_payload: false,
            retry_attempt: None,
            retry_reason: None,
        }
    }

    /// Sets whether this request accepts a response payload.
    pub fn with_accepts_response_payload(mut self, accepts: bool) -> Self {
        self.accepts_response_payload = accepts;
        self
    }

    /// Sets the retry attempt number.
    pub fn with_retry_attempt(mut self, attempt: u32) -> Self {
        self.retry_attempt = Some(attempt);
        self
    }

    /// Sets the retry reason.
    pub fn with_retry_reason(mut self, reason: impl Into<String>) -> Self {
        self.retry_reason = Some(reason.into());
        self
    }
}

/// A Socket Mode response (acknowledgment) to send back to Slack.
///
/// After receiving a Socket Mode request, the client must acknowledge it
/// by sending a response with the same envelope_id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocketModeResponse {
    /// The envelope ID from the request being acknowledged
    pub envelope_id: String,

    /// Optional response payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

impl SocketModeResponse {
    /// Creates a new Socket Mode response to acknowledge a request.
    pub fn new(envelope_id: impl Into<String>) -> Self {
        Self {
            envelope_id: envelope_id.into(),
            payload: None,
        }
    }

    /// Creates a response with a payload.
    pub fn with_payload(envelope_id: impl Into<String>, payload: Value) -> Self {
        Self {
            envelope_id: envelope_id.into(),
            payload: Some(payload),
        }
    }
}

/// Socket Mode message types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketModeMessageType {
    /// Events API event
    EventsApi,

    /// Slash command
    SlashCommands,

    /// Interactive component (buttons, menus, etc.)
    Interactive,

    /// App mention event
    AppMention,

    /// Disconnect message from Slack
    Disconnect,

    /// Hello message (connection confirmation)
    Hello,

    /// Unknown message type
    Unknown,
}

impl SocketModeMessageType {
    /// Parses a message type string into an enum variant.
    pub fn from_string(s: &str) -> Self {
        match s {
            "events_api" => Self::EventsApi,
            "slash_commands" => Self::SlashCommands,
            "interactive" => Self::Interactive,
            "app_mention" => Self::AppMention,
            "disconnect" => Self::Disconnect,
            "hello" => Self::Hello,
            _ => Self::Unknown,
        }
    }

    /// Returns the string representation of the message type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EventsApi => "events_api",
            Self::SlashCommands => "slash_commands",
            Self::Interactive => "interactive",
            Self::AppMention => "app_mention",
            Self::Disconnect => "disconnect",
            Self::Hello => "hello",
            Self::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_socket_mode_request_new() {
        let payload = json!({"event": {"type": "message", "text": "hello"}});
        let req = SocketModeRequest::new("events_api", "envelope-123", payload.clone());

        assert_eq!(req.message_type, "events_api");
        assert_eq!(req.envelope_id, "envelope-123");
        assert_eq!(req.payload, payload);
        assert!(!req.accepts_response_payload);
        assert!(req.retry_attempt.is_none());
        assert!(req.retry_reason.is_none());
    }

    #[test]
    fn test_socket_mode_request_builder() {
        let payload = json!({"text": "test"});
        let req = SocketModeRequest::new("slash_commands", "env-456", payload)
            .with_accepts_response_payload(true)
            .with_retry_attempt(2)
            .with_retry_reason("timeout");

        assert!(req.accepts_response_payload);
        assert_eq!(req.retry_attempt, Some(2));
        assert_eq!(req.retry_reason, Some("timeout".to_string()));
    }

    #[test]
    fn test_socket_mode_request_serialization() {
        let payload = json!({"foo": "bar"});
        let req = SocketModeRequest::new("events_api", "env-789", payload);

        let json_str = serde_json::to_string(&req).unwrap();
        let deserialized: SocketModeRequest = serde_json::from_str(&json_str).unwrap();

        assert_eq!(req, deserialized);
    }

    #[test]
    fn test_socket_mode_request_deserialization() {
        let json_data = r#"{
            "type": "events_api",
            "envelope_id": "x-12345",
            "payload": {
                "event": {
                    "type": "app_mention",
                    "text": "hello bot"
                }
            },
            "accepts_response_payload": true,
            "retry_attempt": 1,
            "retry_reason": "timeout"
        }"#;

        let req: SocketModeRequest = serde_json::from_str(json_data).unwrap();

        assert_eq!(req.message_type, "events_api");
        assert_eq!(req.envelope_id, "x-12345");
        assert!(req.accepts_response_payload);
        assert_eq!(req.retry_attempt, Some(1));
        assert_eq!(req.retry_reason, Some("timeout".to_string()));
    }

    #[test]
    fn test_socket_mode_response_new() {
        let resp = SocketModeResponse::new("envelope-123");

        assert_eq!(resp.envelope_id, "envelope-123");
        assert!(resp.payload.is_none());
    }

    #[test]
    fn test_socket_mode_response_with_payload() {
        let payload = json!({"status": "ok"});
        let resp = SocketModeResponse::with_payload("envelope-456", payload.clone());

        assert_eq!(resp.envelope_id, "envelope-456");
        assert_eq!(resp.payload, Some(payload));
    }

    #[test]
    fn test_socket_mode_response_serialization() {
        let resp = SocketModeResponse::new("env-xyz");
        let json_str = serde_json::to_string(&resp).unwrap();
        let deserialized: SocketModeResponse = serde_json::from_str(&json_str).unwrap();

        assert_eq!(resp, deserialized);
    }

    #[test]
    fn test_socket_mode_message_type_from_str() {
        assert_eq!(
            SocketModeMessageType::from_string("events_api"),
            SocketModeMessageType::EventsApi
        );
        assert_eq!(
            SocketModeMessageType::from_string("slash_commands"),
            SocketModeMessageType::SlashCommands
        );
        assert_eq!(
            SocketModeMessageType::from_string("interactive"),
            SocketModeMessageType::Interactive
        );
        assert_eq!(
            SocketModeMessageType::from_string("disconnect"),
            SocketModeMessageType::Disconnect
        );
        assert_eq!(
            SocketModeMessageType::from_string("hello"),
            SocketModeMessageType::Hello
        );
        assert_eq!(
            SocketModeMessageType::from_string("unknown_type"),
            SocketModeMessageType::Unknown
        );
    }

    #[test]
    fn test_socket_mode_message_type_as_str() {
        assert_eq!(SocketModeMessageType::EventsApi.as_str(), "events_api");
        assert_eq!(
            SocketModeMessageType::SlashCommands.as_str(),
            "slash_commands"
        );
        assert_eq!(SocketModeMessageType::Interactive.as_str(), "interactive");
        assert_eq!(SocketModeMessageType::Disconnect.as_str(), "disconnect");
        assert_eq!(SocketModeMessageType::Hello.as_str(), "hello");
        assert_eq!(SocketModeMessageType::Unknown.as_str(), "unknown");
    }
}
