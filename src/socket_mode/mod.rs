//! Socket Mode client for real-time Slack events.
//!
//! Socket Mode allows your app to use the Events API and interactive components
//! without exposing a public HTTP endpoint. Instead, your app connects to Slack
//! via WebSocket.
//!
//! # Overview
//!
//! Socket Mode is ideal for development, internal tools, or apps running behind
//! firewalls where exposing a public endpoint is not practical.
//!
//! # Features
//!
//! - **WebSocket-based**: No public HTTP endpoint required
//! - **Automatic reconnection**: Handles connection drops gracefully
//! - **Event handlers**: Register callbacks for different event types
//! - **Auto-acknowledgment**: Automatically acknowledges messages (configurable)
//!
//! # Requirements
//!
//! To use Socket Mode, you need:
//! - An app-level token (starts with `xapp-`)
//! - Socket Mode enabled in your Slack app settings
//!
//! # Examples
//!
//! ## Basic usage
//!
//! ```no_run
//! use slack_rs::socket_mode::SocketModeClient;
//! use slack_rs::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a Socket Mode client with your app-level token
//!     let client = SocketModeClient::new("xapp-1-A123-456-abc");
//!
//!     // Register an event handler
//!     client.on_events_api(|request| {
//!         println!("Received event: {:?}", request.payload);
//!         Ok(())
//!     }).await;
//!
//!     // Connect and start processing events
//!     client.connect().await?;
//!     client.start().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Handling different event types
//!
//! ```no_run
//! use slack_rs::socket_mode::SocketModeClient;
//! use slack_rs::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = SocketModeClient::new("xapp-1-A123-456-abc");
//!
//!     // Handle Events API events
//!     client.on_events_api(|request| {
//!         println!("Event: {:?}", request.payload);
//!         Ok(())
//!     }).await;
//!
//!     // Handle slash commands
//!     client.on_slash_commands(|request| {
//!         println!("Slash command: {:?}", request.payload);
//!         Ok(())
//!     }).await;
//!
//!     // Handle interactive components (buttons, menus, etc.)
//!     client.on_interactive(|request| {
//!         println!("Interactive: {:?}", request.payload);
//!         Ok(())
//!     }).await;
//!
//!     client.connect().await?;
//!     client.start().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Manual acknowledgment
//!
//! ```no_run
//! use slack_rs::socket_mode::SocketModeClient;
//! use slack_rs::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = SocketModeClient::new("xapp-1-A123-456-abc");
//!
//!     // Disable auto-acknowledgment
//!     client.set_auto_acknowledge(false).await;
//!
//!     let client_clone = client.clone();
//!     client.on_events_api(move |request| {
//!         // Process the event
//!         println!("Processing event: {:?}", request.payload);
//!
//!         // Manually acknowledge the event
//!         let envelope_id = request.envelope_id.clone();
//!         tokio::spawn(async move {
//!             client_clone.acknowledge(envelope_id).await.ok();
//!         });
//!
//!         Ok(())
//!     }).await;
//!
//!     client.connect().await?;
//!     client.start().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod connection;
pub mod types;

// Re-export commonly used types
pub use client::{EventHandler, SocketModeClient};
pub use types::{SocketModeMessageType, SocketModeRequest, SocketModeResponse};
