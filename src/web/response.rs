//! Slack API response types and iterators for pagination.

use crate::error::{Result, SlackError};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;

/// A response from the Slack Web API.
///
/// This type wraps the JSON response data along with HTTP metadata like
/// status code and headers. It implements dictionary-like access patterns
/// and supports automatic pagination through the Iterator trait.
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// # use slack_rs::web::AsyncWebClient;
/// # async fn example() -> slack_rs::error::Result<()> {
/// let client = AsyncWebClient::new("xoxb-token");
/// let response = client.api_test(None).await?;
///
/// // Dictionary-style access
/// if response["ok"].as_bool().unwrap_or(false) {
///     println!("API test successful!");
/// }
/// # Ok(())
/// # }
/// ```
///
/// Pagination:
/// ```no_run
/// # use slack_rs::web::AsyncWebClient;
/// # use serde_json::json;
/// # async fn example() -> slack_rs::error::Result<()> {
/// let client = AsyncWebClient::new("xoxb-token");
/// let params = serde_json::json!({ "limit": 100 });
///
/// // Automatically paginate through all users
/// let mut all_users = Vec::new();
/// let mut response = client.users_list(Some(params)).await?;
/// loop {
///     if let Some(members) = response["members"].as_array() {
///         all_users.extend(members.clone());
///     }
///     if !response.has_next_cursor() {
///         break;
///     }
///     response = response.next().await?;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct SlackResponse {
    /// The HTTP verb used for the request (GET, POST, etc.)
    pub http_verb: String,

    /// The full API URL that was called
    pub api_url: String,

    /// The JSON-decoded response data
    pub data: Value,

    /// HTTP response headers
    pub headers: HeaderMap,

    /// HTTP status code
    pub status_code: u16,

    /// The original request arguments (for pagination)
    req_args: RequestArgs,

    /// Reference to the client for pagination
    client_ref: Option<ClientRef>,
}

/// Internal structure to hold request arguments for pagination
#[derive(Debug, Clone)]
struct RequestArgs {
    params: HashMap<String, Value>,
    json: Option<Value>,
    headers: HashMap<String, String>,
}

/// Reference to the client for making pagination requests
#[derive(Debug, Clone)]
#[allow(dead_code)] // base_url will be used for pagination in future
struct ClientRef {
    token: Option<String>,
    base_url: String,
}

impl SlackResponse {
    /// Creates a new SlackResponse.
    ///
    /// # Arguments
    ///
    /// * `http_verb` - The HTTP method used (GET, POST, etc.)
    /// * `api_url` - The full URL of the API endpoint
    /// * `data` - The parsed JSON response data
    /// * `headers` - The HTTP response headers
    /// * `status_code` - The HTTP status code
    pub fn new(
        http_verb: String,
        api_url: String,
        data: Value,
        headers: HeaderMap,
        status_code: u16,
    ) -> Self {
        Self {
            http_verb,
            api_url,
            data,
            headers,
            status_code,
            req_args: RequestArgs {
                params: HashMap::new(),
                json: None,
                headers: HashMap::new(),
            },
            client_ref: None,
        }
    }

    /// Sets the request arguments for pagination support.
    pub fn with_request_args(
        mut self,
        params: HashMap<String, Value>,
        json: Option<Value>,
        headers: HashMap<String, String>,
    ) -> Self {
        self.req_args = RequestArgs {
            params,
            json,
            headers,
        };
        self
    }

    /// Sets the client reference for pagination support.
    pub fn with_client_ref(mut self, token: Option<String>, base_url: String) -> Self {
        self.client_ref = Some(ClientRef { token, base_url });
        self
    }

    /// Gets a value from the response data by key.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use slack_rs::web::SlackResponse;
    /// # use serde_json::json;
    /// # fn example(response: SlackResponse) {
    /// let ok = response.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    /// let user_id = response.get("user").and_then(|v| v.get("id")).and_then(|v| v.as_str());
    /// # }
    /// ```
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Checks if the response contains a key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.get(key).is_some()
    }

    /// Validates that the response indicates success.
    ///
    /// A response is considered successful if:
    /// - Status code is 200
    /// - The "ok" field is true (if present)
    ///
    /// # Errors
    ///
    /// Returns `SlackError::ApiError` if the response indicates failure.
    pub fn validate(self) -> Result<Self> {
        if self.status_code == 200
            && self
                .data
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
        {
            Ok(self)
        } else {
            let error_msg = self
                .data
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_error");
            Err(SlackError::ApiError {
                message: format!(
                    "The request to the Slack API failed: {} (url: {})",
                    error_msg, self.api_url
                ),
                response: self.data.clone(),
            })
        }
    }

    /// Checks if there's a next cursor for pagination.
    ///
    /// Returns true if the response contains a `next_cursor` field
    /// in either the top level or in `response_metadata`.
    pub fn has_next_cursor(&self) -> bool {
        self.get_next_cursor().is_some()
    }

    /// Gets the next cursor value for pagination.
    pub fn get_next_cursor(&self) -> Option<String> {
        // Check response_metadata.next_cursor first
        if let Some(metadata) = self.data.get("response_metadata") {
            if let Some(cursor) = metadata.get("next_cursor").and_then(|v| v.as_str()) {
                if !cursor.is_empty() {
                    return Some(cursor.to_string());
                }
            }
        }

        // Check top-level next_cursor
        if let Some(cursor) = self.data.get("next_cursor").and_then(|v| v.as_str()) {
            if !cursor.is_empty() {
                return Some(cursor.to_string());
            }
        }

        None
    }

    /// Fetches the next page of results using the cursor.
    ///
    /// This method makes a new API request with the cursor parameter
    /// set to continue pagination.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - There is no next cursor
    /// - The API request fails
    /// - No client reference was set
    pub async fn next(self) -> Result<Self> {
        let next_cursor = self
            .get_next_cursor()
            .ok_or_else(|| SlackError::PaginationError("No next_cursor present".to_string()))?;

        let client_ref = self
            .client_ref
            .as_ref()
            .ok_or_else(|| SlackError::PaginationError("No client reference set".to_string()))?;

        // Create updated params with cursor
        let mut params = self.req_args.params.clone();
        params.insert("cursor".to_string(), Value::String(next_cursor));

        // Make the pagination request using reqwest
        let http_client = reqwest::Client::new();
        let mut req = http_client.post(&self.api_url);

        // Add authorization header if token exists
        if let Some(ref token) = client_ref.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        // Add custom headers
        for (key, value) in &self.req_args.headers {
            req = req.header(key, value);
        }

        // Send request with params as JSON body
        req = req.json(&params);

        let response = req.send().await.map_err(|e| SlackError::HttpError {
            message: format!("Pagination request failed: {}", e),
        })?;

        let status_code = response.status().as_u16();
        let headers = response.headers().clone();
        let data: Value = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse pagination response: {}", e),
        })?;

        Ok(SlackResponse {
            http_verb: "POST".to_string(),
            api_url: self.api_url,
            data,
            headers,
            status_code,
            req_args: RequestArgs {
                params,
                json: self.req_args.json,
                headers: self.req_args.headers,
            },
            client_ref: self.client_ref,
        }
        .validate()?)
    }
}

