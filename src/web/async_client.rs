//! Asynchronous Web API client for Slack.
//!
//! This module provides the `AsyncWebClient` which makes async HTTP requests
//! to the Slack Web API. It supports all 292+ API methods with automatic
//! retry handling, pagination, and proper error handling.

use crate::error::{Result, SlackError};
use crate::http_retry::RetryHandler;
use crate::web::internal_utils::{
    convert_bool_to_0_or_1, get_headers, get_url, remove_none_values,
};
use crate::web::response::SlackResponse;
use reqwest::multipart::Form;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

/// Asynchronous Slack Web API client.
///
/// This client handles constructing and sending HTTP requests to Slack
/// as well as parsing responses into `SlackResponse` objects.
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use slack_rs::web::AsyncWebClient;
///
/// #[tokio::main]
/// async fn main() -> slack_rs::error::Result<()> {
///     let client = AsyncWebClient::new("xoxb-your-token");
///
///     // Test the connection
///     let response = client.api_test(None).await?;
///     println!("API test: {}", response["ok"]);
///
///     // Post a message
///     let params = serde_json::json!({
///         "channel": "C123456",
///         "text": "Hello from Rust!"
///     });
///     let response = client.chat_post_message(Some(params)).await?;
///     println!("Message sent: {}", response["ts"]);
///
///     Ok(())
/// }
/// ```
///
/// With pagination:
/// ```no_run
/// use slack_rs::web::AsyncWebClient;
///
/// #[tokio::main]
/// async fn main() -> slack_rs::error::Result<()> {
///     let client = AsyncWebClient::new("xoxb-your-token");
///
///     let mut all_users = Vec::new();
///     let params = serde_json::json!({"limit": 100});
///     let mut response = client.users_list(Some(params)).await?;
///
///     loop {
///         if let Some(members) = response["members"].as_array() {
///             all_users.extend(members.clone());
///         }
///         if !response.has_next_cursor() {
///             break;
///         }
///         response = response.next().await?;
///     }
///
///     println!("Total users: {}", all_users.len());
///     Ok(())
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct AsyncWebClient {
    /// The Slack API token (xoxb-* or xoxp-*)
    token: Option<String>,

    /// Base URL for the Slack API
    base_url: String,

    /// HTTP client
    http_client: reqwest::Client,

    /// Default request timeout
    timeout: Duration,

    /// Default headers to include in all requests
    headers: HashMap<String, String>,

    /// Retry handlers for failed requests (kept for future extensibility)
    #[allow(dead_code)]
    retry_handlers: Vec<Box<dyn RetryHandler + Send + Sync>>,

    /// Maximum number of retry attempts
    max_retries: usize,
}

impl Clone for AsyncWebClient {
    fn clone(&self) -> Self {
        Self {
            token: self.token.clone(),
            base_url: self.base_url.clone(),
            http_client: self.http_client.clone(),
            timeout: self.timeout,
            headers: self.headers.clone(),
            retry_handlers: vec![], // Don't clone retry handlers
            max_retries: self.max_retries,
        }
    }
}

impl AsyncWebClient {
    /// The default Slack API base URL
    pub const BASE_URL: &'static str = "https://slack.com/api/";

    /// Creates a new AsyncWebClient with the given token.
    ///
    /// # Arguments
    ///
    /// * `token` - The Slack API token (xoxb-* or xoxp-*)
    ///
    /// # Examples
    ///
    /// ```
    /// use slack_rs::web::AsyncWebClient;
    ///
    /// let client = AsyncWebClient::new("xoxb-your-token");
    /// ```
    pub fn new(token: impl Into<String>) -> Self {
        Self::builder().token(token).build()
    }

    /// Creates a builder for constructing an AsyncWebClient with custom settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use slack_rs::web::AsyncWebClient;
    /// use std::time::Duration;
    ///
    /// let client = AsyncWebClient::builder()
    ///     .token("xoxb-your-token")
    ///     .base_url("https://slack.com/api/")
    ///     .timeout(Duration::from_secs(60))
    ///     .build();
    /// ```
    pub fn builder() -> AsyncWebClientBuilder {
        AsyncWebClientBuilder::default()
    }

