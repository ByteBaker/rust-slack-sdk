//! SCIM API clients for user and group provisioning.
//!
//! This module provides both synchronous and asynchronous clients for
//! interacting with Slack's SCIM API for user and group management.

use crate::error::{Result, SlackError};
use crate::scim::models::{Group, PatchRequest, ScimResponse, User};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use std::time::Duration;

/// Base URL for the Slack SCIM API.
pub const SCIM_BASE_URL: &str = "https://api.slack.com/scim/v1/";

/// Asynchronous SCIM API client.
///
/// Provides async methods for managing users and groups via the SCIM 2.0 protocol.
///
/// # Examples
///
/// ```no_run
/// use slack_rs::scim::AsyncScimClient;
/// use slack_rs::scim::models::User;
/// use slack_rs::error::Result;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let client = AsyncScimClient::new("xoxp-your-token");
///
///     // Get a user
///     let user = client.get_user("U123456").await?;
///     println!("User: {:?}", user);
///
///     // List all users
///     let response = client.search_users(None, None, None).await?;
///     println!("Total users: {:?}", response.total_results);
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct AsyncScimClient {
    /// Bearer token for authentication
    token: String,

    /// Base URL for the SCIM API
    base_url: String,

    /// HTTP client
    client: reqwest::Client,
}

impl AsyncScimClient {
    /// Creates a new async SCIM client with the given token.
    pub fn new(token: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token: token.into(),
            base_url: SCIM_BASE_URL.to_string(),
            client,
        }
    }

    /// Creates a client with a custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Builds headers for SCIM requests.
    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        let auth_value = HeaderValue::from_str(&format!("Bearer {}", self.token))
            .map_err(|e| SlackError::InvalidToken(format!("Invalid token: {}", e)))?;

        headers.insert(AUTHORIZATION, auth_value);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Ok(headers)
    }

    /// Gets a user by ID.
    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let url = format!("{}Users/{}", self.base_url, user_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to get user: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let user: User = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse user response: {}", e),
        })?;

        Ok(user)
    }

    /// Creates a new user.
    pub async fn create_user(&self, user: &User) -> Result<User> {
        let url = format!("{}Users", self.base_url);
        let headers = self.build_headers()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(user)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to create user: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let created_user: User = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse user response: {}", e),
        })?;

        Ok(created_user)
    }

    /// Updates a user.
    pub async fn update_user(&self, user_id: &str, user: &User) -> Result<User> {
        let url = format!("{}Users/{}", self.base_url, user_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(user)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to update user: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let updated_user: User = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse user response: {}", e),
        })?;

        Ok(updated_user)
    }

    /// Patches a user with partial updates.
    pub async fn patch_user(&self, user_id: &str, patch: &PatchRequest) -> Result<User> {
        let url = format!("{}Users/{}", self.base_url, user_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .patch(&url)
            .headers(headers)
            .json(patch)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to patch user: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let patched_user: User = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse user response: {}", e),
        })?;

        Ok(patched_user)
    }

    /// Deletes a user.
    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let url = format!("{}Users/{}", self.base_url, user_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to delete user: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        Ok(())
    }

    /// Searches for users with optional filters and pagination.
    pub async fn search_users(
        &self,
        filter: Option<&str>,
        start_index: Option<u32>,
        count: Option<u32>,
    ) -> Result<ScimResponse<User>> {
        let mut url = format!("{}Users", self.base_url);
        let mut params = Vec::new();

        if let Some(f) = filter {
            params.push(format!("filter={}", urlencoding::encode(f)));
        }
        if let Some(si) = start_index {
            params.push(format!("startIndex={}", si));
        }
        if let Some(c) = count {
            params.push(format!("count={}", c));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let headers = self.build_headers()?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to search users: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let scim_response: ScimResponse<User> =
            response.json().await.map_err(|e| SlackError::HttpError {
                message: format!("Failed to parse search response: {}", e),
            })?;

        Ok(scim_response)
    }

    /// Gets a group by ID.
    pub async fn get_group(&self, group_id: &str) -> Result<Group> {
        let url = format!("{}Groups/{}", self.base_url, group_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to get group: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let group: Group = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse group response: {}", e),
        })?;

        Ok(group)
    }

    /// Creates a new group.
    pub async fn create_group(&self, group: &Group) -> Result<Group> {
        let url = format!("{}Groups", self.base_url);
        let headers = self.build_headers()?;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(group)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to create group: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let created_group: Group = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse group response: {}", e),
        })?;

        Ok(created_group)
    }

    /// Updates a group.
    pub async fn update_group(&self, group_id: &str, group: &Group) -> Result<Group> {
        let url = format!("{}Groups/{}", self.base_url, group_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .put(&url)
            .headers(headers)
            .json(group)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to update group: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let updated_group: Group = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse group response: {}", e),
        })?;

        Ok(updated_group)
    }

    /// Patches a group with partial updates.
    pub async fn patch_group(&self, group_id: &str, patch: &PatchRequest) -> Result<Group> {
        let url = format!("{}Groups/{}", self.base_url, group_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .patch(&url)
            .headers(headers)
            .json(patch)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to patch group: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let patched_group: Group = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse group response: {}", e),
        })?;

        Ok(patched_group)
    }

    /// Deletes a group.
    pub async fn delete_group(&self, group_id: &str) -> Result<()> {
        let url = format!("{}Groups/{}", self.base_url, group_id);
        let headers = self.build_headers()?;

        let response = self
            .client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to delete group: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        Ok(())
    }

    /// Searches for groups with optional filters and pagination.
    pub async fn search_groups(
        &self,
        filter: Option<&str>,
        start_index: Option<u32>,
        count: Option<u32>,
    ) -> Result<ScimResponse<Group>> {
        let mut url = format!("{}Groups", self.base_url);
        let mut params = Vec::new();

        if let Some(f) = filter {
            params.push(format!("filter={}", urlencoding::encode(f)));
        }
        if let Some(si) = start_index {
            params.push(format!("startIndex={}", si));
        }
        if let Some(c) = count {
            params.push(format!("count={}", c));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        let headers = self.build_headers()?;

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("Failed to search groups: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(SlackError::ApiError {
                message: format!("SCIM API error ({}): {}", status, body),
                response: serde_json::from_str(&body).unwrap_or(Value::Null),
            });
        }

        let scim_response: ScimResponse<Group> =
            response.json().await.map_err(|e| SlackError::HttpError {
                message: format!("Failed to parse search response: {}", e),
            })?;

        Ok(scim_response)
    }
}

