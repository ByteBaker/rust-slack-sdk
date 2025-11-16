//! Integration tests for the Web API client.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use slack_rs::web::{AsyncWebClient, WebClient};
use std::sync::Arc;
use tokio::net::TcpListener;

/// Mock Slack API server state
#[derive(Clone)]
struct MockSlackServer {
    /// Expected token for auth
    #[allow(dead_code)]
    token: String,
}

/// Handler for api.test endpoint
async fn handle_api_test(
    State(_state): State<Arc<MockSlackServer>>,
    Json(body): Json<Value>,
) -> Response {
    // api.test is open and doesn't require authentication
    let mut response = json!({
        "ok": true,
    });

    // Echo back any parameters
    if let Some(obj) = body.as_object() {
        for (key, value) in obj {
            response[key] = value.clone();
        }
    }

    Json(response).into_response()
}

/// Handler for auth.test endpoint
async fn handle_auth_test(State(_state): State<Arc<MockSlackServer>>) -> Response {
    Json(json!({
        "ok": true,
        "url": "https://test-workspace.slack.com/",
        "team": "Test Workspace",
        "user": "test_user",
        "team_id": "T123456",
        "user_id": "U123456",
        "bot_id": "B123456"
    }))
    .into_response()
}

/// Handler for chat.postMessage endpoint
async fn handle_chat_post_message(
    State(_state): State<Arc<MockSlackServer>>,
    Json(body): Json<Value>,
) -> Response {
    let channel = body["channel"].as_str().unwrap_or("C123456");
    let text = body["text"].as_str().unwrap_or("");

    Json(json!({
        "ok": true,
        "channel": channel,
        "ts": "1234567890.123456",
        "message": {
            "type": "message",
            "user": "U123456",
            "text": text,
            "ts": "1234567890.123456"
        }
    }))
    .into_response()
}

/// Handler for users.list endpoint (with pagination support)
async fn handle_users_list(
    State(_state): State<Arc<MockSlackServer>>,
    Json(body): Json<Value>,
) -> Response {
    let cursor = body["cursor"].as_str().unwrap_or("");
    let limit = body["limit"].as_u64().unwrap_or(100) as usize;

    // Simulate paginated users
    let all_users: Vec<Value> = (1..=250)
        .map(|i| {
            json!({
                "id": format!("U{:06}", i),
                "name": format!("user{}", i),
                "real_name": format!("User {}", i)
            })
        })
        .collect();

    let start = if cursor.is_empty() {
        0
    } else {
        cursor.parse::<usize>().unwrap_or(0)
    };

    let end = (start + limit).min(all_users.len());
    let page = &all_users[start..end];

    let mut response = json!({
        "ok": true,
        "members": page,
    });

    // Add next_cursor if there are more results
    if end < all_users.len() {
        response["response_metadata"] = json!({
            "next_cursor": end.to_string()
        });
    }

    Json(response).into_response()
}

/// Handler for error responses
async fn handle_error() -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "ok": false,
            "error": "invalid_auth"
        })),
    )
        .into_response()
}

/// Handler for rate limit responses
async fn handle_rate_limit() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(header::RETRY_AFTER, "1")],
        Json(json!({
            "ok": false,
            "error": "rate_limited"
        })),
    )
        .into_response()
}

/// Start mock Slack API server
async fn start_mock_server() -> (String, tokio::task::JoinHandle<()>) {
    let state = Arc::new(MockSlackServer {
        token: "xoxb-test-token".to_string(),
    });

    let app = Router::new()
        .route("/api/api.test", post(handle_api_test))
        .route("/api/auth.test", post(handle_auth_test))
        .route("/api/chat.postMessage", post(handle_chat_post_message))
        .route("/api/users.list", post(handle_users_list))
        .route("/api/error", post(handle_error))
        .route("/api/rate_limit", post(handle_rate_limit))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://127.0.0.1:{}/api/", addr.port());

    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    (url, handle)
}

#[tokio::test]
async fn test_async_client_api_test() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let response = client.api_test(None).await.unwrap();
    assert_eq!(response["ok"], json!(true));
}

#[tokio::test]
async fn test_async_client_api_test_with_params() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let params = json!({
        "foo": "bar",
        "test": true
    });

    let response = client.api_test(Some(params)).await.unwrap();
    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["foo"], json!("bar"));
    assert_eq!(response["test"], json!("1")); // Bool converted to "1"
}

#[tokio::test]
async fn test_async_client_auth_test() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let response = client.auth_test(None).await.unwrap();
    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["team_id"], json!("T123456"));
    assert_eq!(response["user_id"], json!("U123456"));
}

#[tokio::test]
async fn test_async_client_chat_post_message() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let params = json!({
        "channel": "C123456",
        "text": "Hello, World!"
    });

    let response = client.chat_post_message(Some(params)).await.unwrap();
    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["channel"], json!("C123456"));
    assert_eq!(response["message"]["text"], json!("Hello, World!"));
}

#[tokio::test]
async fn test_async_client_pagination() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let mut all_users = Vec::new();
    let params = json!({"limit": 100});
    let mut response = client.users_list(Some(params)).await.unwrap();

    // Collect first page
    if let Some(members) = response["members"].as_array() {
        all_users.extend(members.clone());
    }

    // Pagination loop
    while response.has_next_cursor() {
        response = response.next().await.unwrap();
        if let Some(members) = response["members"].as_array() {
            all_users.extend(members.clone());
        }
    }

    // Should have collected all 250 users
    assert_eq!(all_users.len(), 250);
}

