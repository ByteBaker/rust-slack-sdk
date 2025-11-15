//! View objects for modals and Home tabs.

use crate::error::{Result, SlackError};
use crate::models::objects::TextObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Maximum length for view titles (24 characters).
pub const MAX_TITLE_LENGTH: usize = 24;

/// Maximum length for close button text (24 characters).
pub const MAX_CLOSE_LENGTH: usize = 24;

/// Maximum length for submit button text (24 characters).
pub const MAX_SUBMIT_LENGTH: usize = 24;

/// Maximum number of blocks in a view (100).
pub const MAX_BLOCKS: usize = 100;

/// Maximum length for private metadata (3000 characters).
pub const MAX_PRIVATE_METADATA_LENGTH: usize = 3000;

/// Maximum length for callback ID (255 characters).
pub const MAX_CALLBACK_ID_LENGTH: usize = 255;

/// A view object for modals and Home tabs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct View {
    /// The type of view ("modal" or "home").
    #[serde(rename = "type")]
    pub view_type: String,

    /// The view ID (assigned by Slack).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// A unique identifier for the view (max 255 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_id: Option<String>,

    /// An external ID for the view.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    /// The title of the view (required for modals, max 24 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<TextObject>,

    /// The text for the submit button (modals only, max 24 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit: Option<TextObject>,

    /// The text for the close button (modals only, max 24 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close: Option<TextObject>,

    /// An array of blocks that defines the content of the view (max 100).
    pub blocks: Vec<Value>,

    /// Private metadata for the view (max 3000 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_metadata: Option<String>,

    /// The state values from the view (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<ViewState>,

    /// A hash representing the view state (used for updates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// Whether to clear all views when this modal is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clear_on_close: Option<bool>,

    /// Whether to send an interaction payload when the view is closed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_on_close: Option<bool>,

    /// The team ID (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,

    /// The bot ID (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot_id: Option<String>,

    /// The app ID (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,

    /// The root view ID (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_view_id: Option<String>,

    /// The previous view ID (read-only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_view_id: Option<String>,
}

impl View {
    /// Creates a new modal view.
    ///
    /// # Arguments
    /// * `title` - The modal title (max 24 characters)
    /// * `blocks` - The blocks to display (max 100)
    pub fn modal(title: impl Into<String>, blocks: Vec<Value>) -> Result<Self> {
        let title_str = title.into();

        if title_str.is_empty() || title_str.len() > MAX_TITLE_LENGTH {
            return Err(SlackError::Validation(format!(
                "Modal title length must be between 1 and {}",
                MAX_TITLE_LENGTH
            )));
        }

        if blocks.is_empty() || blocks.len() > MAX_BLOCKS {
            return Err(SlackError::Validation(format!(
                "View must have between 1 and {} blocks",
                MAX_BLOCKS
            )));
        }

        Ok(Self {
            view_type: "modal".to_string(),
            id: None,
            callback_id: None,
            external_id: None,
            title: Some(TextObject::plain(title_str)?),
            submit: None,
            close: None,
            blocks,
            private_metadata: None,
            state: None,
            hash: None,
            clear_on_close: None,
            notify_on_close: None,
            team_id: None,
            bot_id: None,
            app_id: None,
            root_view_id: None,
            previous_view_id: None,
        })
    }

    /// Creates a new home tab view.
    ///
    /// # Arguments
    /// * `blocks` - The blocks to display (max 100)
    pub fn home(blocks: Vec<Value>) -> Result<Self> {
        if blocks.is_empty() || blocks.len() > MAX_BLOCKS {
            return Err(SlackError::Validation(format!(
                "View must have between 1 and {} blocks",
                MAX_BLOCKS
            )));
        }

        Ok(Self {
            view_type: "home".to_string(),
            id: None,
            callback_id: None,
            external_id: None,
            title: None,
            submit: None,
            close: None,
            blocks,
            private_metadata: None,
            state: None,
            hash: None,
            clear_on_close: None,
            notify_on_close: None,
            team_id: None,
            bot_id: None,
            app_id: None,
            root_view_id: None,
            previous_view_id: None,
        })
    }