/// Synchronous SCIM API client.
///
/// Provides blocking methods for managing users and groups via the SCIM 2.0 protocol.
///
/// # Examples
///
/// ```no_run
/// use slack_rs::scim::ScimClient;
/// use slack_rs::scim::models::User;
/// use slack_rs::error::Result;
///
/// fn main() -> Result<()> {
///     let client = ScimClient::new("xoxp-your-token");
///
///     // Get a user
///     let user = client.get_user("U123456")?;
///     println!("User: {:?}", user);
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ScimClient {
    /// Async client (used internally)
    inner: AsyncScimClient,

    /// Tokio runtime for blocking operations
    runtime: std::sync::Arc<tokio::runtime::Runtime>,
}

impl ScimClient {
    /// Creates a new synchronous SCIM client.
    pub fn new(token: impl Into<String>) -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create runtime");

        Self {
            inner: AsyncScimClient::new(token),
            runtime: std::sync::Arc::new(runtime),
        }
    }

    /// Creates a client with a custom base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.inner = self.inner.with_base_url(base_url);
        self
    }

    /// Gets a user by ID.
    pub fn get_user(&self, user_id: &str) -> Result<User> {
        self.runtime.block_on(self.inner.get_user(user_id))
    }

    /// Creates a new user.
    pub fn create_user(&self, user: &User) -> Result<User> {
        self.runtime.block_on(self.inner.create_user(user))
    }

    /// Updates a user.
    pub fn update_user(&self, user_id: &str, user: &User) -> Result<User> {
        self.runtime.block_on(self.inner.update_user(user_id, user))
    }

    /// Patches a user with partial updates.
    pub fn patch_user(&self, user_id: &str, patch: &PatchRequest) -> Result<User> {
        self.runtime.block_on(self.inner.patch_user(user_id, patch))
    }

    /// Deletes a user.
    pub fn delete_user(&self, user_id: &str) -> Result<()> {
        self.runtime.block_on(self.inner.delete_user(user_id))
    }

    /// Searches for users.
    pub fn search_users(
        &self,
        filter: Option<&str>,
        start_index: Option<u32>,
        count: Option<u32>,
    ) -> Result<ScimResponse<User>> {
        self.runtime
            .block_on(self.inner.search_users(filter, start_index, count))
    }

    /// Gets a group by ID.
    pub fn get_group(&self, group_id: &str) -> Result<Group> {
        self.runtime.block_on(self.inner.get_group(group_id))
    }

    /// Creates a new group.
    pub fn create_group(&self, group: &Group) -> Result<Group> {
        self.runtime.block_on(self.inner.create_group(group))
    }

    /// Updates a group.
    pub fn update_group(&self, group_id: &str, group: &Group) -> Result<Group> {
        self.runtime
            .block_on(self.inner.update_group(group_id, group))
    }

    /// Patches a group with partial updates.
    pub fn patch_group(&self, group_id: &str, patch: &PatchRequest) -> Result<Group> {
        self.runtime
            .block_on(self.inner.patch_group(group_id, patch))
    }

    /// Deletes a group.
    pub fn delete_group(&self, group_id: &str) -> Result<()> {
        self.runtime.block_on(self.inner.delete_group(group_id))
    }

    /// Searches for groups.
    pub fn search_groups(
        &self,
        filter: Option<&str>,
        start_index: Option<u32>,
        count: Option<u32>,
    ) -> Result<ScimResponse<Group>> {
        self.runtime
            .block_on(self.inner.search_groups(filter, start_index, count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_scim_client_new() {
        let client = AsyncScimClient::new("xoxp-test-token");
        assert_eq!(client.token, "xoxp-test-token");
        assert_eq!(client.base_url, SCIM_BASE_URL);
    }

    #[test]
    fn test_async_scim_client_with_base_url() {
        let client =
            AsyncScimClient::new("xoxp-test").with_base_url("https://custom.example.com/scim/v1/");
        assert_eq!(client.base_url, "https://custom.example.com/scim/v1/");
    }

    #[test]
    fn test_scim_client_new() {
        let client = ScimClient::new("xoxp-test-token");
        assert_eq!(client.inner.token, "xoxp-test-token");
    }

    #[test]
    fn test_scim_client_with_base_url() {
        let client =
            ScimClient::new("xoxp-test").with_base_url("https://custom.example.com/scim/v1/");
        assert_eq!(client.inner.base_url, "https://custom.example.com/scim/v1/");
    }

    #[test]
    fn test_build_headers() {
        let client = AsyncScimClient::new("xoxp-test-token");
        let headers = client.build_headers().unwrap();

        assert!(headers.contains_key(AUTHORIZATION));
        assert!(headers.contains_key(CONTENT_TYPE));
    }

    #[test]
    fn test_client_clone() {
        let client1 = AsyncScimClient::new("xoxp-test");
        let client2 = client1.clone();

        assert_eq!(client1.token, client2.token);
        assert_eq!(client1.base_url, client2.base_url);
    }
}
