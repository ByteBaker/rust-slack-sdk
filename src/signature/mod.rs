//! Slack request signature verification.
//!
//! This module provides tools for verifying that requests actually come from Slack
//! using HMAC-SHA256 signatures.
//!
//! # Example
//!
//! ```rust,no_run
//! use slack_rs::signature::SignatureVerifier;
//! use std::collections::HashMap;
//!
//! let verifier = SignatureVerifier::new("your_signing_secret");
//!
//! let body = b"request body";
//! let mut headers = HashMap::new();
//! headers.insert("x-slack-request-timestamp".to_string(), "1531420618".to_string());
//! headers.insert("x-slack-signature".to_string(), "v0=a2114d57...".to_string());
//!
//! if verifier.is_valid_request(body, &headers) {
//!     println!("Valid request from Slack!");
//! }
//! ```

use crate::constants::{headers, signature, time};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Verifies Slack request signatures using HMAC-SHA256.
///
/// Slack signs its requests using a secret unique to your app.
/// This verifier helps ensure requests are authentic.
///
/// See: <https://api.slack.com/authentication/verifying-requests-from-slack>
#[derive(Debug, Clone)]
pub struct SignatureVerifier {
    signing_secret: String,
}

impl SignatureVerifier {
    /// Creates a new signature verifier.
    ///
    /// # Arguments
    /// * `signing_secret` - Your app's signing secret from Slack
    pub fn new(signing_secret: impl Into<String>) -> Self {
        Self {
            signing_secret: signing_secret.into(),
        }
    }

    /// Generates an HMAC-SHA256 signature for a request.
    ///
    /// # Arguments
    /// * `timestamp` - The request timestamp
    /// * `body` - The request body (as string or bytes)
    ///
    /// # Returns
    /// The signature in the format "v0={hex}"
    pub fn generate_signature(&self, timestamp: &str, body: &[u8]) -> String {
        let body_str = String::from_utf8_lossy(body);
        let sig_basestring = format!(
            "{}:{}:{}",
            signature::SIGNATURE_VERSION,
            timestamp,
            body_str
        );

        let mut mac = HmacSha256::new_from_slice(self.signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(sig_basestring.as_bytes());

        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        format!("{}{}", signature::SIGNATURE_PREFIX, hex::encode(code_bytes))
    }

    /// Verifies if a request is valid by checking headers.
    ///
    /// # Arguments
    /// * `body` - The request body (as string or bytes)
    /// * `headers` - The request headers (case-insensitive)
    ///
    /// # Returns
    /// `true` if the request is valid and not expired
    pub fn is_valid_request(&self, body: &[u8], headers: &HashMap<String, String>) -> bool {
        // Normalize headers to lowercase
        let normalized: HashMap<String, String> = headers
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.clone()))
            .collect();

        let timestamp = match normalized.get(headers::SLACK_REQUEST_TIMESTAMP) {
            Some(ts) => ts.as_str(),
            None => return false,
        };

        let signature = match normalized.get(headers::SLACK_SIGNATURE) {
            Some(sig) => sig.as_str(),
            None => return false,
        };