// Implement Index trait for dictionary-style access
impl std::ops::Index<&str> for SlackResponse {
    type Output = Value;

    fn index(&self, key: &str) -> &Self::Output {
        self.data
            .get(key)
            .unwrap_or_else(|| panic!("Key '{}' not found in response", key))
    }
}

// Implement Display for easy printing
impl std::fmt::Display for SlackResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use serde_json::json;

    #[test]
    fn test_slack_response_creation() {
        let data = json!({"ok": true, "message": "Success"});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/api.test".to_string(),
            data.clone(),
            HeaderMap::new(),
            200,
        );

        assert_eq!(response.http_verb, "POST");
        assert_eq!(response.status_code, 200);
        assert_eq!(response.data, data);
    }

    #[test]
    fn test_get_method() {
        let data = json!({"ok": true, "user": {"id": "U123", "name": "Test User"}});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/users.info".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        assert!(response.get("ok").is_some());
        assert_eq!(response.get("ok").unwrap(), &json!(true));
        assert!(response.get("user").is_some());
        assert!(response.get("nonexistent").is_none());
    }

    #[test]
    fn test_index_operator() {
        let data = json!({"ok": true, "channel": "C123"});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/chat.postMessage".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        assert_eq!(response["ok"], json!(true));
        assert_eq!(response["channel"], json!("C123"));
    }

    #[test]
    #[should_panic(expected = "Key 'missing' not found")]
    fn test_index_operator_panic() {
        let data = json!({"ok": true});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/api.test".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        let _ = &response["missing"];
    }

    #[test]
    fn test_validate_success() {
        let data = json!({"ok": true});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/api.test".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        assert!(response.validate().is_ok());
    }

    #[test]
    fn test_validate_failure() {
        let data = json!({"ok": false, "error": "invalid_auth"});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/chat.postMessage".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        let result = response.validate();
        assert!(result.is_err());
        if let Err(SlackError::ApiError { message, .. }) = result {
            assert!(message.contains("invalid_auth"));
        } else {
            panic!("Expected ApiError");
        }
    }

    #[test]
    fn test_has_next_cursor() {
        // Test with response_metadata.next_cursor
        let data1 = json!({
            "ok": true,
            "response_metadata": {
                "next_cursor": "dGVhbTpDMDYxRkE1UEI="
            }
        });
        let response1 = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/users.list".to_string(),
            data1,
            HeaderMap::new(),
            200,
        );
        assert!(response1.has_next_cursor());

        // Test with top-level next_cursor
        let data2 = json!({
            "ok": true,
            "next_cursor": "abc123"
        });
        let response2 = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/conversations.list".to_string(),
            data2,
            HeaderMap::new(),
            200,
        );
        assert!(response2.has_next_cursor());

        // Test with empty cursor
        let data3 = json!({
            "ok": true,
            "response_metadata": {
                "next_cursor": ""
            }
        });
        let response3 = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/users.list".to_string(),
            data3,
            HeaderMap::new(),
            200,
        );
        assert!(!response3.has_next_cursor());

        // Test without cursor
        let data4 = json!({"ok": true});
        let response4 = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/api.test".to_string(),
            data4,
            HeaderMap::new(),
            200,
        );
        assert!(!response4.has_next_cursor());
    }

    #[test]
    fn test_get_next_cursor() {
        let data = json!({
            "ok": true,
            "response_metadata": {
                "next_cursor": "dGVhbTpDMDYxRkE1UEI="
            }
        });
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/users.list".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        assert_eq!(
            response.get_next_cursor(),
            Some("dGVhbTpDMDYxRkE1UEI=".to_string())
        );
    }

    #[test]
    fn test_contains_key() {
        let data = json!({"ok": true, "user": "U123"});
        let response = SlackResponse::new(
            "POST".to_string(),
            "https://slack.com/api/users.info".to_string(),
            data,
            HeaderMap::new(),
            200,
        );

        assert!(response.contains_key("ok"));
        assert!(response.contains_key("user"));
        assert!(!response.contains_key("missing"));
    }
}
