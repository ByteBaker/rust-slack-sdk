//! Retry handlers for different error scenarios.
//!
//! This module provides handlers that determine whether and how to retry failed requests.

use super::interval::{BackoffIntervalCalculator, IntervalCalculator};
use super::state::RetryState;
use std::time::Duration;
use tracing::{info, warn};

/// HTTP response information needed for retry decisions.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub status_code: u16,

    /// Response headers.
    pub headers: Vec<(String, String)>,

    /// Response body (for error analysis).
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Gets a header value by name (case-insensitive).
    pub fn get_header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| key.to_lowercase() == name_lower)
            .map(|(_, value)| value.as_str())
    }
}

/// Trait for retry handlers.
///
/// Handlers decide whether a failed request should be retried and
/// prepare the state for the next attempt.
pub trait RetryHandler: Send + Sync {
    /// Determines whether the request should be retried.
    ///
    /// # Arguments
    /// * `state` - The current retry state
    /// * `response` - The HTTP response (if available)
    /// * `error` - The error that occurred (if any)
    ///
    /// # Returns
    /// `true` if the request should be retried, `false` otherwise.
    fn can_retry(
        &self,
        state: &RetryState,
        response: Option<&HttpResponse>,
        error: Option<&str>,
    ) -> bool;

    /// Prepares for the next retry attempt.
    ///
    /// This method is called when `can_retry` returns true.
    /// It should update the state to schedule the next attempt.
    fn prepare_for_next_attempt(&mut self, state: &mut RetryState);

    /// Returns the maximum number of retry attempts.
    fn max_attempts(&self) -> u32;
}

/// Handler for rate limit errors (HTTP 429).
#[derive(Debug, Clone)]
pub struct RateLimitErrorHandler {
    max_attempts: u32,
    interval_calculator: BackoffIntervalCalculator,
}

impl Default for RateLimitErrorHandler {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            interval_calculator: BackoffIntervalCalculator::default(),
        }
    }
}

impl RateLimitErrorHandler {
    /// Creates a new rate limit handler with the specified max attempts.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            interval_calculator: BackoffIntervalCalculator::default(),
        }
    }

    /// Sets the interval calculator.
    pub fn with_interval_calculator(mut self, calculator: BackoffIntervalCalculator) -> Self {
        self.interval_calculator = calculator;
        self
    }

    /// Gets the retry-after duration from the response headers.
    fn get_retry_after(&self, response: &HttpResponse) -> Option<Duration> {
        response
            .get_header("Retry-After")
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs)
    }
}

impl RetryHandler for RateLimitErrorHandler {
    fn can_retry(
        &self,
        state: &RetryState,
        response: Option<&HttpResponse>,
        _error: Option<&str>,
    ) -> bool {
        if state.current_attempt >= self.max_attempts {
            return false;
        }

        if let Some(resp) = response {
            if resp.status_code == 429 {
                let retry_after = self
                    .get_retry_after(resp)
                    .unwrap_or_else(|| Duration::from_secs(1));

                info!(
                    attempt = state.current_attempt,
                    retry_after_secs = retry_after.as_secs(),
                    "Rate limited, will retry after delay"
                );
                return true;
            }
        }

        false
    }

    fn prepare_for_next_attempt(&mut self, state: &mut RetryState) {
        state.increment_attempt();

        let interval = self.interval_calculator.calculate(state.current_attempt);

        info!(
            attempt = state.current_attempt,
            wait_secs = interval.as_secs(),
            "Going to retry the same request"
        );

        state.set_next_attempt(interval);
    }

    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

/// Handler for server errors (HTTP 5xx).
#[derive(Debug, Clone)]
pub struct ServerErrorHandler {
    max_attempts: u32,
    interval_calculator: BackoffIntervalCalculator,
}

impl Default for ServerErrorHandler {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            interval_calculator: BackoffIntervalCalculator::default(),
        }
    }
}

impl ServerErrorHandler {
    /// Creates a new server error handler with the specified max attempts.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            interval_calculator: BackoffIntervalCalculator::default(),
        }
    }

    /// Sets the interval calculator.
    pub fn with_interval_calculator(mut self, calculator: BackoffIntervalCalculator) -> Self {
        self.interval_calculator = calculator;
        self
    }
}