    /// Makes a generic API call to any Slack Web API method.
    ///
    /// This is the core method that all specific API methods delegate to.
    ///
    /// # Arguments
    ///
    /// * `api_method` - The Slack API method name (e.g., "chat.postMessage")
    /// * `params` - Optional JSON object with parameters
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the API returns an error.
    pub async fn api_call(&self, api_method: &str, params: Option<Value>) -> Result<SlackResponse> {
        let url = get_url(&self.base_url, api_method);

        // Prepare parameters
        let mut req_params = params.unwrap_or(Value::Object(serde_json::Map::new()));

        // Convert bools to "0"/"1"
        req_params = convert_bool_to_0_or_1(&req_params);

        // Remove null values
        req_params = remove_none_values(req_params);

        // Build headers
        let has_json = true;
        let has_files = false;
        let headers = get_headers(
            self.token.as_deref(),
            has_json,
            has_files,
            &self.headers,
            None,
        );

        // Make the HTTP request with retry logic
        let mut retry_count = 0;

        loop {
            // Build request
            let mut req = self.http_client.post(&url).timeout(self.timeout);

            // Add headers
            for (key, value) in &headers {
                req = req.header(key, value);
            }

            // Add JSON body
            req = req.json(&req_params);

            // Send request
            match req.send().await {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    let response_headers = response.headers().clone();

                    // Check if we should retry based on status code
                    if (status_code == 429 || (500..600).contains(&status_code))
                        && retry_count < self.max_retries
                    {
                        retry_count += 1;

                        // Extract retry-after header if present
                        let wait_time = if status_code == 429 {
                            response_headers
                                .get("retry-after")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|v| v.parse::<u64>().ok())
                                .map(Duration::from_secs)
                                .unwrap_or_else(|| Duration::from_secs(1))
                        } else {
                            // Exponential backoff for server errors
                            Duration::from_millis(100 * 2_u64.pow(retry_count as u32))
                        };

                        tokio::time::sleep(wait_time).await;
                        continue;
                    }

                    // Parse response body
                    let data: Value = response.json().await.map_err(|e| SlackError::HttpError {
                        message: format!("Failed to parse response JSON: {}", e),
                    })?;

                    // Create and return SlackResponse
                    let slack_response = SlackResponse::new(
                        "POST".to_string(),
                        url.clone(),
                        data,
                        response_headers,
                        status_code,
                    )
                    .with_client_ref(self.token.clone(), self.base_url.clone());

                    return slack_response.validate();
                }
                Err(e) => {
                    // Connection or request error
                    if retry_count < self.max_retries {
                        retry_count += 1;
                        let wait_time = Duration::from_millis(100 * 2_u64.pow(retry_count as u32));
                        tokio::time::sleep(wait_time).await;
                        continue;
                    } else {
                        return Err(SlackError::HttpError {
                            message: format!("HTTP request failed: {}", e),
                        });
                    }
                }
            }
        }
    }

    /// Makes an API call with file uploads.
    ///
    /// # Arguments
    ///
    /// * `api_method` - The Slack API method name
    /// * `params` - Optional JSON object with parameters
    /// * `files` - HashMap of field names to file data (name, content, mime_type)
    pub async fn api_call_with_files(
        &self,
        api_method: &str,
        params: Option<Value>,
        files: HashMap<String, (String, Vec<u8>, String)>,
    ) -> Result<SlackResponse> {
        let url = get_url(&self.base_url, api_method);

        // Prepare parameters
        let req_params = params.unwrap_or(Value::Object(serde_json::Map::new()));

        // Build multipart form
        let mut form = Form::new();

        // Add parameters as form fields
        if let Value::Object(map) = &req_params {
            for (key, value) in map {
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                form = form.text(key.clone(), value_str);
            }
        }

        // Add files
        for (field_name, (file_name, content, mime_type)) in files {
            let part = reqwest::multipart::Part::bytes(content)
                .file_name(file_name)
                .mime_str(&mime_type)
                .map_err(|e| SlackError::InvalidInput {
                    message: format!("Invalid MIME type: {}", e),
                })?;
            form = form.part(field_name, part);
        }

        // Build headers
        let has_json = false;
        let has_files = true;
        let headers = get_headers(
            self.token.as_deref(),
            has_json,
            has_files,
            &self.headers,
            None,
        );

        // Build request
        let mut req = self.http_client.post(&url).timeout(self.timeout);

        // Add headers (except Content-Type, which reqwest sets for multipart)
        for (key, value) in &headers {
            if key != "Content-Type" {
                req = req.header(key, value);
            }
        }

        // Add multipart form
        req = req.multipart(form);

        // Send request
        let response = req.send().await.map_err(|e| SlackError::HttpError {
            message: format!("HTTP request failed: {}", e),
        })?;

        let status_code = response.status().as_u16();
        let response_headers = response.headers().clone();
        let data: Value = response.json().await.map_err(|e| SlackError::HttpError {
            message: format!("Failed to parse response JSON: {}", e),
        })?;

        SlackResponse::new("POST".to_string(), url, data, response_headers, status_code)
            .with_client_ref(self.token.clone(), self.base_url.clone())
            .validate()
    }
}

