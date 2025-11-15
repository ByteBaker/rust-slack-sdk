//! Interval calculation for retry delays.
//!
//! This module provides calculators for determining how long to wait between retry attempts.

use rand::Rng;
use std::time::Duration;

/// Trait for calculating retry intervals.
pub trait IntervalCalculator: Send + Sync {
    /// Calculates the interval to wait before the next attempt.
    ///
    /// # Arguments
    /// * `attempt` - The attempt number (1-indexed)
    ///
    /// # Returns
    /// The duration to wait before the next attempt.
    fn calculate(&self, attempt: u32) -> Duration;
}

/// Exponential backoff interval calculator.
///
/// Calculates intervals using exponential backoff: `base * multiplier^(attempt-1)`.
#[derive(Debug, Clone)]
pub struct BackoffIntervalCalculator {
    /// The base interval in seconds (default: 1).
    base_seconds: u64,

    /// The multiplier for exponential growth (default: 2.0).
    multiplier: f64,

    /// The maximum interval in seconds (optional).
    max_seconds: Option<u64>,
}

impl Default for BackoffIntervalCalculator {
    fn default() -> Self {
        Self {
            base_seconds: 1,
            multiplier: 2.0,
            max_seconds: Some(300), // 5 minutes max
        }
    }
}

impl BackoffIntervalCalculator {
    /// Creates a new backoff calculator with the given multiplier.
    pub fn new(multiplier: f64) -> Self {
        Self {
            base_seconds: 1,
            multiplier,
            max_seconds: Some(300),
        }
    }

    /// Sets the base interval in seconds.
    pub fn with_base(mut self, base_seconds: u64) -> Self {
        self.base_seconds = base_seconds;
        self
    }

    /// Sets the maximum interval in seconds.
    pub fn with_max_interval(mut self, max_seconds: u64) -> Self {
        self.max_seconds = Some(max_seconds);
        self
    }

    /// Removes the maximum interval limit.
    pub fn without_max_interval(mut self) -> Self {
        self.max_seconds = None;
        self
    }

    /// Calculates the interval for a given attempt using exponential backoff.
    fn calculate_backoff(&self, attempt: u32) -> u64 {
        if attempt == 0 {
            return 0;
        }

        let exponent = (attempt - 1) as i32;
        let interval = (self.base_seconds as f64) * self.multiplier.powi(exponent);

        interval.round() as u64
    }
}

impl IntervalCalculator for BackoffIntervalCalculator {
    fn calculate(&self, attempt: u32) -> Duration {
        let seconds = self.calculate_backoff(attempt);

        let capped_seconds = if let Some(max) = self.max_seconds {
            seconds.min(max)
        } else {
            seconds
        };

        Duration::from_secs(capped_seconds)
    }
}

/// Random jitter interval calculator.
///
/// Adds random jitter to a base interval to prevent thundering herd problems.
#[derive(Debug, Clone)]
pub struct RandomJitterCalculator {
    /// The base interval in seconds.
    base_seconds: u64,

    /// The maximum jitter to add (as a fraction of base, 0.0-1.0).
    jitter_factor: f64,
}

impl Default for RandomJitterCalculator {
    fn default() -> Self {
        Self {
            base_seconds: 1,
            jitter_factor: 0.5, // Add up to 50% jitter
        }
    }
}

impl RandomJitterCalculator {
    /// Creates a new random jitter calculator.
    pub fn new(base_seconds: u64, jitter_factor: f64) -> Self {
        Self {
            base_seconds,
            jitter_factor: jitter_factor.clamp(0.0, 1.0),
        }
    }
}

