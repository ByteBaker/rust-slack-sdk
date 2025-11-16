//! Error types for the Slack SDK.
//!
//! This module provides a comprehensive error type hierarchy that covers all possible
//! failure modes when interacting with the Slack API.

use std::fmt;
use thiserror::Error;

/// The main error type for the Slack SDK.
///
/// This enum covers all possible errors that can occur when using the SDK,
/// from network failures to API-specific errors.
#[derive(Error, Debug)]
pub enum SlackError {
    /// An error returned by the Slack API.
    #[error("Slack API error: {0}")]
    Api(#[from] SlackApiError),

    /// An HTTP error occurred while making a request.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// The request signature verification failed.
    #[error("Invalid signature")]
    InvalidSignature,

    /// An invalid or malformed token was provided.
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// A validation error occurred (e.g., field too long, missing required field).
    #[error("Validation error: {0}")]
    Validation(String),

    /// A wrong token type was used for the operation.
    #[error("Wrong token type: expected {expected}, got {actual}")]
    WrongTokenType { expected: String, actual: String },

    /// An OAuth token rotation error.
    #[error("Token rotation failed: {0}")]
    TokenRotation(String),

    /// A WebSocket/Socket Mode error.
    #[error("Socket mode error: {0}")]
    SocketMode(String),

    /// The client is not connected (for Socket Mode).
    #[error("Client not connected")]
    NotConnected,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// An HTTP error occurred (not from reqwest).
    #[error("HTTP error: {message}")]
    HttpError { message: String },

    /// An API error with response data.
    #[error("API error: {message}")]
    ApiError {
        message: String,
        response: serde_json::Value,
    },

    /// Invalid input provided to the SDK.
    #[error("Invalid input: {message}")]
    InvalidInput { message: String },

    /// An error occurred during pagination.
    #[error("Pagination error: {0}")]
    PaginationError(String),

    /// A storage error occurred (OAuth installation storage, state storage, etc.).
    #[error("Storage error: {0}")]
    StorageError(String),

    /// A serialization error occurred.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// A deserialization error occurred.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// A resource was not found.
    #[error("Not found: {0}")]
    NotFound(String),
}

/// An error returned by the Slack API.
///
/// When the Slack API returns an error response, it includes an error code
/// and optionally additional details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlackApiError {
    /// The error code returned by Slack (e.g., "channel_not_found").
    pub error: String,

    /// The HTTP status code.
    pub status_code: u16,

    /// Optional additional error details.
    pub details: Option<String>,

    /// The response body for debugging.
    pub response_body: Option<String>,
}

impl fmt::Display for SlackApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slack API error ({}): {}", self.status_code, self.error)?;
        if let Some(details) = &self.details {
            write!(f, " - {}", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for SlackApiError {}

impl SlackApiError {
    /// Creates a new Slack API error.
    pub fn new(error: impl Into<String>, status_code: u16) -> Self {
        Self {
            error: error.into(),
            status_code,
            details: None,
            response_body: None,
        }
    }

    /// Sets the error details.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Sets the response body for debugging.
    pub fn with_response_body(mut self, body: impl Into<String>) -> Self {
        self.response_body = Some(body.into());
        self
    }
}

/// A specialized Result type for Slack SDK operations.
pub type Result<T> = std::result::Result<T, SlackError>;

/// Convenience type alias for Error
pub type Error = SlackError;

impl SlackError {
    /// Creates a storage error
    pub fn storage_error(message: impl Into<String>) -> Self {
        Self::StorageError(message.into())
    }

    /// Creates a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Creates a deserialization error
    pub fn deserialization_error(message: impl Into<String>) -> Self {
        Self::DeserializationError(message.into())
    }

    /// Creates a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Creates an API error
    pub fn api_error(message: impl Into<String>) -> Self {
        Self::ApiError {
            message: message.into(),
            response: serde_json::Value::Null,
        }
    }

    /// Creates an invalid input error
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_slack_api_error_display() {
        let error = SlackApiError::new("channel_not_found", 404);
        assert_eq!(
            error.to_string(),
            "Slack API error (404): channel_not_found"
        );
    }

    #[test]
    fn test_slack_api_error_display_with_details() {
        let error = SlackApiError::new("invalid_auth", 401)
            .with_details("The token is invalid or has been revoked");

        assert_eq!(
            error.to_string(),
            "Slack API error (401): invalid_auth - The token is invalid or has been revoked"
        );
    }

    #[test]
    fn test_slack_api_error_builder() {
        let error = SlackApiError::new("rate_limited", 429)
            .with_details("Too many requests")
            .with_response_body(r#"{"ok":false,"error":"rate_limited"}"#);

        assert_eq!(error.error, "rate_limited");
        assert_eq!(error.status_code, 429);
        assert_eq!(error.details, Some("Too many requests".to_string()));
        assert!(error.response_body.is_some());
    }

    #[test]
    fn test_slack_error_from_api_error() {
        let api_error = SlackApiError::new("not_found", 404);
        let slack_error: SlackError = api_error.into();

        assert!(matches!(slack_error, SlackError::Api(_)));
    }

    #[test]
    fn test_slack_error_invalid_token_display() {
        let error = SlackError::InvalidToken("Token is empty".to_string());
        assert_eq!(error.to_string(), "Invalid token: Token is empty");
    }

    #[test]
    fn test_slack_error_validation_display() {
        let error = SlackError::Validation("Text exceeds maximum length".to_string());
        assert_eq!(
            error.to_string(),
            "Validation error: Text exceeds maximum length"
        );
    }

    #[test]
    fn test_slack_error_wrong_token_type() {
        let error = SlackError::WrongTokenType {
            expected: "bot token".to_string(),
            actual: "user token".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Wrong token type: expected bot token, got user token"
        );
    }

    #[test]
    fn test_slack_error_debug() {
        let error = SlackError::InvalidSignature;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidSignature"));
    }

    #[test]
    fn test_slack_api_error_equality() {
        let error1 = SlackApiError::new("test_error", 400);
        let error2 = SlackApiError::new("test_error", 400);
        let error3 = SlackApiError::new("other_error", 400);

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        let result = returns_result();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_result_type_alias_error() {
        fn returns_error() -> Result<i32> {
            Err(SlackError::NotConnected)
        }

        let result = returns_error();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::NotConnected));
    }
}
