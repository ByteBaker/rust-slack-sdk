//! Internal utility functions for the Web API client.

use serde_json::Value;
use std::collections::HashMap;

/// Constructs the User-Agent header string.
///
/// The format follows: "{prefix} rust-slack-sdk/{version} rust/{rust_version} {os}/{os_version} {suffix}"
///
/// # Arguments
///
/// * `prefix` - Optional prefix to prepend to the user agent
/// * `suffix` - Optional suffix to append to the user agent
///
/// # Examples
///
/// ```
/// use slack_rs::web::internal_utils::get_user_agent;
///
/// let ua = get_user_agent(None, None);
/// assert!(ua.contains("rust-slack-sdk/"));
/// assert!(ua.contains("rust/"));
///
/// let ua_with_prefix = get_user_agent(Some("MyApp/1.0"), None);
/// assert!(ua_with_prefix.starts_with("MyApp/1.0 "));
/// ```
pub fn get_user_agent(prefix: Option<&str>, suffix: Option<&str>) -> String {
    let sdk_version = env!("CARGO_PKG_VERSION");
    let rust_version = rustc_version_runtime::version();
    let os = std::env::consts::OS;
    let os_version = os_info::get().version().to_string();

    let mut parts = Vec::new();

    if let Some(p) = prefix {
        parts.push(p.to_string());
    }

    parts.push(format!("rust-slack-sdk/{}", sdk_version));
    parts.push(format!("rust/{}", rust_version));
    parts.push(format!("{}/{}", os, os_version));

    if let Some(s) = suffix {
        parts.push(s.to_string());
    }

    parts.join(" ")
}

/// Joins the base Slack URL and an API method to form an absolute URL.
///
/// # Arguments
///
/// * `base_url` - The base URL (should end with /)
/// * `api_method` - The Slack Web API method (e.g., 'chat.postMessage')
///
/// # Examples
///
/// ```
/// use slack_rs::web::internal_utils::get_url;
///
/// let url = get_url("https://slack.com/api/", "chat.postMessage");
/// assert_eq!(url, "https://slack.com/api/chat.postMessage");
///
/// let url2 = get_url("https://slack.com/api", "api.test");
/// assert_eq!(url2, "https://slack.com/api/api.test");
/// ```
pub fn get_url(base_url: &str, api_method: &str) -> String {
    let base = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        format!("{}/", base_url)
    };

    let method = api_method.trim_start_matches('/');
    format!("{}{}", base, method)
}

/// Constructs the headers needed for a request.
///
/// # Arguments
///
/// * `token` - Optional Slack API token
/// * `has_json` - Whether the request body is JSON
/// * `has_files` - Whether the request includes file uploads
/// * `default_headers` - Headers set at client initialization
/// * `request_headers` - Headers specific to this request
///
/// # Returns
///
/// A HashMap of header name/value pairs.
pub fn get_headers(
    token: Option<&str>,
    has_json: bool,
    has_files: bool,
    default_headers: &HashMap<String, String>,
    request_headers: Option<&HashMap<String, String>>,
) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // Set default Content-Type
    if !has_files {
        if has_json {
            headers.insert(
                "Content-Type".to_string(),
                "application/json;charset=utf-8".to_string(),
            );
        } else {
            headers.insert(
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            );
        }
    }
    // For files, let reqwest set the multipart boundary automatically

    // Add User-Agent if not present
    if !default_headers.contains_key("User-Agent") {
        headers.insert("User-Agent".to_string(), get_user_agent(None, None));
    }

    // Add authorization header
    if let Some(t) = token {
        headers.insert("Authorization".to_string(), format!("Bearer {}", t));
    }

    // Merge default headers (from client initialization)
    for (key, value) in default_headers {
        headers.insert(key.clone(), value.clone());
    }

    // Merge request-specific headers (highest priority)
    if let Some(req_headers) = request_headers {
        for (key, value) in req_headers {
            headers.insert(key.clone(), value.clone());
        }
    }

    headers
}

/// Converts boolean values to "0" or "1" strings.
///
/// Slack APIs accept "0"/"1" as boolean values, which is more reliable
/// across different HTTP client implementations.
pub fn convert_bool_to_0_or_1(value: &Value) -> Value {
    match value {
        Value::Bool(b) => Value::String(if *b { "1" } else { "0" }.to_string()),
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                new_map.insert(k.clone(), convert_bool_to_0_or_1(v));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(convert_bool_to_0_or_1).collect()),
        _ => value.clone(),
    }
}

/// Removes entries with null values from a JSON object.
///
/// This helps keep request payloads clean and matches Python SDK behavior.
pub fn remove_none_values(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                if !v.is_null() {
                    new_map.insert(k, remove_none_values(v));
                }
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(remove_none_values).collect()),
        _ => value,
    }
}