// Macro to generate API method wrappers
macro_rules! api_method {
    ($name:ident, $endpoint:expr) => {
        #[doc = concat!("Calls the `", $endpoint, "` API method.")]
        #[doc = ""]
        #[doc = concat!("See: https://api.slack.com/methods/", $endpoint)]
        pub async fn $name(&self, params: Option<Value>) -> Result<SlackResponse> {
            self.api_call($endpoint, params).await
        }
    };
}

// Implement all 292 API methods
impl AsyncWebClient {
    // Admin Analytics Methods
    api_method!(admin_analytics_get_file, "admin.analytics.getFile");

    // Admin Apps Methods
    api_method!(admin_apps_activities_list, "admin.apps.activities.list");
    api_method!(admin_apps_approve, "admin.apps.approve");
    api_method!(admin_apps_approved_list, "admin.apps.approved.list");
    api_method!(admin_apps_clear_resolution, "admin.apps.clearResolution");
    api_method!(admin_apps_config_lookup, "admin.apps.config.lookup");
    api_method!(admin_apps_config_set, "admin.apps.config.set");
    api_method!(admin_apps_requests_cancel, "admin.apps.requests.cancel");
    api_method!(admin_apps_requests_list, "admin.apps.requests.list");
    api_method!(admin_apps_restrict, "admin.apps.restrict");
    api_method!(admin_apps_restricted_list, "admin.apps.restricted.list");
    api_method!(admin_apps_uninstall, "admin.apps.uninstall");

    // Admin Auth Policy Methods
    api_method!(
        admin_auth_policy_assign_entities,
        "admin.auth.policy.assignEntities"
    );
    api_method!(
        admin_auth_policy_get_entities,
        "admin.auth.policy.getEntities"
    );
    api_method!(
        admin_auth_policy_remove_entities,
        "admin.auth.policy.removeEntities"
    );

    // Admin Barriers Methods
    api_method!(admin_barriers_create, "admin.barriers.create");
    api_method!(admin_barriers_delete, "admin.barriers.delete");
    api_method!(admin_barriers_list, "admin.barriers.list");
    api_method!(admin_barriers_update, "admin.barriers.update");

    // Admin Conversations Methods
    api_method!(admin_conversations_archive, "admin.conversations.archive");
    api_method!(
        admin_conversations_bulk_archive,
        "admin.conversations.bulkArchive"
    );
    api_method!(
        admin_conversations_bulk_delete,
        "admin.conversations.bulkDelete"
    );
    api_method!(
        admin_conversations_bulk_move,
        "admin.conversations.bulkMove"
    );
    api_method!(
        admin_conversations_convert_to_private,
        "admin.conversations.convertToPrivate"
    );
    api_method!(
        admin_conversations_convert_to_public,
        "admin.conversations.convertToPublic"
    );
    api_method!(admin_conversations_create, "admin.conversations.create");
    api_method!(
        admin_conversations_create_for_objects,
        "admin.conversations.createForObjects"
    );
    api_method!(admin_conversations_delete, "admin.conversations.delete");
    api_method!(
        admin_conversations_disconnect_shared,
        "admin.conversations.disconnectShared"
    );
    api_method!(
        admin_conversations_ekm_list_original_connected_channel_info,
        "admin.conversations.ekm.listOriginalConnectedChannelInfo"
    );
    api_method!(
        admin_conversations_get_conversation_prefs,
        "admin.conversations.getConversationPrefs"
    );
    api_method!(
        admin_conversations_get_custom_retention,
        "admin.conversations.getCustomRetention"
    );
    api_method!(
        admin_conversations_get_teams,
        "admin.conversations.getTeams"
    );
    api_method!(admin_conversations_invite, "admin.conversations.invite");
    api_method!(
        admin_conversations_link_objects,
        "admin.conversations.linkObjects"
    );
    api_method!(admin_conversations_lookup, "admin.conversations.lookup");
    api_method!(
        admin_conversations_remove_custom_retention,
        "admin.conversations.removeCustomRetention"
    );
    api_method!(admin_conversations_rename, "admin.conversations.rename");
    api_method!(
        admin_conversations_restrict_access_add_group,
        "admin.conversations.restrictAccess.addGroup"
    );
    api_method!(
        admin_conversations_restrict_access_list_groups,
        "admin.conversations.restrictAccess.listGroups"
    );
    api_method!(
        admin_conversations_restrict_access_remove_group,
        "admin.conversations.restrictAccess.removeGroup"
    );
    api_method!(admin_conversations_search, "admin.conversations.search");
    api_method!(
        admin_conversations_set_conversation_prefs,
        "admin.conversations.setConversationPrefs"
    );
    api_method!(
        admin_conversations_set_custom_retention,
        "admin.conversations.setCustomRetention"
    );
    api_method!(
        admin_conversations_set_teams,
        "admin.conversations.setTeams"
    );
    api_method!(
        admin_conversations_unarchive,
        "admin.conversations.unarchive"
    );
    api_method!(
        admin_conversations_unlink_objects,
        "admin.conversations.unlinkObjects"
    );