    /// Sets the callback ID.
    pub fn with_callback_id(mut self, callback_id: impl Into<String>) -> Result<Self> {
        let id = callback_id.into();
        if id.len() > MAX_CALLBACK_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Callback ID length {} exceeds maximum {}",
                id.len(),
                MAX_CALLBACK_ID_LENGTH
            )));
        }
        self.callback_id = Some(id);
        Ok(self)
    }

    /// Sets the external ID.
    pub fn with_external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Sets the submit button text (modals only).
    pub fn with_submit(mut self, submit: impl Into<String>) -> Result<Self> {
        let submit_str = submit.into();
        if submit_str.len() > MAX_SUBMIT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Submit text length {} exceeds maximum {}",
                submit_str.len(),
                MAX_SUBMIT_LENGTH
            )));
        }
        self.submit = Some(TextObject::plain(submit_str)?);
        Ok(self)
    }

    /// Sets the close button text (modals only).
    pub fn with_close(mut self, close: impl Into<String>) -> Result<Self> {
        let close_str = close.into();
        if close_str.len() > MAX_CLOSE_LENGTH {
            return Err(SlackError::Validation(format!(
                "Close text length {} exceeds maximum {}",
                close_str.len(),
                MAX_CLOSE_LENGTH
            )));
        }
        self.close = Some(TextObject::plain(close_str)?);
        Ok(self)
    }

    /// Sets the private metadata.
    pub fn with_private_metadata(mut self, metadata: impl Into<String>) -> Result<Self> {
        let meta = metadata.into();
        if meta.len() > MAX_PRIVATE_METADATA_LENGTH {
            return Err(SlackError::Validation(format!(
                "Private metadata length {} exceeds maximum {}",
                meta.len(),
                MAX_PRIVATE_METADATA_LENGTH
            )));
        }
        self.private_metadata = Some(meta);
        Ok(self)
    }

    /// Sets whether to clear all views on close.
    pub fn with_clear_on_close(mut self, clear: bool) -> Self {
        self.clear_on_close = Some(clear);
        self
    }

    /// Sets whether to notify on close.
    pub fn with_notify_on_close(mut self, notify: bool) -> Self {
        self.notify_on_close = Some(notify);
        self
    }

    /// Validates the view.
    pub fn validate(&self) -> Result<()> {
        // Check view type
        if self.view_type != "modal" && self.view_type != "home" {
            return Err(SlackError::Validation(
                "View type must be either 'modal' or 'home'".to_string(),
            ));
        }

        // Home views cannot have submit or close buttons
        if self.view_type == "home" && (self.submit.is_some() || self.close.is_some()) {
            return Err(SlackError::Validation(
                "Home view cannot have submit or close buttons".to_string(),
            ));
        }

        // Modal views must have a title
        if self.view_type == "modal" && self.title.is_none() {
            return Err(SlackError::Validation(
                "Modal view must have a title".to_string(),
            ));
        }

        Ok(())
    }
}

/// The state of a view, containing values from input elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewState {
    /// A nested map of block_id -> action_id -> value.
    pub values: HashMap<String, HashMap<String, ViewStateValue>>,
}

impl ViewState {
    /// Creates a new view state.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Gets a value by block ID and action ID.
    pub fn get(&self, block_id: &str, action_id: &str) -> Option<&ViewStateValue> {
        self.values
            .get(block_id)
            .and_then(|actions| actions.get(action_id))
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new()
    }
}