#[tokio::test]
async fn test_async_client_error_handling() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let result = client.api_call("error", None).await;
    assert!(result.is_err());

    if let Err(e) = result {
        let error_str = e.to_string();
        assert!(error_str.contains("invalid_auth") || error_str.contains("API error"));
    }
}

#[tokio::test]
async fn test_async_client_response_indexing() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let response = client.api_test(None).await.unwrap();

    // Test Index trait
    assert_eq!(response["ok"], json!(true));
}

#[tokio::test]
async fn test_async_client_response_get() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let response = client.auth_test(None).await.unwrap();

    // Test get method
    assert!(response.get("ok").is_some());
    assert_eq!(response.get("ok").unwrap(), &json!(true));
    assert!(response.get("nonexistent").is_none());
}

#[tokio::test]
async fn test_async_client_response_contains_key() {
    let (base_url, _handle) = start_mock_server().await;

    let client = AsyncWebClient::builder()
        .token("xoxb-test-token")
        .base_url(&base_url)
        .build();

    let response = client.auth_test(None).await.unwrap();

    assert!(response.contains_key("ok"));
    assert!(response.contains_key("team_id"));
    assert!(!response.contains_key("nonexistent"));
}

// Sync client tests need to be run in a separate runtime
#[test]
fn test_sync_client_api_test() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (base_url, _handle) = start_mock_server().await;

        // Need to spawn blocking task to avoid runtime issues
        tokio::task::spawn_blocking(move || {
            let client = WebClient::builder()
                .token("xoxb-test-token")
                .base_url(&base_url)
                .build();

            let response = client.api_test(None).unwrap();
            assert_eq!(response["ok"], json!(true));
        })
        .await
        .unwrap();
    });
}

#[test]
fn test_sync_client_chat_post_message() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let (base_url, _handle) = start_mock_server().await;

        tokio::task::spawn_blocking(move || {
            let client = WebClient::builder()
                .token("xoxb-test-token")
                .base_url(&base_url)
                .build();

            let params = json!({
                "channel": "C123456",
                "text": "Hello from sync!"
            });

            let response = client.chat_post_message(Some(params)).unwrap();
            assert_eq!(response["ok"], json!(true));
            assert_eq!(response["message"]["text"], json!("Hello from sync!"));
        })
        .await
        .unwrap();
    });
}

#[tokio::test]
async fn test_client_builder() {
    let _client = AsyncWebClient::builder()
        .token("xoxb-test")
        .base_url("https://test.slack.com/api/")
        .timeout(std::time::Duration::from_secs(60))
        .header("X-Custom", "value")
        .max_retries(5)
        .build();
}

#[test]
fn test_slack_response_creation() {
    use reqwest::header::HeaderMap;
    use slack_rs::web::SlackResponse;

    let data = json!({"ok": true, "message": "Success"});
    let response = SlackResponse::new(
        "POST".to_string(),
        "https://slack.com/api/api.test".to_string(),
        data.clone(),
        HeaderMap::new(),
        200,
    );

    assert_eq!(response["ok"], json!(true));
    assert_eq!(response["message"], json!("Success"));
}

#[test]
fn test_slack_response_validation_success() {
    use reqwest::header::HeaderMap;
    use slack_rs::web::SlackResponse;

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
fn test_slack_response_validation_failure() {
    use reqwest::header::HeaderMap;
    use slack_rs::web::SlackResponse;

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
}

#[test]
fn test_slack_response_has_next_cursor() {
    use reqwest::header::HeaderMap;
    use slack_rs::web::SlackResponse;

    // With cursor
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

    // Without cursor
    let data2 = json!({"ok": true});
    let response2 = SlackResponse::new(
        "POST".to_string(),
        "https://slack.com/api/api.test".to_string(),
        data2,
        HeaderMap::new(),
        200,
    );
    assert!(!response2.has_next_cursor());
}

#[test]
fn test_url_building() {
    use slack_rs::web::internal_utils::get_url;

    assert_eq!(
        get_url("https://slack.com/api/", "chat.postMessage"),
        "https://slack.com/api/chat.postMessage"
    );

    assert_eq!(
        get_url("https://slack.com/api", "api.test"),
        "https://slack.com/api/api.test"
    );
}

#[test]
fn test_user_agent() {
    use slack_rs::web::internal_utils::get_user_agent;

    let ua = get_user_agent(None, None);
    assert!(ua.contains("rust-slack-sdk/"));
    assert!(ua.contains("rust/"));

    let ua_with_prefix = get_user_agent(Some("MyApp/1.0"), None);
    assert!(ua_with_prefix.starts_with("MyApp/1.0 "));
}

#[test]
fn test_convert_bool_to_0_or_1() {
    use slack_rs::web::internal_utils::convert_bool_to_0_or_1;

    let input = json!({"active": true, "archived": false, "name": "test"});
    let output = convert_bool_to_0_or_1(&input);

    assert_eq!(output["active"], json!("1"));
    assert_eq!(output["archived"], json!("0"));
    assert_eq!(output["name"], json!("test"));
}

#[test]
fn test_remove_none_values() {
    use slack_rs::web::internal_utils::remove_none_values;

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
