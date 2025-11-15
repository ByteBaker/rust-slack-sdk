//! Retry state management.
//!
//! This module provides the state machine for tracking retry attempts and timing.

use std::time::{Duration, Instant};

/// The state of a retry operation.
///
/// This struct tracks the current attempt number, timing information,
/// and other metadata needed to make retry decisions.
#[derive(Debug, Clone)]
pub struct RetryState {
    /// The current attempt number (0-indexed).
    pub current_attempt: u32,

    /// When the next retry attempt should be made.
    pub next_attempt_requested_at: Instant,

    /// When the first attempt was made.
    pub first_attempt_at: Instant,

    /// The error from the last attempt, if any.
    pub last_error: Option<String>,
}

impl Default for RetryState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            current_attempt: 0,
            next_attempt_requested_at: now,
            first_attempt_at: now,
            last_error: None,
        }
    }
}

impl RetryState {
    /// Creates a new retry state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increments the attempt counter.
    pub fn increment_attempt(&mut self) {
        self.current_attempt += 1;
    }

    /// Sets when the next attempt should be made.
    pub fn set_next_attempt(&mut self, duration: Duration) {
        self.next_attempt_requested_at = Instant::now() + duration;
    }

    /// Returns the duration since the first attempt.
    pub fn elapsed_since_first_attempt(&self) -> Duration {
        Instant::now().duration_since(self.first_attempt_at)
    }

    /// Returns the duration until the next attempt should be made.
    pub fn duration_until_next_attempt(&self) -> Duration {
        self.next_attempt_requested_at
            .saturating_duration_since(Instant::now())
    }

    /// Returns whether it's time to make the next attempt.
    pub fn should_attempt_now(&self) -> bool {
        Instant::now() >= self.next_attempt_requested_at
    }

    /// Sets the last error message.
    pub fn set_last_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
    }

    /// Clears the last error.
    pub fn clear_last_error(&mut self) {
        self.last_error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_retry_state_default() {
        let state = RetryState::default();
        assert_eq!(state.current_attempt, 0);
        assert!(state.last_error.is_none());
    }

    #[test]
    fn test_retry_state_new() {
        let state = RetryState::new();
        assert_eq!(state.current_attempt, 0);
    }

    #[test]
    fn test_increment_attempt() {
        let mut state = RetryState::new();
        assert_eq!(state.current_attempt, 0);

        state.increment_attempt();
        assert_eq!(state.current_attempt, 1);

        state.increment_attempt();
        assert_eq!(state.current_attempt, 2);
    }

    #[test]
    fn test_set_next_attempt() {
        let mut state = RetryState::new();
        let duration = Duration::from_secs(5);

        state.set_next_attempt(duration);

        // Should be approximately 5 seconds from now
        let until_next = state.duration_until_next_attempt();
        assert!(until_next <= duration);
        assert!(until_next >= Duration::from_secs(4)); // Allow for small timing variance
    }

    #[test]
    fn test_elapsed_since_first_attempt() {
        let state = RetryState::new();

        // Should be very close to zero immediately after creation
        let elapsed = state.elapsed_since_first_attempt();
        assert!(elapsed < Duration::from_millis(100));

        // Wait a bit and check again
        sleep(Duration::from_millis(10));
        let elapsed = state.elapsed_since_first_attempt();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_should_attempt_now_initially_true() {
        let state = RetryState::new();
        // Should be true immediately
        assert!(state.should_attempt_now());
    }

    #[test]
    fn test_should_attempt_now_false_when_delayed() {
        let mut state = RetryState::new();
        state.set_next_attempt(Duration::from_secs(10));

        // Should be false since we just set a future time
        assert!(!state.should_attempt_now());
    }

    #[test]
    fn test_duration_until_next_attempt_saturating() {
        let mut state = RetryState::new();

        // Set next attempt to the past
        state.next_attempt_requested_at = Instant::now() - Duration::from_secs(5);

        // Should return zero (saturating)
        let duration = state.duration_until_next_attempt();
        assert_eq!(duration, Duration::ZERO);
    }

    #[test]
    fn test_set_last_error() {
        let mut state = RetryState::new();
        assert!(state.last_error.is_none());

        state.set_last_error("Connection timeout");
        assert_eq!(state.last_error, Some("Connection timeout".to_string()));

        state.set_last_error(String::from("Rate limited"));
        assert_eq!(state.last_error, Some("Rate limited".to_string()));
    }

    #[test]
    fn test_clear_last_error() {
        let mut state = RetryState::new();
        state.set_last_error("Some error");
        assert!(state.last_error.is_some());

        state.clear_last_error();
        assert!(state.last_error.is_none());
    }

    #[test]
    fn test_retry_state_clone() {
        let mut state = RetryState::new();
        state.increment_attempt();
        state.set_last_error("Test error");

        let cloned = state.clone();
        assert_eq!(cloned.current_attempt, 1);
        assert_eq!(cloned.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_retry_state_debug() {
        let state = RetryState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("RetryState"));
        assert!(debug_str.contains("current_attempt"));
    }
}