impl IntervalCalculator for RandomJitterCalculator {
    fn calculate(&self, _attempt: u32) -> Duration {
        let mut rng = rand::thread_rng();
        let jitter = rng.gen::<f64>() * self.jitter_factor;
        let total = self.base_seconds as f64 * (1.0 + jitter);

        Duration::from_millis((total * 1000.0) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculator_default() {
        let calc = BackoffIntervalCalculator::default();
        assert_eq!(calc.base_seconds, 1);
        assert_eq!(calc.multiplier, 2.0);
        assert_eq!(calc.max_seconds, Some(300));
    }

    #[test]
    fn test_backoff_calculator_new() {
        let calc = BackoffIntervalCalculator::new(3.0);
        assert_eq!(calc.multiplier, 3.0);
    }

    #[test]
    fn test_backoff_calculator_with_base() {
        let calc = BackoffIntervalCalculator::new(2.0).with_base(5);
        assert_eq!(calc.base_seconds, 5);
    }

    #[test]
    fn test_backoff_calculator_with_max_interval() {
        let calc = BackoffIntervalCalculator::new(2.0).with_max_interval(10);
        assert_eq!(calc.max_seconds, Some(10));
    }

    #[test]
    fn test_backoff_calculator_without_max_interval() {
        let calc = BackoffIntervalCalculator::new(2.0).without_max_interval();
        assert_eq!(calc.max_seconds, None);
    }

    #[test]
    fn test_backoff_calculation_zero_attempt() {
        let calc = BackoffIntervalCalculator::new(2.0);
        assert_eq!(calc.calculate(0), Duration::ZERO);
    }

    #[test]
    fn test_backoff_calculation_exponential_growth() {
        let calc = BackoffIntervalCalculator::new(2.0);

        // base * 2^0 = 1 * 1 = 1
        assert_eq!(calc.calculate(1), Duration::from_secs(1));

        // base * 2^1 = 1 * 2 = 2
        assert_eq!(calc.calculate(2), Duration::from_secs(2));

        // base * 2^2 = 1 * 4 = 4
        assert_eq!(calc.calculate(3), Duration::from_secs(4));

        // base * 2^3 = 1 * 8 = 8
        assert_eq!(calc.calculate(4), Duration::from_secs(8));

        // base * 2^4 = 1 * 16 = 16
        assert_eq!(calc.calculate(5), Duration::from_secs(16));
    }

    #[test]
    fn test_backoff_calculation_with_max_cap() {
        let calc = BackoffIntervalCalculator::new(2.0).with_max_interval(10);

        assert_eq!(calc.calculate(1), Duration::from_secs(1));
        assert_eq!(calc.calculate(2), Duration::from_secs(2));
        assert_eq!(calc.calculate(3), Duration::from_secs(4));
        assert_eq!(calc.calculate(4), Duration::from_secs(8));

        // Should be capped at 10
        assert_eq!(calc.calculate(5), Duration::from_secs(10));
        assert_eq!(calc.calculate(6), Duration::from_secs(10));
        assert_eq!(calc.calculate(100), Duration::from_secs(10));
    }

    #[test]
    fn test_backoff_calculation_custom_base() {
        let calc = BackoffIntervalCalculator::new(2.0).with_base(3);

        // base * 2^0 = 3 * 1 = 3
        assert_eq!(calc.calculate(1), Duration::from_secs(3));

        // base * 2^1 = 3 * 2 = 6
        assert_eq!(calc.calculate(2), Duration::from_secs(6));

        // base * 2^2 = 3 * 4 = 12
        assert_eq!(calc.calculate(3), Duration::from_secs(12));
    }

    #[test]
    fn test_backoff_calculation_custom_multiplier() {
        let calc = BackoffIntervalCalculator::new(3.0);

        // base * 3^0 = 1 * 1 = 1
        assert_eq!(calc.calculate(1), Duration::from_secs(1));

        // base * 3^1 = 1 * 3 = 3
        assert_eq!(calc.calculate(2), Duration::from_secs(3));

        // base * 3^2 = 1 * 9 = 9
        assert_eq!(calc.calculate(3), Duration::from_secs(9));

        // base * 3^3 = 1 * 27 = 27
        assert_eq!(calc.calculate(4), Duration::from_secs(27));
    }

    #[test]
    fn test_random_jitter_calculator_default() {
        let calc = RandomJitterCalculator::default();
        assert_eq!(calc.base_seconds, 1);
        assert_eq!(calc.jitter_factor, 0.5);
    }

    #[test]
    fn test_random_jitter_calculator_new() {
        let calc = RandomJitterCalculator::new(5, 0.3);
        assert_eq!(calc.base_seconds, 5);
        assert_eq!(calc.jitter_factor, 0.3);
    }

    #[test]
    fn test_random_jitter_factor_clamped() {
        let calc1 = RandomJitterCalculator::new(1, -0.5);
        assert_eq!(calc1.jitter_factor, 0.0);

        let calc2 = RandomJitterCalculator::new(1, 1.5);
        assert_eq!(calc2.jitter_factor, 1.0);

        let calc3 = RandomJitterCalculator::new(1, 0.5);
        assert_eq!(calc3.jitter_factor, 0.5);
    }

    #[test]
    fn test_random_jitter_calculation_range() {
        let calc = RandomJitterCalculator::new(10, 0.5);

        // Run multiple times to test randomness
        for _ in 0..100 {
            let duration = calc.calculate(1);
            let secs = duration.as_secs_f64();

            // Should be between base and base * (1 + jitter_factor)
            // 10 to 15 seconds
            assert!(secs >= 10.0);
            assert!(secs <= 15.0);
        }
    }

    #[test]
    fn test_random_jitter_ignores_attempt_number() {
        let calc = RandomJitterCalculator::new(5, 0.5);

        // All attempts should be in the same range
        for attempt in 1..=10 {
            let duration = calc.calculate(attempt);
            let secs = duration.as_secs_f64();
            assert!(secs >= 5.0);
            assert!(secs <= 7.5);
        }
    }

    #[test]
    fn test_backoff_calculator_clone() {
        let calc1 = BackoffIntervalCalculator::new(2.0).with_max_interval(100);
        let calc2 = calc1.clone();

        assert_eq!(calc1.multiplier, calc2.multiplier);
        assert_eq!(calc1.max_seconds, calc2.max_seconds);
    }

    #[test]
    fn test_random_jitter_calculator_clone() {
        let calc1 = RandomJitterCalculator::new(5, 0.3);
        let calc2 = calc1.clone();

        assert_eq!(calc1.base_seconds, calc2.base_seconds);
        assert_eq!(calc1.jitter_factor, calc2.jitter_factor);
    }
}