/// A value from a view state input element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewStateValue {
    /// The type of element.
    #[serde(rename = "type")]
    pub value_type: String,

    /// The value (for plain text inputs, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Selected option (for single selects).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_option: Option<Value>,

    /// Selected options (for multi selects).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_options: Option<Vec<Value>>,

    /// Selected date (for date pickers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_date: Option<String>,

    /// Selected time (for time pickers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_time: Option<String>,

    /// Selected date-time (for date-time pickers).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_date_time: Option<i64>,

    /// Selected user (for user select).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_user: Option<String>,

    /// Selected users (for user multi-select).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_users: Option<Vec<String>>,

    /// Selected conversation (for conversation select).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_conversation: Option<String>,

    /// Selected conversations (for conversation multi-select).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_conversations: Option<Vec<String>>,

    /// Selected channel (for channel select).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_channel: Option<String>,

    /// Selected channels (for channel multi-select).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_channels: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // View tests
    #[test]
    fn test_modal_view_basic() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("My Modal", blocks).unwrap();

        assert_eq!(view.view_type, "modal");
        assert_eq!(view.title.as_ref().unwrap().text(), "My Modal");
        assert_eq!(view.blocks.len(), 1);
    }

    #[test]
    fn test_home_view_basic() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::home(blocks).unwrap();

        assert_eq!(view.view_type, "home");
        assert!(view.title.is_none());
        assert_eq!(view.blocks.len(), 1);
    }

    #[test]
    fn test_modal_view_with_submit_and_close() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("My Modal", blocks)
            .unwrap()
            .with_submit("Submit")
            .unwrap()
            .with_close("Cancel")
            .unwrap();

        assert_eq!(view.submit.as_ref().unwrap().text(), "Submit");
        assert_eq!(view.close.as_ref().unwrap().text(), "Cancel");
    }

    #[test]
    fn test_view_with_callback_id() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("My Modal", blocks)
            .unwrap()
            .with_callback_id("modal_callback")
            .unwrap();

        assert_eq!(view.callback_id, Some("modal_callback".to_string()));
    }

    #[test]
    fn test_view_with_private_metadata() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("My Modal", blocks)
            .unwrap()
            .with_private_metadata("secret data")
            .unwrap();

        assert_eq!(view.private_metadata, Some("secret data".to_string()));
    }

    #[test]
    fn test_modal_view_title_length_validation() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let long_title = "a".repeat(25);
        let result = View::modal(&long_title, blocks);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_empty_blocks_validation() {
        let blocks = vec![];
        let result = View::modal("Title", blocks);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_max_blocks_validation() {
        let blocks: Vec<Value> = (0..101)
            .map(|i| json!({"type": "section", "text": {"type": "mrkdwn", "text": format!("Text {}", i)}}))
            .collect();
        let result = View::modal("Title", blocks);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_exactly_100_blocks_ok() {
        let blocks: Vec<Value> = (0..100)
            .map(|i| json!({"type": "section", "text": {"type": "mrkdwn", "text": format!("Text {}", i)}}))
            .collect();
        let result = View::modal("Title", blocks);
        assert!(result.is_ok());
    }

    #[test]
    fn test_home_view_cannot_have_submit() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::home(blocks).unwrap().with_submit("Submit").unwrap();
        let result = view.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_modal_view_serialization() {
        let input = json!({
            "type": "modal",
            "title": {"type": "plain_text", "text": "My Modal"},
            "blocks": [
                {"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}}
            ]
        });

        let view: View = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&view).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_view_round_trip() {
        let input = json!({
            "type": "modal",
            "callback_id": "modal_1",
            "title": {"type": "plain_text", "text": "Title"},
            "submit": {"type": "plain_text", "text": "Submit"},
            "close": {"type": "plain_text", "text": "Cancel"},
            "blocks": [
                {"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}}
            ]
        });

        let view: View = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&view).unwrap();

        assert_eq!(input, output);
    }

    // ViewState tests
    #[test]
    fn test_view_state_basic() {
        let state = ViewState::new();
        assert_eq!(state.values.len(), 0);
    }

    #[test]
    fn test_view_state_serialization() {
        let input = json!({
            "values": {
                "block_1": {
                    "action_1": {
                        "type": "plain_text_input",
                        "value": "user input"
                    }
                }
            }
        });

        let state: ViewState = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&state).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_view_state_value_serialization() {
        let input = json!({
            "type": "plain_text_input",
            "value": "test value"
        });

        let value: ViewStateValue = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&value).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_view_with_external_id() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("Title", blocks)
            .unwrap()
            .with_external_id("ext123");
        assert_eq!(view.external_id, Some("ext123".to_string()));
    }

    #[test]
    fn test_view_with_clear_on_close() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("Title", blocks)
            .unwrap()
            .with_clear_on_close(true);
        assert_eq!(view.clear_on_close, Some(true));
    }

    #[test]
    fn test_view_with_notify_on_close() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let view = View::modal("Title", blocks)
            .unwrap()
            .with_notify_on_close(true);
        assert_eq!(view.notify_on_close, Some(true));
    }

    #[test]
    fn test_view_callback_id_length_validation() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let long_id = "a".repeat(256);
        let result = View::modal("Title", blocks).unwrap().with_callback_id(&long_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_private_metadata_length_validation() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let long_meta = "a".repeat(3001);
        let result = View::modal("Title", blocks)
            .unwrap()
            .with_private_metadata(&long_meta);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_submit_length_validation() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let long_submit = "a".repeat(25);
        let result = View::modal("Title", blocks).unwrap().with_submit(&long_submit);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_close_length_validation() {
        let blocks = vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})];
        let long_close = "a".repeat(25);
        let result = View::modal("Title", blocks).unwrap().with_close(&long_close);
        assert!(result.is_err());
    }

    #[test]
    fn test_view_validate_modal_requires_title() {
        let mut view = View::modal(
            "Title",
            vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})],
        )
        .unwrap();
        view.title = None;
        let result = view.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_view_validate_invalid_type() {
        let mut view = View::modal(
            "Title",
            vec![json!({"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}})],
        )
        .unwrap();
        view.view_type = "invalid".to_string();
        let result = view.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_view_state_get() {
        let mut state = ViewState::new();
        let mut actions = HashMap::new();
        actions.insert(
            "action_1".to_string(),
            ViewStateValue {
                value_type: "plain_text_input".to_string(),
                value: Some("test".to_string()),
                selected_option: None,
                selected_options: None,
                selected_date: None,
                selected_time: None,
                selected_date_time: None,
                selected_user: None,
                selected_users: None,
                selected_conversation: None,
                selected_conversations: None,
                selected_channel: None,
                selected_channels: None,
            },
        );
        state.values.insert("block_1".to_string(), actions);

        let value = state.get("block_1", "action_1");
        assert!(value.is_some());
        assert_eq!(value.unwrap().value, Some("test".to_string()));
    }

    #[test]
    fn test_view_state_get_missing() {
        let state = ViewState::new();
        let value = state.get("block_1", "action_1");
        assert!(value.is_none());
    }

    #[test]
    fn test_home_view_serialization() {
        let input = json!({
            "type": "home",
            "blocks": [
                {"type": "section", "text": {"type": "mrkdwn", "text": "Hello"}}
            ]
        });

        let view: View = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&view).unwrap();

        assert_eq!(input, output);
    }
}