    // Admin Emoji Methods
    api_method!(admin_emoji_add, "admin.emoji.add");
    api_method!(admin_emoji_add_alias, "admin.emoji.addAlias");
    api_method!(admin_emoji_list, "admin.emoji.list");
    api_method!(admin_emoji_remove, "admin.emoji.remove");
    api_method!(admin_emoji_rename, "admin.emoji.rename");

    // Admin Functions Methods
    api_method!(admin_functions_list, "admin.functions.list");
    api_method!(
        admin_functions_permissions_lookup,
        "admin.functions.permissions.lookup"
    );
    api_method!(
        admin_functions_permissions_set,
        "admin.functions.permissions.set"
    );

    // Admin Invite Requests Methods
    api_method!(
        admin_invite_requests_approve,
        "admin.inviteRequests.approve"
    );
    api_method!(
        admin_invite_requests_approved_list,
        "admin.inviteRequests.approved.list"
    );
    api_method!(
        admin_invite_requests_denied_list,
        "admin.inviteRequests.denied.list"
    );
    api_method!(admin_invite_requests_deny, "admin.inviteRequests.deny");
    api_method!(admin_invite_requests_list, "admin.inviteRequests.list");

    // Admin Roles Methods
    api_method!(admin_roles_add_assignments, "admin.roles.addAssignments");
    api_method!(admin_roles_list_assignments, "admin.roles.listAssignments");
    api_method!(
        admin_roles_remove_assignments,
        "admin.roles.removeAssignments"
    );

    // Admin Teams Methods
    api_method!(admin_teams_admins_list, "admin.teams.admins.list");
    api_method!(admin_teams_create, "admin.teams.create");
    api_method!(admin_teams_list, "admin.teams.list");
    api_method!(admin_teams_owners_list, "admin.teams.owners.list");
    api_method!(admin_teams_settings_info, "admin.teams.settings.info");
    api_method!(
        admin_teams_settings_set_default_channels,
        "admin.teams.settings.setDefaultChannels"
    );
    api_method!(
        admin_teams_settings_set_description,
        "admin.teams.settings.setDescription"
    );
    api_method!(
        admin_teams_settings_set_discoverability,
        "admin.teams.settings.setDiscoverability"
    );
    api_method!(
        admin_teams_settings_set_icon,
        "admin.teams.settings.setIcon"
    );
    api_method!(
        admin_teams_settings_set_name,
        "admin.teams.settings.setName"
    );

    // Admin Usergroups Methods
    api_method!(
        admin_usergroups_add_channels,
        "admin.usergroups.addChannels"
    );
    api_method!(admin_usergroups_add_teams, "admin.usergroups.addTeams");
    api_method!(
        admin_usergroups_list_channels,
        "admin.usergroups.listChannels"
    );
    api_method!(
        admin_usergroups_remove_channels,
        "admin.usergroups.removeChannels"
    );