impl RetryHandler for ServerErrorHandler {
    fn can_retry(
        &self,
        state: &RetryState,
        response: Option<&HttpResponse>,
        _error: Option<&str>,
    ) -> bool {
        if state.current_attempt >= self.max_attempts {
            return false;
        }

        if let Some(resp) = response {
            if (500..600).contains(&resp.status_code) {
                warn!(
                    status_code = resp.status_code,
                    attempt = state.current_attempt,
                    "Server error detected, will retry"
                );
                return true;
            }
        }

        false
    }

    fn prepare_for_next_attempt(&mut self, state: &mut RetryState) {
        state.increment_attempt();
        let interval = self.interval_calculator.calculate(state.current_attempt);
        state.set_next_attempt(interval);

        info!(
            attempt = state.current_attempt,
            wait_secs = interval.as_secs(),
            "Retrying after server error"
        );
    }

    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

/// Handler for connection errors (network failures).
#[derive(Debug, Clone)]
pub struct ConnectionErrorHandler {
    max_attempts: u32,
    interval_calculator: BackoffIntervalCalculator,
}

impl Default for ConnectionErrorHandler {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            interval_calculator: BackoffIntervalCalculator::default(),
        }
    }
}

impl ConnectionErrorHandler {
    /// Creates a new connection error handler with the specified max attempts.
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            interval_calculator: BackoffIntervalCalculator::default(),
        }
    }

    /// Sets the interval calculator.
    pub fn with_interval_calculator(mut self, calculator: BackoffIntervalCalculator) -> Self {
        self.interval_calculator = calculator;
        self
    }

    /// Checks if the error is a connection error.
    fn is_connection_error(&self, error: &str) -> bool {
        let error_lower = error.to_lowercase();
        error_lower.contains("connection")
            || error_lower.contains("timeout")
            || error_lower.contains("network")
            || error_lower.contains("dns")
    }
}

impl RetryHandler for ConnectionErrorHandler {
    fn can_retry(
        &self,
        state: &RetryState,
        _response: Option<&HttpResponse>,
        error: Option<&str>,
    ) -> bool {
        if state.current_attempt >= self.max_attempts {
            return false;
        }

        if let Some(err) = error {
            if self.is_connection_error(err) {
                warn!(
                    error = err,
                    attempt = state.current_attempt,
                    "Connection error detected, will retry"
                );
                return true;
            }
        }

        false
    }

    fn prepare_for_next_attempt(&mut self, state: &mut RetryState) {
        state.increment_attempt();
        let interval = self.interval_calculator.calculate(state.current_attempt);
        state.set_next_attempt(interval);

        info!(
            attempt = state.current_attempt,
            wait_secs = interval.as_secs(),
            "Retrying after connection error"
        );
    }

    fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_http_response_get_header() {
        let response = HttpResponse {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "application/json".to_string()),
                ("Retry-After".to_string(), "60".to_string()),
            ],
            body: vec![],
        };

        assert_eq!(
            response.get_header("Content-Type"),
            Some("application/json")
        );
        assert_eq!(response.get_header("Retry-After"), Some("60"));
        assert_eq!(response.get_header("retry-after"), Some("60")); // Case-insensitive
        assert_eq!(response.get_header("X-Missing"), None);
    }

    // Rate Limit Handler Tests

    #[test]
    fn test_rate_limit_handler_default() {
        let handler = RateLimitErrorHandler::default();
        assert_eq!(handler.max_attempts, 3);
    }

    #[test]
    fn test_rate_limit_handler_new() {
        let handler = RateLimitErrorHandler::new(5);
        assert_eq!(handler.max_attempts, 5);
    }

    #[test]
    fn test_rate_limit_handler_detects_429() {
        let handler = RateLimitErrorHandler::default();
        let response = HttpResponse {
            status_code: 429,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState::default();

        assert!(handler.can_retry(&state, Some(&response), None));
    }

    #[test]
    fn test_rate_limit_handler_ignores_other_status() {
        let handler = RateLimitErrorHandler::default();
        let response = HttpResponse {
            status_code: 404,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState::default();

        assert!(!handler.can_retry(&state, Some(&response), None));
    }

    #[test]
    fn test_rate_limit_handler_respects_max_attempts() {
        let handler = RateLimitErrorHandler::new(3);
        let response = HttpResponse {
            status_code: 429,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState {
            current_attempt: 3,
            ..Default::default()
        };

        assert!(!handler.can_retry(&state, Some(&response), None));
    }

    #[test]
    fn test_rate_limit_handler_uses_retry_after_header() {
        let handler = RateLimitErrorHandler::default();
        let response = HttpResponse {
            status_code: 429,
            headers: vec![("Retry-After".to_string(), "120".to_string())],
            body: vec![],
        };

        let retry_after = handler.get_retry_after(&response);
        assert_eq!(retry_after, Some(Duration::from_secs(120)));
    }

    #[test]
    fn test_rate_limit_handler_prepare_for_next_attempt() {
        let mut handler = RateLimitErrorHandler::new(5);
        let mut state = RetryState::default();

        handler.prepare_for_next_attempt(&mut state);

        assert_eq!(state.current_attempt, 1);
        assert!(state.duration_until_next_attempt() > Duration::ZERO);
    }

    // Server Error Handler Tests

    #[test]
    fn test_server_error_handler_default() {
        let handler = ServerErrorHandler::default();
        assert_eq!(handler.max_attempts, 3);
    }

    #[test]
    fn test_server_error_handler_detects_500() {
        let handler = ServerErrorHandler::default();
        let response = HttpResponse {
            status_code: 500,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState::default();

        assert!(handler.can_retry(&state, Some(&response), None));
    }

    #[test]
    fn test_server_error_handler_detects_503() {
        let handler = ServerErrorHandler::default();
        let response = HttpResponse {
            status_code: 503,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState::default();

        assert!(handler.can_retry(&state, Some(&response), None));
    }

    #[test]
    fn test_server_error_handler_ignores_4xx() {
        let handler = ServerErrorHandler::default();
        let response = HttpResponse {
            status_code: 404,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState::default();

        assert!(!handler.can_retry(&state, Some(&response), None));
    }

    #[test]
    fn test_server_error_handler_respects_max_attempts() {
        let handler = ServerErrorHandler::new(2);
        let response = HttpResponse {
            status_code: 500,
            headers: vec![],
            body: vec![],
        };
        let state = RetryState {
            current_attempt: 2,
            ..Default::default()
        };

        assert!(!handler.can_retry(&state, Some(&response), None));
    }

    // Connection Error Handler Tests

    #[test]
    fn test_connection_error_handler_default() {
        let handler = ConnectionErrorHandler::default();
        assert_eq!(handler.max_attempts, 3);
    }

    #[test]
    fn test_connection_error_handler_detects_connection_errors() {
        let handler = ConnectionErrorHandler::default();
        let state = RetryState::default();

        assert!(handler.can_retry(&state, None, Some("Connection timeout")));
        assert!(handler.can_retry(&state, None, Some("connection refused")));
        assert!(handler.can_retry(&state, None, Some("Network error")));
        assert!(handler.can_retry(&state, None, Some("DNS lookup failed")));
    }

    #[test]
    fn test_connection_error_handler_ignores_other_errors() {
        let handler = ConnectionErrorHandler::default();
        let state = RetryState::default();

        assert!(!handler.can_retry(&state, None, Some("Invalid JSON")));
        assert!(!handler.can_retry(&state, None, Some("Parsing failed")));
    }

    #[test]
    fn test_connection_error_handler_respects_max_attempts() {
        let handler = ConnectionErrorHandler::new(2);
        let state = RetryState {
            current_attempt: 2,
            ..Default::default()
        };

        assert!(!handler.can_retry(&state, None, Some("Connection timeout")));
    }

    #[test]
    fn test_connection_error_handler_prepare_for_next_attempt() {
        let mut handler = ConnectionErrorHandler::new(5);
        let mut state = RetryState::default();

        handler.prepare_for_next_attempt(&mut state);

        assert_eq!(state.current_attempt, 1);
        assert!(state.duration_until_next_attempt() > Duration::ZERO);
    }

    #[test]
    fn test_handler_max_attempts_method() {
        let rate_limit_handler = RateLimitErrorHandler::new(5);
        let server_error_handler = ServerErrorHandler::new(3);
        let connection_handler = ConnectionErrorHandler::new(7);

        assert_eq!(rate_limit_handler.max_attempts(), 5);
        assert_eq!(server_error_handler.max_attempts(), 3);
        assert_eq!(connection_handler.max_attempts(), 7);
    }

    #[test]
    fn test_handler_with_custom_interval_calculator() {
        let calculator = BackoffIntervalCalculator::new(3.0).with_max_interval(60);

        let mut handler = RateLimitErrorHandler::new(5).with_interval_calculator(calculator);

        let mut state = RetryState::default();
        handler.can_retry(
            &state,
            Some(&HttpResponse {
                status_code: 429,
                headers: vec![],
                body: vec![],
            }),
            None,
        );

        handler.prepare_for_next_attempt(&mut state);

        assert_eq!(state.current_attempt, 1);
    }
}
