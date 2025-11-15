//! HTTP retry logic for the Slack SDK.
//!
//! This module provides a comprehensive retry system for handling transient failures
//! when making HTTP requests to the Slack API. It includes:
//!
//! - **Retry handlers** for different error scenarios (rate limits, server errors, connection failures)
//! - **Interval calculators** for determining retry delays (exponential backoff, random jitter)
//! - **State management** for tracking retry attempts
//!
//! # Example
//!
//! ```rust,no_run
//! use slack_rs::http_retry::{
//!     handler::{RateLimitErrorHandler, RetryHandler, HttpResponse},
//!     state::RetryState,
//! };
//!
//! let mut handler = RateLimitErrorHandler::new(3);
//! let mut state = RetryState::new();
//! let response = HttpResponse {
//!     status_code: 429,
//!     headers: vec![],
//!     body: vec![],
//! };
//!
//! // Check if we should retry
//! if handler.can_retry(&state, Some(&response), None) {
//!     handler.prepare_for_next_attempt(&mut state);
//!     // Wait for state.duration_until_next_attempt()
//!     // Then retry the request
//! }
//! ```

pub mod handler;
pub mod interval;
pub mod state;

// Re-export commonly used types
pub use handler::{
    ConnectionErrorHandler, HttpResponse, RateLimitErrorHandler, RetryHandler, ServerErrorHandler,
};
pub use interval::{BackoffIntervalCalculator, IntervalCalculator, RandomJitterCalculator};
pub use state::RetryState;

/// Creates a default set of retry handlers.
///
/// This includes handlers for:
/// - Rate limit errors (HTTP 429)
/// - Server errors (HTTP 5xx)
/// - Connection errors
pub fn default_retry_handlers() -> Vec<Box<dyn RetryHandler>> {
    vec![
        Box::new(RateLimitErrorHandler::default()),
        Box::new(ServerErrorHandler::default()),
        Box::new(ConnectionErrorHandler::default()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_handlers_count() {
        let handlers = default_retry_handlers();
        assert_eq!(handlers.len(), 3);
    }

    #[test]
    fn test_default_retry_handlers_max_attempts() {
        let handlers = default_retry_handlers();

        for handler in handlers {
            // All default handlers should have 3 max attempts
            assert_eq!(handler.max_attempts(), 3);
        }
    }
}
