//! # Slack SDK for Rust
//!
//! A comprehensive Rust port of the Python Slack SDK, providing full access to the
//! Slack API with type-safe, idiomatic Rust interfaces.
//!
//! ## Features
//!
//! - **Web API Client**: Full support for 260+ Slack API methods
//! - **Block Kit Models**: Type-safe builders for Slack's UI components
//! - **WebSocket Support**: Real-time event handling via Socket Mode
//! - **OAuth**: Complete OAuth 2.0 flow implementation
//! - **HTTP Retry**: Automatic retry with exponential backoff
//! - **Signature Verification**: Request validation for webhooks
//!
//! ## Quick Start (Phase 1 - HTTP Retry)
//!
//! ```rust
//! use slack_rs::http_retry::{
//!     RateLimitErrorHandler, RetryHandler, RetryState, HttpResponse
//! };
//!
//! // Create a retry handler
//! let mut handler = RateLimitErrorHandler::new(3);
//! let mut state = RetryState::new();
//!
//! // Simulate a rate limit response
//! let response = HttpResponse {
//!     status_code: 429,
//!     headers: vec![("Retry-After".to_string(), "60".to_string())],
//!     body: vec![],
//! };
//!
//! // Check if we should retry
//! if handler.can_retry(&state, Some(&response), None) {
//!     handler.prepare_for_next_attempt(&mut state);
//!     let wait_time = state.duration_until_next_attempt();
//!     println!("Will retry after {:?}", wait_time);
//! }
//! ```
//!
//! ## Full API Coming Soon
//!
//! The Web API client will be available in Phase 5. Stay tuned!
//!
//! ## Modules
//!
//! - [`error`]: Error types for the SDK
//! - [`http_retry`]: HTTP retry logic with handlers and state management
//! - [`logging`]: Logging infrastructure using `tracing`
//! - [`webhook`]: Webhook clients for incoming webhooks and response URLs

pub mod constants;
pub mod error;
pub mod http_retry;
pub mod logging;
pub mod models;
pub mod signature;
pub mod webhook;

// Re-export commonly used types
