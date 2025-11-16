//! Slack Web API client module.
//!
//! This module provides both synchronous and asynchronous clients for interacting
//! with the Slack Web API. It includes support for all 292+ API methods, automatic
//! retry handling, pagination, and proper error handling.
//!
//! # Examples
//!
//! ## Async Client
//!
//! ```no_run
//! use slack_rs::web::AsyncWebClient;
//!
//! #[tokio::main]
//! async fn main() -> slack_rs::error::Result<()> {
//!     let client = AsyncWebClient::new("xoxb-your-token");
//!
//!     // Test API connection
//!     let response = client.api_test(None).await?;
//!     println!("API is working: {}", response["ok"]);
//!
//!     // Post a message
//!     let params = serde_json::json!({
//!         "channel": "C123456",
//!         "text": "Hello from Rust!"
//!     });
//!     let response = client.chat_post_message(Some(params)).await?;
//!     println!("Message posted with ts: {}", response["ts"]);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Sync Client
//!
//! ```no_run
//! use slack_rs::web::WebClient;
//!
//! fn main() -> slack_rs::error::Result<()> {
//!     let client = WebClient::new("xoxb-your-token");
//!
//!     let response = client.api_test(None)?;
//!     println!("API is working: {}", response["ok"]);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Pagination
//!
//! ```no_run
//! use slack_rs::web::AsyncWebClient;
//!
//! #[tokio::main]
//! async fn main() -> slack_rs::error::Result<()> {
//!     let client = AsyncWebClient::new("xoxb-your-token");
//!
//!     let mut all_users = Vec::new();
//!     let params = serde_json::json!({"limit": 100});
//!     let mut response = client.users_list(Some(params)).await?;
//!
//!     loop {
//!         if let Some(members) = response["members"].as_array() {
//!             all_users.extend(members.clone());
//!         }
//!         if !response.has_next_cursor() {
//!             break;
//!         }
//!         response = response.next().await?;
//!     }
//!
//!     println!("Total users: {}", all_users.len());
//!     Ok(())
//! }
//! ```

pub mod async_client;
pub mod client;
pub mod internal_utils;
pub mod response;

// Re-export main types
pub use async_client::{AsyncWebClient, AsyncWebClientBuilder};
pub use client::{WebClient, WebClientBuilder};
pub use response::SlackResponse;