    // Admin Users Methods
    api_method!(admin_users_assign, "admin.users.assign");
    api_method!(admin_users_invite, "admin.users.invite");
    api_method!(admin_users_list, "admin.users.list");
    api_method!(admin_users_remove, "admin.users.remove");
    api_method!(admin_users_set_admin, "admin.users.setAdmin");
    api_method!(admin_users_set_expiration, "admin.users.setExpiration");
    api_method!(admin_users_set_owner, "admin.users.setOwner");
    api_method!(admin_users_set_regular, "admin.users.setRegular");

    // Admin Users Session Methods
    api_method!(
        admin_users_session_clear_settings,
        "admin.users.session.clearSettings"
    );
    api_method!(
        admin_users_session_get_settings,
        "admin.users.session.getSettings"
    );
    api_method!(
        admin_users_session_invalidate,
        "admin.users.session.invalidate"
    );
    api_method!(admin_users_session_list, "admin.users.session.list");
    api_method!(admin_users_session_reset, "admin.users.session.reset");
    api_method!(
        admin_users_session_reset_bulk,
        "admin.users.session.resetBulk"
    );
    api_method!(
        admin_users_session_set_settings,
        "admin.users.session.setSettings"
    );
    api_method!(
        admin_users_unsupported_versions_export,
        "admin.users.unsupportedVersions.export"
    );

    // Admin Workflows Methods
    api_method!(
        admin_workflows_collaborators_add,
        "admin.workflows.collaborators.add"
    );
    api_method!(
        admin_workflows_collaborators_remove,
        "admin.workflows.collaborators.remove"
    );
    api_method!(
        admin_workflows_permissions_lookup,
        "admin.workflows.permissions.lookup"
    );
    api_method!(admin_workflows_search, "admin.workflows.search");
    api_method!(admin_workflows_unpublish, "admin.workflows.unpublish");

    // API Methods
    api_method!(api_test, "api.test");

    // Apps Methods
    api_method!(apps_connections_open, "apps.connections.open");
    api_method!(
        apps_event_authorizations_list,
        "apps.event.authorizations.list"
    );
    api_method!(apps_manifest_create, "apps.manifest.create");
    api_method!(apps_manifest_delete, "apps.manifest.delete");
    api_method!(apps_manifest_export, "apps.manifest.export");
    api_method!(apps_manifest_update, "apps.manifest.update");
    api_method!(apps_manifest_validate, "apps.manifest.validate");
    api_method!(apps_uninstall, "apps.uninstall");

    // Auth Methods
    api_method!(auth_revoke, "auth.revoke");
    api_method!(auth_teams_list, "auth.teams.list");
    api_method!(auth_test, "auth.test");

    // Bookmarks Methods
    api_method!(bookmarks_add, "bookmarks.add");
    api_method!(bookmarks_edit, "bookmarks.edit");
    api_method!(bookmarks_list, "bookmarks.list");
    api_method!(bookmarks_remove, "bookmarks.remove");

    // Bots Methods
    api_method!(bots_info, "bots.info");

    // Calls Methods
    api_method!(calls_add, "calls.add");
    api_method!(calls_end, "calls.end");
    api_method!(calls_info, "calls.info");
    api_method!(calls_participants_add, "calls.participants.add");
    api_method!(calls_participants_remove, "calls.participants.remove");
    api_method!(calls_update, "calls.update");

    // Channels Methods (Legacy)
    api_method!(channels_archive, "channels.archive");
    api_method!(channels_create, "channels.create");
    api_method!(channels_history, "channels.history");
    api_method!(channels_info, "channels.info");
    api_method!(channels_invite, "channels.invite");
    api_method!(channels_join, "channels.join");
    api_method!(channels_kick, "channels.kick");
    api_method!(channels_leave, "channels.leave");
    api_method!(channels_list, "channels.list");
    api_method!(channels_mark, "channels.mark");
    api_method!(channels_rename, "channels.rename");
    api_method!(channels_replies, "channels.replies");
    api_method!(channels_set_purpose, "channels.setPurpose");
    api_method!(channels_set_topic, "channels.setTopic");
    api_method!(channels_unarchive, "channels.unarchive");

    // Chat Methods
    api_method!(chat_delete, "chat.delete");
    api_method!(chat_delete_scheduled_message, "chat.deleteScheduledMessage");
    api_method!(chat_get_permalink, "chat.getPermalink");
    api_method!(chat_me_message, "chat.meMessage");
    api_method!(chat_post_ephemeral, "chat.postEphemeral");
    api_method!(chat_post_message, "chat.postMessage");
    api_method!(chat_schedule_message, "chat.scheduleMessage");
    api_method!(chat_scheduled_messages_list, "chat.scheduledMessages.list");
    api_method!(chat_unfurl, "chat.unfurl");
    api_method!(chat_update, "chat.update");