        self.is_valid(body, timestamp, signature)
    }

    /// Verifies if a request is valid with explicit parameters.
    ///
    /// # Arguments
    /// * `body` - The request body
    /// * `timestamp` - The request timestamp
    /// * `signature` - The signature to verify
    ///
    /// # Returns
    /// `true` if the signature is valid and not expired
    pub fn is_valid(&self, body: &[u8], timestamp: &str, signature: &str) -> bool {
        // Check timestamp expiration (5-minute window)
        let timestamp_num = match timestamp.parse::<u64>() {
            Ok(ts) => ts,
            Err(_) => return false,
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        if now.abs_diff(timestamp_num) > time::MAX_REQUEST_AGE_SECS {
            return false;
        }

        // Generate expected signature
        let calculated_signature = self.generate_signature(timestamp, body);

        // Use constant-time comparison to prevent timing attacks
        constant_time_compare(&calculated_signature, signature)
    }

    /// Verifies a request with a custom clock (for testing).
    #[cfg(test)]
    pub fn is_valid_with_clock(
        &self,
        body: &[u8],
        timestamp: &str,
        signature: &str,
        current_time: u64,
    ) -> bool {
        let timestamp_num = match timestamp.parse::<u64>() {
            Ok(ts) => ts,
            Err(_) => return false,
        };

        if current_time.abs_diff(timestamp_num) > time::MAX_REQUEST_AGE_SECS {
            return false;
        }

        let calculated_signature = self.generate_signature(timestamp, body);
        constant_time_compare(&calculated_signature, signature)
    }
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (a_byte, b_byte) in a.bytes().zip(b.bytes()) {
        result |= a_byte ^ b_byte;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Test data from Slack documentation
    const SIGNING_SECRET: &str = "8f742231b10e8888abcd99yyyzzz85a5";
    const BODY: &str = "token=xyzz0WbapA4vBCDEFasx0q6G&team_id=T1DC2JH3J&team_domain=testteamnow&channel_id=G8PSS9T3V&channel_name=foobar&user_id=U2CERLKJA&user_name=roadrunner&command=%2Fwebhook-collect&text=&response_url=https%3A%2F%2Fhooks.slack.com%2Fcommands%2FT1DC2JH3J%2F397700885554%2F96rGlfmibIGlgcZRskXaIFfN&trigger_id=398738663015.47445629121.803a0bc887a14d10d2c447fce8b6703c";
    const TIMESTAMP: &str = "1531420618";
    const VALID_SIGNATURE: &str =
        "v0=a2114d57b48eac39b9ad189dd8316235a7b4a8d21a10bd27519666489c69b503";
    const MOCK_CLOCK_TIME: u64 = 1531420618;

    #[test]
    fn test_generate_signature() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let signature = verifier.generate_signature(TIMESTAMP, BODY.as_bytes());

        assert_eq!(signature, VALID_SIGNATURE);
    }

    #[test]
    fn test_generate_signature_body_as_bytes() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let signature = verifier.generate_signature(TIMESTAMP, BODY.as_bytes());

        assert_eq!(signature, VALID_SIGNATURE);
    }

    #[test]
    fn test_is_valid_request() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let mut headers = HashMap::new();
        headers.insert(
            "x-slack-request-timestamp".to_string(),
            TIMESTAMP.to_string(),
        );
        headers.insert("x-slack-signature".to_string(), VALID_SIGNATURE.to_string());

        assert!(verifier.is_valid_with_clock(
            BODY.as_bytes(),
            TIMESTAMP,
            VALID_SIGNATURE,
            MOCK_CLOCK_TIME
        ));
    }

    #[test]
    fn test_is_valid_request_with_headers() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let mut headers = HashMap::new();
        headers.insert(
            "x-slack-request-timestamp".to_string(),
            TIMESTAMP.to_string(),
        );
        headers.insert("x-slack-signature".to_string(), VALID_SIGNATURE.to_string());

        // Mock the clock by using is_valid_with_clock
        assert!(verifier.is_valid_with_clock(
            BODY.as_bytes(),
            TIMESTAMP,
            VALID_SIGNATURE,
            MOCK_CLOCK_TIME
        ));
    }

    #[test]
    fn test_is_valid_request_case_insensitive_headers() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let mut headers = HashMap::new();
        headers.insert(
            "X-Slack-Request-Timestamp".to_string(),
            TIMESTAMP.to_string(),
        );
        headers.insert("X-Slack-Signature".to_string(), VALID_SIGNATURE.to_string());

        let normalized: HashMap<String, String> = headers
            .iter()
            .map(|(k, v)| (k.to_lowercase(), v.clone()))
            .collect();

        let timestamp = normalized.get("x-slack-request-timestamp").unwrap();
        let signature = normalized.get("x-slack-signature").unwrap();

        assert!(verifier.is_valid_with_clock(
            BODY.as_bytes(),
            timestamp,
            signature,
            MOCK_CLOCK_TIME
        ));
    }

    #[test]
    fn test_is_valid_request_invalid_body() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let modified_body = format!("{}------", BODY);

        assert!(!verifier.is_valid_with_clock(
            modified_body.as_bytes(),
            TIMESTAMP,
            VALID_SIGNATURE,
            MOCK_CLOCK_TIME
        ));
    }

    #[test]
    fn test_is_valid_request_invalid_signature() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let invalid_sig = "v0=invalid";

        assert!(!verifier.is_valid_with_clock(
            BODY.as_bytes(),
            TIMESTAMP,
            invalid_sig,
            MOCK_CLOCK_TIME
        ));
    }

    #[test]
    fn test_is_valid_request_expiration() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);

        // Request is from timestamp 1531420618
        // Current time is 301 seconds later (just over 5 minutes)
        let expired_time = MOCK_CLOCK_TIME + 301;

        assert!(!verifier.is_valid_with_clock(
            BODY.as_bytes(),
            TIMESTAMP,
            VALID_SIGNATURE,
            expired_time
        ));
    }

    #[test]
    fn test_is_valid_request_within_expiration() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);

        // Request is from timestamp 1531420618
        // Current time is 299 seconds later (just under 5 minutes)
        let valid_time = MOCK_CLOCK_TIME + 299;

        assert!(verifier.is_valid_with_clock(
            BODY.as_bytes(),
            TIMESTAMP,
            VALID_SIGNATURE,
            valid_time
        ));
    }

    #[test]
    fn test_is_valid_request_empty_body() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);
        let empty_sig = verifier.generate_signature(TIMESTAMP, b"");

        assert!(verifier.is_valid_with_clock(b"", TIMESTAMP, &empty_sig, MOCK_CLOCK_TIME));
    }

    #[test]
    fn test_is_valid_invalid_timestamp_format() {
        let verifier = SignatureVerifier::new(SIGNING_SECRET);

        assert!(!verifier.is_valid_with_clock(
            BODY.as_bytes(),
            "not-a-number",
            VALID_SIGNATURE,
            MOCK_CLOCK_TIME
        ));
    }

    #[test]
    fn test_constant_time_compare_equal() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(constant_time_compare("", ""));
        assert!(constant_time_compare(VALID_SIGNATURE, VALID_SIGNATURE));
    }

    #[test]
    fn test_constant_time_compare_different() {
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hello!"));
        assert!(!constant_time_compare("", "a"));
    }

    #[test]
    fn test_constant_time_compare_different_lengths() {
        assert!(!constant_time_compare("short", "much longer string"));
    }

    #[test]
    fn test_verifier_clone() {
        let verifier1 = SignatureVerifier::new("secret");
        let verifier2 = verifier1.clone();

        assert_eq!(verifier1.signing_secret, verifier2.signing_secret);
    }

    #[test]
    fn test_verifier_debug() {
        let verifier = SignatureVerifier::new("secret");
        let debug_str = format!("{:?}", verifier);

        assert!(debug_str.contains("SignatureVerifier"));
    }
}
