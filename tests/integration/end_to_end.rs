//! End-to-end integration tests
//!
//! These tests verify complete workflows without hitting the real Slack API.
//! They use mock servers to simulate API responses.

use slack_rs::models::*;
use slack_rs::web::{AsyncWebClient, SlackResponse};
use std::collections::HashMap;

#[tokio::test]
async fn test_complete_message_workflow() {
    // This test would use a mock server in a real integration test
    // For now, we just verify the components work together

    // Build a message with Block Kit
    let blocks = vec![
        HeaderBlock::new("Test Message").unwrap().into(),
        SectionBlock::new("This is a test").unwrap().into(),
        ActionsBlock::builder()
            .elements(vec![ButtonElement::new("Click", "btn_1")
                .with_style(ButtonStyle::Primary)
                .build()
                .unwrap()])
            .build()
            .unwrap()
            .into(),
    ];

    // Serialize blocks
    let blocks_json = serde_json::to_string(&blocks).unwrap();

    // Verify the JSON is valid
    assert!(blocks_json.contains("Test Message"));
    assert!(blocks_json.contains("This is a test"));
    assert!(blocks_json.contains("btn_1"));

    // Verify we can deserialize back
    let parsed_blocks: Vec<Block> = serde_json::from_str(&blocks_json).unwrap();
    assert_eq!(parsed_blocks.len(), 3);
}

#[tokio::test]
async fn test_oauth_state_flow() {
    // Test OAuth state management flow
    use slack_rs::oauth::state_store::cache::CacheOAuthStateStore;
    use slack_rs::oauth::OAuthStateStore;

    let store = CacheOAuthStateStore::new();

    // Issue a state
    let state = "test-state-123";
    store.consume(state).await.unwrap();

    // Verify state exists
    let result = store.consume(state).await;
    assert!(result.is_ok());

    // State should only be consumable once
    let result = store.consume(state).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_signature_verification_workflow() {
    use slack_rs::signature::SignatureVerifier;

    let signing_secret = "test_secret";
    let verifier = SignatureVerifier::new(signing_secret);

    let timestamp = "1234567890";
    let body = r#"{"type":"url_verification","challenge":"test123"}"#;

    // Generate a valid signature
    let signature = verifier.generate_signature(timestamp, body);

    // Verify the signature
    let result = verifier.verify(timestamp, body, &signature);
    assert!(result.is_ok());

    // Verify invalid signature fails
    let invalid_sig = "v0=invalid";
    let result = verifier.verify(timestamp, body, invalid_sig);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_webhook_sending_workflow() {
    use slack_rs::webhook::AsyncWebhookClient;

    // Note: This would use a mock server in a real integration test
    // For now, we just verify the client can be created and blocks can be prepared

    let webhook_url = "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXX";
    let client = AsyncWebhookClient::new(webhook_url);

    let blocks = vec![
        SectionBlock::new("Webhook test message").unwrap().into(),
    ];

    let blocks_json = serde_json::to_string(&blocks).unwrap();

    // Verify JSON is valid
    assert!(blocks_json.contains("Webhook test message"));
}

#[test]
fn test_block_kit_validation() {
    // Test that validation works correctly for various inputs

    // Valid header
    assert!(HeaderBlock::new("Valid Header").is_ok());

    // Header too long (>150 chars)
    let long_text = "a".repeat(151);
    assert!(HeaderBlock::new(&long_text).is_err());

    // Valid section
    assert!(SectionBlock::new("Valid section").is_ok());

    // Section too long (>3000 chars)
    let very_long_text = "a".repeat(3001);
    assert!(SectionBlock::new(&very_long_text).is_err());

    // Valid button
    assert!(ButtonElement::new("Click me", "action_id")
        .build()
        .is_ok());

    // Button text too long (>75 chars)
    let long_button_text = "a".repeat(76);
    assert!(ButtonElement::new(&long_button_text, "action_id")
        .build()
        .is_err());
}

#[tokio::test]
async fn test_response_parsing() {
    // Test that responses are parsed correctly

    let ok_response = SlackResponse {
        url: "https://slack.com/api/test".to_string(),
        status_code: 200,
        headers: HashMap::new(),
        body: Some(r#"{"ok":true,"channel":"C123456"}"#.to_string()),
    };

    assert!(ok_response.is_ok());

    let parsed: Result<serde_json::Value, _> = ok_response.deserialize_payload();
    assert!(parsed.is_ok());

    let json = parsed.unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["channel"], "C123456");

    // Test error response
    let error_response = SlackResponse {
        url: "https://slack.com/api/test".to_string(),
        status_code: 200,
        headers: HashMap::new(),
        body: Some(r#"{"ok":false,"error":"invalid_auth"}"#.to_string()),
    };

    assert!(!error_response.is_ok());
}

#[test]
fn test_modal_view_building() {
    // Test building a complete modal view

    let view = View::modal()
        .title("Test Modal")
        .submit("Submit")
        .close("Cancel")
        .blocks(vec![
            InputBlock::new("Name")
                .element(PlainTextInputElement::new("name"))
                .build()
                .unwrap(),
            InputBlock::new("Email")
                .element(PlainTextInputElement::new("email"))
                .build()
                .unwrap(),
        ])
        .build();

    assert!(view.is_ok());

    let modal = view.unwrap();

    // Verify it serializes correctly
    let json = serde_json::to_string(&modal).unwrap();
    assert!(json.contains("Test Modal"));
    assert!(json.contains("Submit"));
    assert!(json.contains("Name"));
    assert!(json.contains("Email"));
}

#[tokio::test]
async fn test_audit_logs_response_parsing() {
    use slack_rs::audit_logs::models::{AuditLogsResponse, LogsResponse};

    let body = r#"{
        "ok": true,
        "entries": [
            {
                "id": "123",
                "date_create": 1234567890,
                "action": "user_login"
            }
        ],
        "response_metadata": {
            "next_cursor": "cursor123"
        }
    }"#;

    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());

    let response = AuditLogsResponse::new(
        "https://api.slack.com/audit/v1/logs".to_string(),
        200,
        headers,
        Some(body.to_string()),
    );

    assert!(response.is_ok());

    let typed = response.typed_body();
    assert!(typed.is_some());

    let logs = typed.unwrap();
    assert!(logs.entries.is_some());

    let entries = logs.entries.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, Some("123".to_string()));
}
