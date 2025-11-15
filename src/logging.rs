//! Logging infrastructure for the Slack SDK.
//!
//! This module provides structured logging using `tracing`, mirroring the
//! Python SDK's logging patterns.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initializes the global tracing subscriber with default settings.
///
/// This reads the `RUST_LOG` environment variable to configure log levels.
/// If not set, defaults to `info` level for the slack-rs crate.
///
/// # Example
///
/// ```rust
/// use slack_rs::logging::init_logging;
///
/// // Call once at application startup
/// init_logging();
/// ```
pub fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("slack_rs=info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

/// Initializes the global tracing subscriber with a custom filter.
///
/// # Arguments
///
/// * `filter` - A filter string (e.g., "slack_rs=debug,slack_rs::web=trace")
///
/// # Example
///
/// ```rust
/// use slack_rs::logging::init_logging_with_filter;
///
/// init_logging_with_filter("slack_rs=debug");
/// ```
pub fn init_logging_with_filter(filter: &str) {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::new(filter))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_no_panic() {
        // Just verify it doesn't panic
        // (Can only init once per process, so this might fail if other tests run first)
        let _ = tracing_subscriber::registry()
            .with(fmt::layer().with_test_writer())
            .with(EnvFilter::new("slack_rs=debug"))
            .try_init();
    }
}
