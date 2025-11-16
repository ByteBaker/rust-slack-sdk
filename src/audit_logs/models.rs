//! Audit Logs API data models
//!
//! This module contains the data structures representing audit log entries
//! and related metadata from Slack's Enterprise Grid Audit Logs API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a user in audit logs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
}

/// Actor who performed the action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Actor {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub actor_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
}

/// Location information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub location_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// App information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct App {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_distributed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_directory_approved: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_workflow_app: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// Context of the audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ua: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<App>,
}

/// Details of the audit log entry
///
/// This uses a HashMap to capture any dynamic fields that may be present
/// in the details depending on the action type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Details {
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Channel information in an entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Channel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_org_shared: Option<bool>,
}

/// Entity that was affected by the action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<User>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise: Option<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<App>,
}

/// A single audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_create: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor: Option<Actor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<Entity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Context>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Details>,
}

/// Response metadata with pagination cursor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from the logs endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<LogEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_metadata: Option<ResponseMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Unified audit logs API response
#[derive(Debug, Clone)]
pub struct AuditLogsResponse {
    pub url: String,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub raw_body: Option<String>,
    pub body: Option<serde_json::Value>,
}

impl AuditLogsResponse {
    /// Create a new AuditLogsResponse
    pub fn new(
        url: String,
        status_code: u16,
        headers: HashMap<String, String>,
        raw_body: Option<String>,
    ) -> Self {
        let body = raw_body.as_ref().and_then(|b| {
            if b.starts_with('{') {
                serde_json::from_str(b).ok()
            } else {
                None
            }
        });

        Self {
            url,
            status_code,
            headers,
            raw_body,
            body,
        }
    }

    /// Get the typed logs response
    pub fn typed_body(&self) -> Option<LogsResponse> {
        self.body
            .as_ref()
            .and_then(|b| serde_json::from_value(b.clone()).ok())
    }

    /// Check if the response was successful
    pub fn is_ok(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            id: Some("xxx-yyy-zzz-111".to_string()),
            date_create: Some(1521214343),
            action: Some("user_login".to_string()),
            actor: Some(Actor {
                actor_type: Some("user".to_string()),
                user: Some(User {
                    id: Some("W123456".to_string()),
                    name: Some("Alice".to_string()),
                    email: Some("alice@example.com".to_string()),
                    team: Some("T123456".to_string()),
                }),
            }),
            entity: None,
            context: None,
            details: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("xxx-yyy-zzz-111"));
        assert!(json.contains("user_login"));
    }

    #[test]
    fn test_logs_response_deserialization() {
        let json = r#"{
            "entries": [
                {
                    "id": "xxx-yyy-zzz-111",
                    "date_create": 1521214343,
                    "action": "user_login",
                    "actor": {
                        "type": "user",
                        "user": {
                            "id": "W123456",
                            "name": "Alice",
                            "email": "alice@example.com"
                        }
                    }
                }
            ],
            "response_metadata": {
                "next_cursor": "dXNlcjpVMEc5V0ZYTlo="
            }
        }"#;

        let response: LogsResponse = serde_json::from_str(json).unwrap();
        assert!(response.entries.is_some());
        let entries = response.entries.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, Some("xxx-yyy-zzz-111".to_string()));
        assert_eq!(entries[0].action, Some("user_login".to_string()));
    }

    #[test]
    fn test_audit_logs_response_creation() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let body = r#"{"ok": true, "entries": []}"#;
        let response = AuditLogsResponse::new(
            "https://api.slack.com/audit/v1/logs".to_string(),
            200,
            headers,
            Some(body.to_string()),
        );

        assert_eq!(response.status_code, 200);
        assert!(response.is_ok());
        assert!(response.body.is_some());
    }

    #[test]
    fn test_response_metadata_with_cursor() {
        let metadata = ResponseMetadata {
            next_cursor: Some("cursor123".to_string()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("cursor123"));
    }

    #[test]
    fn test_context_serialization() {
        let context = Context {
            location: Some(Location {
                location_type: Some("workspace".to_string()),
                id: Some("T123456".to_string()),
                name: Some("My Workspace".to_string()),
                domain: Some("my-workspace".to_string()),
            }),
            ua: Some("Mozilla/5.0".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            session_id: Some("session123".to_string()),
            app: None,
        };

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("192.168.1.1"));
        assert!(json.contains("workspace"));
    }
}