/// Checks if a cursor is present in the response for pagination.
pub fn next_cursor_is_present(data: &Value) -> bool {
    // Check response_metadata.next_cursor
    if let Some(metadata) = data.get("response_metadata") {
        if let Some(cursor) = metadata.get("next_cursor") {
            if let Some(cursor_str) = cursor.as_str() {
                if !cursor_str.is_empty() {
                    return true;
                }
            }
        }
    }

    // Check top-level next_cursor
    if let Some(cursor) = data.get("next_cursor") {
        if let Some(cursor_str) = cursor.as_str() {
            if !cursor_str.is_empty() {
                return true;
            }
        }
    }

    false
}

// We need to add these dependencies to Cargo.toml
// For now, let's provide a simple implementation that doesn't require external crates
// We'll use a simpler version detection

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_get_url() {
        assert_eq!(
            get_url("https://slack.com/api/", "chat.postMessage"),
            "https://slack.com/api/chat.postMessage"
        );

        assert_eq!(
            get_url("https://slack.com/api", "api.test"),
            "https://slack.com/api/api.test"
        );

        assert_eq!(
            get_url("https://slack.com/api/", "/users.list"),
            "https://slack.com/api/users.list"
        );
    }

    #[test]
    fn test_get_user_agent() {
        let ua = get_user_agent(None, None);
        assert!(ua.contains("rust-slack-sdk/"));
        assert!(ua.contains("rust/"));
    }

    #[test]
    fn test_get_user_agent_with_prefix() {
        let ua = get_user_agent(Some("MyApp/1.0"), None);
        assert!(ua.starts_with("MyApp/1.0 "));
    }

    #[test]
    fn test_get_user_agent_with_suffix() {
        let ua = get_user_agent(None, Some("custom"));
        assert!(ua.ends_with(" custom"));
    }

    #[test]
    fn test_get_headers_basic() {
        let default_headers = HashMap::new();
        let headers = get_headers(Some("xoxb-token"), false, false, &default_headers, None);

        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/x-www-form-urlencoded".to_string())
        );
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer xoxb-token".to_string())
        );
        assert!(headers.contains_key("User-Agent"));
    }

    #[test]
    fn test_get_headers_json() {
        let default_headers = HashMap::new();
        let headers = get_headers(Some("xoxb-token"), true, false, &default_headers, None);

        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json;charset=utf-8".to_string())
        );
    }

    #[test]
    fn test_get_headers_files() {
        let default_headers = HashMap::new();
        let headers = get_headers(Some("xoxb-token"), false, true, &default_headers, None);

        // Content-Type should not be set for file uploads
        assert!(!headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_get_headers_merge() {
        let mut default_headers = HashMap::new();
        default_headers.insert("X-Default".to_string(), "default-value".to_string());

        let mut request_headers = HashMap::new();
        request_headers.insert("X-Request".to_string(), "request-value".to_string());

        let headers = get_headers(
            Some("xoxb-token"),
            false,
            false,
            &default_headers,
            Some(&request_headers),
        );

        assert_eq!(headers.get("X-Default"), Some(&"default-value".to_string()));
        assert_eq!(headers.get("X-Request"), Some(&"request-value".to_string()));
    }

    #[test]
    fn test_convert_bool_to_0_or_1() {
        assert_eq!(convert_bool_to_0_or_1(&json!(true)), json!("1"));
        assert_eq!(convert_bool_to_0_or_1(&json!(false)), json!("0"));

        let input = json!({"active": true, "archived": false, "name": "test"});
        let output = convert_bool_to_0_or_1(&input);
        assert_eq!(output["active"], json!("1"));
        assert_eq!(output["archived"], json!("0"));
        assert_eq!(output["name"], json!("test"));
    }

    #[test]
    fn test_remove_none_values() {
        let input = json!({
            "name": "test",
            "value": null,
            "nested": {
                "keep": "this",
                "remove": null
            }
        });

        let output = remove_none_values(input);
        assert!(output.get("name").is_some());
        assert!(output.get("value").is_none());
        assert!(output["nested"].get("keep").is_some());
        assert!(output["nested"].get("remove").is_none());
    }

    #[test]
    fn test_next_cursor_is_present() {
        // With cursor in response_metadata
        let data1 = json!({
            "ok": true,
            "response_metadata": {
                "next_cursor": "dGVhbTpDMDYxRkE1UEI="
            }
        });
        assert!(next_cursor_is_present(&data1));

        // With empty cursor
        let data2 = json!({
            "ok": true,
            "response_metadata": {
                "next_cursor": ""
            }
        });
        assert!(!next_cursor_is_present(&data2));

        // Without cursor
        let data3 = json!({"ok": true});
        assert!(!next_cursor_is_present(&data3));

        // With top-level cursor
        let data4 = json!({
            "ok": true,
            "next_cursor": "abc123"
        });
        assert!(next_cursor_is_present(&data4));
    }
}