    // Conversations Methods
    api_method!(
        conversations_accept_shared_invite,
        "conversations.acceptSharedInvite"
    );
    api_method!(
        conversations_approve_shared_invite,
        "conversations.approveSharedInvite"
    );
    api_method!(conversations_archive, "conversations.archive");
    api_method!(
        conversations_canvases_create,
        "conversations.canvases.create"
    );
    api_method!(conversations_close, "conversations.close");
    api_method!(conversations_create, "conversations.create");
    api_method!(
        conversations_decline_shared_invite,
        "conversations.declineSharedInvite"
    );
    api_method!(
        conversations_external_invite_permissions_set,
        "conversations.externalInvitePermissions.set"
    );
    api_method!(conversations_history, "conversations.history");
    api_method!(conversations_info, "conversations.info");
    api_method!(conversations_invite, "conversations.invite");
    api_method!(conversations_invite_shared, "conversations.inviteShared");
    api_method!(conversations_join, "conversations.join");
    api_method!(conversations_kick, "conversations.kick");
    api_method!(conversations_leave, "conversations.leave");
    api_method!(conversations_list, "conversations.list");
    api_method!(
        conversations_list_connect_invites,
        "conversations.listConnectInvites"
    );
    api_method!(conversations_mark, "conversations.mark");
    api_method!(conversations_members, "conversations.members");
    api_method!(conversations_open, "conversations.open");
    api_method!(conversations_rename, "conversations.rename");
    api_method!(conversations_replies, "conversations.replies");
    api_method!(
        conversations_request_shared_invite_approve,
        "conversations.requestSharedInvite.approve"
    );
    api_method!(
        conversations_request_shared_invite_deny,
        "conversations.requestSharedInvite.deny"
    );
    api_method!(
        conversations_request_shared_invite_list,
        "conversations.requestSharedInvite.list"
    );
    api_method!(conversations_set_purpose, "conversations.setPurpose");
    api_method!(conversations_set_topic, "conversations.setTopic");
    api_method!(conversations_unarchive, "conversations.unarchive");

    // Dialog Methods
    api_method!(dialog_open, "dialog.open");

    // DND Methods
    api_method!(dnd_end_dnd, "dnd.endDnd");
    api_method!(dnd_end_snooze, "dnd.endSnooze");
    api_method!(dnd_info, "dnd.info");
    api_method!(dnd_set_snooze, "dnd.setSnooze");
    api_method!(dnd_team_info, "dnd.teamInfo");

    // Emoji Methods
    api_method!(emoji_list, "emoji.list");

    // Files Methods
    api_method!(files_comments_delete, "files.comments.delete");
    api_method!(
        files_complete_upload_external,
        "files.completeUploadExternal"
    );
    api_method!(files_delete, "files.delete");
    api_method!(files_get_upload_url_external, "files.getUploadURLExternal");
    api_method!(files_info, "files.info");
    api_method!(files_list, "files.list");
    api_method!(files_remote_add, "files.remote.add");
    api_method!(files_remote_info, "files.remote.info");
    api_method!(files_remote_list, "files.remote.list");
    api_method!(files_remote_remove, "files.remote.remove");
    api_method!(files_remote_share, "files.remote.share");
    api_method!(files_remote_update, "files.remote.update");
    api_method!(files_revoke_public_url, "files.revokePublicURL");
    api_method!(files_shared_public_url, "files.sharedPublicURL");
    api_method!(files_upload, "files.upload");

