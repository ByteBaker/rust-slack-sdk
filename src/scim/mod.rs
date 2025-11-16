//! SCIM API for user and group provisioning.
//!
//! SCIM (System for Cross-domain Identity Management) is a standard protocol
//! for automating the exchange of user identity information. Slack's SCIM API
//! enables enterprise customers to manage users and groups programmatically.
//!
//! # Overview
//!
//! The SCIM API allows you to:
//! - Create, read, update, and delete users
//! - Create, read, update, and delete groups
//! - Search for users and groups with filters
//! - Manage group memberships
//! - Perform partial updates with PATCH operations
//!
//! # Authentication
//!
//! SCIM API requires a user token with `admin` scope. The token should start
//! with `xoxp-`.
//!
//! # Examples
//!
//! ## Creating and managing users
//!
//! ```no_run
//! use slack_rs::scim::{AsyncScimClient, models::{User, UserEmail}};
//! use slack_rs::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = AsyncScimClient::new("xoxp-your-admin-token");
//!
//!     // Create a new user
//!     let user = User::new()
//!         .with_user_name("john.doe@example.com")
//!         .with_display_name("John Doe")
//!         .with_active(true)
//!         .with_emails(vec![
//!             UserEmail {
//!                 value: "john.doe@example.com".to_string(),
//!                 email_type: Some("work".to_string()),
//!                 primary: Some(true),
//!             }
//!         ]);
//!
//!     let created_user = client.create_user(&user).await?;
//!     println!("Created user: {:?}", created_user.id);
//!
//!     // Get the user
//!     if let Some(user_id) = created_user.id {
//!         let fetched_user = client.get_user(&user_id).await?;
//!         println!("User: {:?}", fetched_user);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Searching for users
//!
//! ```no_run
//! use slack_rs::scim::AsyncScimClient;
//! use slack_rs::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = AsyncScimClient::new("xoxp-your-admin-token");
//!
//!     // Search for active users
//!     let response = client.search_users(
//!         Some("active eq true"),
//!         Some(1),  // Start at index 1
//!         Some(10), // Get 10 results
//!     ).await?;
//!
//!     println!("Total users: {:?}", response.total_results);
//!     if let Some(users) = response.resources {
//!         for user in users {
//!             println!("User: {:?}", user.user_name);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Managing groups
//!
//! ```no_run
//! use slack_rs::scim::{AsyncScimClient, models::{Group, GroupMember}};
//! use slack_rs::error::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = AsyncScimClient::new("xoxp-your-admin-token");
//!
//!     // Create a new group
//!     let group = Group::new()
//!         .with_display_name("Engineering Team")
//!         .with_members(vec![
//!             GroupMember {
//!                 value: "U123456".to_string(),
//!                 display: Some("John Doe".to_string()),
//!             }
//!         ]);
//!
//!     let created_group = client.create_group(&group).await?;
//!     println!("Created group: {:?}", created_group.id);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Partial updates with PATCH
//!
//! ```no_run
//! use slack_rs::scim::{AsyncScimClient, models::{PatchRequest, PatchOperation}};
//! use slack_rs::error::Result;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let client = AsyncScimClient::new("xoxp-your-admin-token");
//!
//!     // Deactivate a user
//!     let patch = PatchRequest::new(vec![
//!         PatchOperation {
//!             op: "replace".to_string(),
//!             path: Some("active".to_string()),
//!             value: Some(json!(false)),
//!         }
//!     ]);
//!
//!     let updated_user = client.patch_user("U123456", &patch).await?;
//!     println!("User active: {:?}", updated_user.active);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Using the synchronous client
//!
//! ```no_run
//! use slack_rs::scim::{ScimClient, models::User};
//! use slack_rs::error::Result;
//!
//! fn main() -> Result<()> {
//!     let client = ScimClient::new("xoxp-your-admin-token");
//!
//!     // Get a user (blocking call)
//!     let user = client.get_user("U123456")?;
//!     println!("User: {:?}", user.user_name);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod models;

// Re-export commonly used types
pub use client::{AsyncScimClient, ScimClient, SCIM_BASE_URL};
pub use models::{
    Group, GroupMember, GroupMeta, PatchOperation, PatchRequest, ScimError, ScimResponse, User,
    UserAddress, UserEmail, UserGroup, UserMeta, UserName, UserPhoneNumber, UserPhoto, UserRole,
};
