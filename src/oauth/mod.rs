//! OAuth 2.0 support for Slack apps
//!
//! This module provides comprehensive OAuth 2.0 functionality for building Slack apps,
//! including installation management, state handling, and token rotation.
//!
//! # Overview
//!
//! The OAuth module includes:
//!
//! - **Models**: Types for installations, bots, and OAuth responses
//! - **Authorization URL Generation**: Create OAuth authorization URLs
//! - **Installation Storage**: Persist installation data with multiple backends
//! - **State Management**: CSRF protection for OAuth flows
//! - **Token Rotation**: Automatic token refresh when rotation is enabled
//!
//! # Quick Start
//!
//! ```no_run
//! use slack_rs::oauth::{
//!     AuthorizeUrlGenerator,
//!     installation_store::{CacheInstallationStore, InstallationStore},
//!     state_store::{CacheOAuthStateStore, OAuthStateStore},
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create storage backends
//!     let installation_store = CacheInstallationStore::new();
//!     let state_store = CacheOAuthStateStore::new();
//!
//!     // Generate authorization URL
//!     let generator = AuthorizeUrlGenerator::new("client_id_123")
//!         .scopes(vec!["chat:write".to_string(), "channels:read".to_string()])
//!         .redirect_uri("https://example.com/oauth/callback");
//!
//!     let state = state_store.issue().await?;
//!     let auth_url = generator.generate(&state, None);
//!
//!     println!("Authorize at: {}", auth_url);
//!
//!     // After OAuth callback, validate state and save installation
//!     let is_valid = state_store.consume(&state).await?;
//!     if !is_valid {
//!         panic!("Invalid state!");
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Storage Backends
//!
//! Multiple storage backends are available:
//!
//! - **CacheInstallationStore**: In-memory storage (development/testing)
//! - **FileInstallationStore**: File-based storage
//! - **SqliteInstallationStore**: SQLite database (requires `sqlite` feature)
//!
//! # Token Rotation
//!
//! When token rotation is enabled, use the `TokenRotator` to automatically
//! refresh tokens before they expire:
//!
//! ```no_run
//! use slack_rs::oauth::token_rotation::TokenRotator;
//! use slack_rs::oauth::installation_store::CacheInstallationStore;
//! use slack_rs::web::AsyncWebClient;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AsyncWebClient::new("xoxb-token");
//!     let store = Arc::new(CacheInstallationStore::new());
//!     let rotator = TokenRotator::new(
//!         client,
//!         store,
//!         "client_id",
//!         "client_secret"
//!     );
//!
//!     // Check and rotate tokens automatically
//!     // let installation = rotator.check_and_rotate_bot_token(installation, None).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod authorize_url_generator;
pub mod installation_store;
pub mod models;
pub mod state_store;
pub mod token_rotation;

// Re-export commonly used types
pub use authorize_url_generator::{AuthorizeUrlGenerator, OpenIDConnectAuthorizeUrlGenerator};
pub use installation_store::InstallationStore;
pub use models::{Bot, Installation, OAuthV2Response};
pub use state_store::OAuthStateStore;
pub use token_rotation::TokenRotator;