    /// Uploads a file using the V2 API (recommended).
    ///
    /// This method uses the newer files.upload V2 API which is more efficient
    /// for large files.
    pub async fn files_upload_v2(
        &self,
        filename: String,
        content: Vec<u8>,
        params: Option<Value>,
    ) -> Result<SlackResponse> {
        // Step 1: Get upload URL
        let mut url_params = serde_json::json!({
            "filename": filename,
            "length": content.len()
        });

        if let Some(Value::Object(map)) = params {
            for (k, v) in map {
                url_params[k] = v;
            }
        }

        let url_response = self
            .api_call("files.getUploadURLExternal", Some(url_params))
            .await?;

        let upload_url =
            url_response["upload_url"]
                .as_str()
                .ok_or_else(|| SlackError::ApiError {
                    message: "No upload_url in response".to_string(),
                    response: url_response.data.clone(),
                })?;

        let file_id = url_response["file_id"]
            .as_str()
            .ok_or_else(|| SlackError::ApiError {
                message: "No file_id in response".to_string(),
                response: url_response.data.clone(),
            })?;

        // Step 2: Upload file to URL
        let upload_response = self
            .http_client
            .post(upload_url)
            .body(content)
            .send()
            .await
            .map_err(|e| SlackError::HttpError {
                message: format!("File upload failed: {}", e),
            })?;

        if !upload_response.status().is_success() {
            return Err(SlackError::HttpError {
                message: format!(
                    "File upload failed with status: {}",
                    upload_response.status()
                ),
            });
        }

        // Step 3: Complete the upload
        let complete_params = serde_json::json!({
            "files": [{"id": file_id}]
        });

        self.api_call("files.completeUploadExternal", Some(complete_params))
            .await
    }

    // Groups Methods (Legacy)
    api_method!(groups_archive, "groups.archive");
    api_method!(groups_create, "groups.create");
    api_method!(groups_create_child, "groups.createChild");
    api_method!(groups_history, "groups.history");
    api_method!(groups_info, "groups.info");
    api_method!(groups_invite, "groups.invite");
    api_method!(groups_kick, "groups.kick");
    api_method!(groups_leave, "groups.leave");
    api_method!(groups_list, "groups.list");
    api_method!(groups_mark, "groups.mark");
    api_method!(groups_open, "groups.open");
    api_method!(groups_rename, "groups.rename");
    api_method!(groups_replies, "groups.replies");
    api_method!(groups_set_purpose, "groups.setPurpose");
    api_method!(groups_set_topic, "groups.setTopic");
    api_method!(groups_unarchive, "groups.unarchive");

    // IM Methods (Legacy)
    api_method!(im_close, "im.close");
    api_method!(im_history, "im.history");
    api_method!(im_list, "im.list");
    api_method!(im_mark, "im.mark");
    api_method!(im_open, "im.open");
    api_method!(im_replies, "im.replies");

    // Migration Methods
    api_method!(migration_exchange, "migration.exchange");

    // MPIM Methods (Legacy)
    api_method!(mpim_close, "mpim.close");
    api_method!(mpim_history, "mpim.history");
    api_method!(mpim_list, "mpim.list");
    api_method!(mpim_mark, "mpim.mark");
    api_method!(mpim_open, "mpim.open");
    api_method!(mpim_replies, "mpim.replies");

    // OAuth Methods
    api_method!(oauth_access, "oauth.access");
    api_method!(oauth_v2_access, "oauth.v2.access");
    api_method!(oauth_v2_exchange, "oauth.v2.exchange");

    // Pins Methods
    api_method!(pins_add, "pins.add");
    api_method!(pins_list, "pins.list");
    api_method!(pins_remove, "pins.remove");

    // Reactions Methods
    api_method!(reactions_add, "reactions.add");
    api_method!(reactions_get, "reactions.get");
    api_method!(reactions_list, "reactions.list");
    api_method!(reactions_remove, "reactions.remove");

    // Reminders Methods
    api_method!(reminders_add, "reminders.add");
    api_method!(reminders_complete, "reminders.complete");
    api_method!(reminders_delete, "reminders.delete");
    api_method!(reminders_info, "reminders.info");
    api_method!(reminders_list, "reminders.list");

    // RTM Methods
    api_method!(rtm_connect, "rtm.connect");
    api_method!(rtm_start, "rtm.start");

    // Search Methods
    api_method!(search_all, "search.all");
    api_method!(search_files, "search.files");
    api_method!(search_messages, "search.messages");

    // Stars Methods
    api_method!(stars_add, "stars.add");
    api_method!(stars_list, "stars.list");
    api_method!(stars_remove, "stars.remove");

    // Team Methods
    api_method!(team_access_logs, "team.accessLogs");
    api_method!(team_billable_info, "team.billableInfo");
    api_method!(team_billing_info, "team.billing.info");
    api_method!(
        team_external_teams_disconnect,
        "team.externalTeams.disconnect"
    );
    api_method!(team_external_teams_list, "team.externalTeams.list");
    api_method!(team_info, "team.info");
    api_method!(team_integration_logs, "team.integrationLogs");
    api_method!(team_preferences_list, "team.preferences.list");
    api_method!(team_profile_get, "team.profile.get");

