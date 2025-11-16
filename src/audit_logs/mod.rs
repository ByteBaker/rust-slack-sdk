//! Audit Logs API
//!
//! The Audit Logs API provides access to your Enterprise Grid organization's
//! audit logs. You can monitor what's happening in your organization for
//! security and compliance purposes.
//!
//! # Authentication
//!
//! The Audit Logs API requires an admin user token (starting with `xoxp-`)
//! with the `admin` scope.
//!
//! # Example (Sync)
//!
//! ```no_run
//! use slack_rs::audit_logs::AuditLogsClient;
//!
//! let client = AuditLogsClient::new("xoxp-your-admin-token");
//!
//! // Fetch recent login events
//! match client.logs(Some(10), Some("user_login"), None, None, None, None, None) {
//!     Ok(response) => {
//!         if let Some(typed) = response.typed_body() {
//!             if let Some(entries) = typed.entries {
//!                 println!("Found {} log entries", entries.len());
//!                 for entry in entries {
//!                     println!("  Action: {:?}", entry.action);
//!                 }
//!             }
//!         }
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```
//!
//! # Example (Async)
//!
//! ```no_run
//! use slack_rs::audit_logs::AsyncAuditLogsClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = AsyncAuditLogsClient::new("xoxp-your-admin-token");
//!
//!     match client.logs(Some(10), Some("user_login"), None, None, None, None, None).await {
//!         Ok(response) => {
//!             if let Some(typed) = response.typed_body() {
//!                 if let Some(entries) = typed.entries {
//!                     println!("Found {} log entries", entries.len());
//!                 }
//!             }
//!         }
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```
//!
//! # Pagination
//!
//! The Audit Logs API uses cursor-based pagination. Use the `next_cursor`
//! from the response metadata to fetch the next page:
//!
//! ```no_run
//! use slack_rs::audit_logs::AuditLogsClient;
//!
//! let client = AuditLogsClient::new("xoxp-your-admin-token");
//!
//! let mut cursor: Option<String> = None;
//! loop {
//!     let response = client.logs(
//!         Some(100),
//!         Some("user_login"),
//!         None,
//!         None,
//!         None,
//!         None,
//!         cursor.as_deref(),
//!     ).unwrap();
//!
//!     if let Some(typed) = response.typed_body() {
//!         // Process entries...
//!
//!         // Check for next page
//!         if let Some(metadata) = typed.response_metadata {
//!             cursor = metadata.next_cursor;
//!             if cursor.is_none() {
//!                 break; // No more pages
//!             }
//!         } else {
//!             break;
//!         }
//!     } else {
//!         break;
//!     }
//! }
//! ```

pub mod client;
pub mod models;

// Re-export main types
pub use client::{AsyncAuditLogsClient, AuditLogsClient};
pub use models::{
    Actor, App, AuditLogsResponse, Channel, Context, Details, Entity, Location, LogEntry,
    LogsResponse, ResponseMetadata, User,
};