    // Usergroups Methods
    api_method!(usergroups_create, "usergroups.create");
    api_method!(usergroups_disable, "usergroups.disable");
    api_method!(usergroups_enable, "usergroups.enable");
    api_method!(usergroups_list, "usergroups.list");
    api_method!(usergroups_update, "usergroups.update");
    api_method!(usergroups_users_list, "usergroups.users.list");
    api_method!(usergroups_users_update, "usergroups.users.update");

    // Users Methods
    api_method!(users_conversations, "users.conversations");
    api_method!(users_delete_photo, "users.deletePhoto");
    api_method!(
        users_discoverable_contacts_lookup,
        "users.discoverableContacts.lookup"
    );
    api_method!(users_get_presence, "users.getPresence");
    api_method!(users_identity, "users.identity");
    api_method!(users_info, "users.info");
    api_method!(users_list, "users.list");
    api_method!(users_lookup_by_email, "users.lookupByEmail");
    api_method!(users_profile_get, "users.profile.get");
    api_method!(users_profile_set, "users.profile.set");
    api_method!(users_set_photo, "users.setPhoto");
    api_method!(users_set_presence, "users.setPresence");

    // Views Methods
    api_method!(views_open, "views.open");
    api_method!(views_publish, "views.publish");
    api_method!(views_push, "views.push");
    api_method!(views_update, "views.update");

    // Workflows Methods
    api_method!(workflows_featured_add, "workflows.featured.add");
    api_method!(workflows_featured_list, "workflows.featured.list");
    api_method!(workflows_featured_remove, "workflows.featured.remove");
    api_method!(workflows_featured_set, "workflows.featured.set");
    api_method!(workflows_step_completed, "workflows.stepCompleted");
    api_method!(workflows_step_failed, "workflows.stepFailed");
    api_method!(workflows_update_step, "workflows.updateStep");
}

/// Builder for constructing an AsyncWebClient with custom configuration.
#[allow(missing_debug_implementations)]
pub struct AsyncWebClientBuilder {
    token: Option<String>,
    base_url: String,
    timeout: Duration,
    headers: HashMap<String, String>,
    retry_handlers: Vec<Box<dyn RetryHandler + Send + Sync>>,
    max_retries: usize,
}

impl Default for AsyncWebClientBuilder {
    fn default() -> Self {
        Self {
            token: None,
            base_url: AsyncWebClient::BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
            headers: HashMap::new(),
            retry_handlers: vec![],
            max_retries: 3,
        }
    }
}

impl AsyncWebClientBuilder {
    /// Sets the Slack API token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Sets the base URL for the Slack API.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Adds a custom header to all requests.
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Adds custom headers to all requests.
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Adds a retry handler.
    pub fn retry_handler(mut self, handler: Box<dyn RetryHandler + Send + Sync>) -> Self {
        self.retry_handlers.push(handler);
        self
    }

    /// Sets the maximum number of retry attempts.
    pub fn max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Builds the AsyncWebClient.
    pub fn build(self) -> AsyncWebClient {
        let http_client = reqwest::Client::builder()
            .timeout(self.timeout)
            .build()
            .expect("Failed to create HTTP client");

        let retry_handlers = if self.retry_handlers.is_empty() {
            crate::http_retry::default_retry_handlers()
        } else {
            self.retry_handlers
        };

        AsyncWebClient {
            token: self.token,
            base_url: self.base_url,
            http_client,
            timeout: self.timeout,
            headers: self.headers,
            retry_handlers,
            max_retries: self.max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder() {
        let client = AsyncWebClient::builder()
            .token("xoxb-test")
            .base_url("https://test.slack.com/api/")
            .timeout(Duration::from_secs(60))
            .header("X-Custom", "value")
            .build();

        assert_eq!(client.token, Some("xoxb-test".to_string()));
        assert_eq!(client.base_url, "https://test.slack.com/api/");
        assert_eq!(client.timeout, Duration::from_secs(60));
        assert_eq!(client.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_client_new() {
        let client = AsyncWebClient::new("xoxb-test");
        assert_eq!(client.token, Some("xoxb-test".to_string()));
        assert_eq!(client.base_url, AsyncWebClient::BASE_URL);
    }
}
