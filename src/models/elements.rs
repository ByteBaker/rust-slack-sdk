//! Block Kit interactive elements.
//!
//! Elements are smaller UI components that can be used inside blocks.
//! They include buttons, select menus, date pickers, and more.

use crate::error::{Result, SlackError};
use crate::models::objects::{ConfirmObject, OptionGroup, SlackOption, TextObject};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum length for action IDs (255 characters).
pub const MAX_ACTION_ID_LENGTH: usize = 255;

/// Maximum length for placeholder text (150 characters).
pub const MAX_PLACEHOLDER_LENGTH: usize = 150;

/// Maximum length for button text (75 characters).
pub const MAX_BUTTON_TEXT_LENGTH: usize = 75;

/// Maximum length for button values (2000 characters).
pub const MAX_BUTTON_VALUE_LENGTH: usize = 2000;

/// Maximum length for button URLs (3000 characters).
pub const MAX_BUTTON_URL_LENGTH: usize = 3000;

/// Maximum length for image URLs (3000 characters).
pub const MAX_IMAGE_URL_LENGTH: usize = 3000;

/// Maximum length for image alt text (2000 characters).
pub const MAX_IMAGE_ALT_TEXT_LENGTH: usize = 2000;

/// Maximum length for plain text input (3000 characters).
pub const MAX_PLAIN_TEXT_INPUT_LENGTH: usize = 3000;

/// Maximum number of options in a select menu (100).
pub const MAX_SELECT_OPTIONS: usize = 100;

/// Maximum number of option groups in a select menu (100).
pub const MAX_SELECT_OPTION_GROUPS: usize = 100;

/// Maximum number of initial options for multi-select (100).
pub const MAX_MULTI_SELECT_INITIAL_OPTIONS: usize = 100;

/// Maximum number of options in overflow menu (5).
pub const MAX_OVERFLOW_OPTIONS: usize = 5;

/// Maximum number of options in checkboxes (10).
pub const MAX_CHECKBOX_OPTIONS: usize = 10;

/// Maximum number of options in radio buttons (10).
pub const MAX_RADIO_BUTTON_OPTIONS: usize = 10;

/// Style for buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonStyle {
    /// Default button style.
    #[serde(rename = "default")]
    Default,
    /// Primary (green) button.
    Primary,
    /// Danger (red) button.
    Danger,
}

/// A button element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ButtonElement {
    /// The type of element (always "button").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The text shown on the button.
    pub text: TextObject,

    /// The action ID for this button.
    pub action_id: String,

    /// The value sent when the button is clicked (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// A URL to load when the button is clicked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// The button style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ButtonStyle>,

    /// A confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Accessibility label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessibility_label: Option<String>,
}

impl ButtonElement {
    /// Creates a new button element.
    ///
    /// # Arguments
    /// * `text` - The button text (max 75 characters, plain text only)
    /// * `action_id` - The action ID (max 255 characters)
    pub fn new(text: impl Into<String>, action_id: impl Into<String>) -> Result<Self> {
        let text_str = text.into();
        let action_id_str = action_id.into();

        if text_str.len() > MAX_BUTTON_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Button text length {} exceeds maximum {}",
                text_str.len(),
                MAX_BUTTON_TEXT_LENGTH
            )));
        }

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "button".to_string(),
            text: TextObject::plain(text_str)?,
            action_id: action_id_str,
            value: None,
            url: None,
            style: None,
            confirm: None,
            accessibility_label: None,
        })
    }

    /// Sets the value for this button.
    pub fn with_value(mut self, value: impl Into<String>) -> Result<Self> {
        let value_str = value.into();
        if value_str.len() > MAX_BUTTON_VALUE_LENGTH {
            return Err(SlackError::Validation(format!(
                "Button value length {} exceeds maximum {}",
                value_str.len(),
                MAX_BUTTON_VALUE_LENGTH
            )));
        }
        self.value = Some(value_str);
        Ok(self)
    }

    /// Sets the URL for this button.
    pub fn with_url(mut self, url: impl Into<String>) -> Result<Self> {
        let url_str = url.into();
        if url_str.len() > MAX_BUTTON_URL_LENGTH {
            return Err(SlackError::Validation(format!(
                "Button URL length {} exceeds maximum {}",
                url_str.len(),
                MAX_BUTTON_URL_LENGTH
            )));
        }
        self.url = Some(url_str);
        Ok(self)
    }

    /// Sets the style for this button.
    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Sets the confirmation dialog.
    pub fn with_confirm(mut self, confirm: ConfirmObject) -> Self {
        self.confirm = Some(confirm);
        self
    }

    /// Sets the accessibility label.
    pub fn with_accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

/// An image element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageElement {
    /// The type of element (always "image").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The URL of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    /// Alternative representation using Slack file object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slack_file: Option<Value>,

    /// Alt text for the image.
    pub alt_text: String,
}

impl ImageElement {
    /// Creates a new image element with a URL.
    ///
    /// # Arguments
    /// * `image_url` - The image URL (max 3000 characters)
    /// * `alt_text` - Alt text for the image (max 2000 characters)
    pub fn new(image_url: impl Into<String>, alt_text: impl Into<String>) -> Result<Self> {
        let url = image_url.into();
        let alt = alt_text.into();

        if url.len() > MAX_IMAGE_URL_LENGTH {
            return Err(SlackError::Validation(format!(
                "Image URL length {} exceeds maximum {}",
                url.len(),
                MAX_IMAGE_URL_LENGTH
            )));
        }

        if alt.len() > MAX_IMAGE_ALT_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Image alt text length {} exceeds maximum {}",
                alt.len(),
                MAX_IMAGE_ALT_TEXT_LENGTH
            )));
        }

        Ok(Self {
            element_type: "image".to_string(),
            image_url: Some(url),
            slack_file: None,
            alt_text: alt,
        })
    }

    /// Creates a new image element with a Slack file.
    pub fn from_slack_file(slack_file: Value, alt_text: impl Into<String>) -> Result<Self> {
        let alt = alt_text.into();

        if alt.len() > MAX_IMAGE_ALT_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Image alt text length {} exceeds maximum {}",
                alt.len(),
                MAX_IMAGE_ALT_TEXT_LENGTH
            )));
        }

        Ok(Self {
            element_type: "image".to_string(),
            image_url: None,
            slack_file: Some(slack_file),
            alt_text: alt,
        })
    }
}

/// Dispatch action configuration for plain text input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchActionConfig {
    /// Trigger actions when pressing enter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_actions_on: Option<Vec<String>>,
}

/// A plain text input element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlainTextInputElement {
    /// The type of element (always "plain_text_input").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text (max 150 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial value for the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_value: Option<String>,

    /// Whether this is a multiline input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiline: Option<bool>,

    /// Minimum length for input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length for input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Dispatch action configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatch_action_config: Option<DispatchActionConfig>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl PlainTextInputElement {
    /// Creates a new plain text input element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "plain_text_input".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_value: None,
            multiline: None,
            min_length: None,
            max_length: None,
            dispatch_action_config: None,
            focus_on_load: None,
        })
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Result<Self> {
        let text = placeholder.into();
        if text.len() > MAX_PLACEHOLDER_LENGTH {
            return Err(SlackError::Validation(format!(
                "Placeholder length {} exceeds maximum {}",
                text.len(),
                MAX_PLACEHOLDER_LENGTH
            )));
        }
        self.placeholder = Some(TextObject::plain(text)?);
        Ok(self)
    }

    /// Sets the initial value.
    pub fn with_initial_value(mut self, value: impl Into<String>) -> Result<Self> {
        let val = value.into();
        if val.len() > MAX_PLAIN_TEXT_INPUT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Initial value length {} exceeds maximum {}",
                val.len(),
                MAX_PLAIN_TEXT_INPUT_LENGTH
            )));
        }
        self.initial_value = Some(val);
        Ok(self)
    }

    /// Sets whether this is a multiline input.
    pub fn with_multiline(mut self, multiline: bool) -> Self {
        self.multiline = Some(multiline);
        self
    }

    /// Sets the minimum length.
    pub fn with_min_length(mut self, min: usize) -> Self {
        self.min_length = Some(min);
        self
    }

    /// Sets the maximum length.
    pub fn with_max_length(mut self, max: usize) -> Result<Self> {
        if max > MAX_PLAIN_TEXT_INPUT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Max length {} exceeds maximum {}",
                max, MAX_PLAIN_TEXT_INPUT_LENGTH
            )));
        }
        self.max_length = Some(max);
        Ok(self)
    }

    /// Sets whether to focus on load.
    pub fn with_focus_on_load(mut self, focus: bool) -> Self {
        self.focus_on_load = Some(focus);
        self
    }
}

/// Options or option groups for select menus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SelectOptions {
    /// A list of options.
    Options(Vec<SlackOption>),
    /// A list of option groups.
    OptionGroups(Vec<OptionGroup>),
}

/// A static select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticSelectElement {
    /// The type of element (always "static_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Options for the select menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SlackOption>>,

    /// Option groups for the select menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_groups: Option<Vec<OptionGroup>>,

    /// Initial option selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_option: Option<SlackOption>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl StaticSelectElement {
    /// Creates a new static select element with options.
    pub fn new(action_id: impl Into<String>, options: Vec<SlackOption>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        if options.len() > MAX_SELECT_OPTIONS {
            return Err(SlackError::Validation(format!(
                "Options count {} exceeds maximum {}",
                options.len(),
                MAX_SELECT_OPTIONS
            )));
        }

        Ok(Self {
            element_type: "static_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            options: Some(options),
            option_groups: None,
            initial_option: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Creates a new static select element with option groups.
    pub fn with_option_groups(
        action_id: impl Into<String>,
        option_groups: Vec<OptionGroup>,
    ) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        if option_groups.len() > MAX_SELECT_OPTION_GROUPS {
            return Err(SlackError::Validation(format!(
                "Option groups count {} exceeds maximum {}",
                option_groups.len(),
                MAX_SELECT_OPTION_GROUPS
            )));
        }

        Ok(Self {
            element_type: "static_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            options: None,
            option_groups: Some(option_groups),
            initial_option: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Result<Self> {
        let text = placeholder.into();
        if text.len() > MAX_PLACEHOLDER_LENGTH {
            return Err(SlackError::Validation(format!(
                "Placeholder length {} exceeds maximum {}",
                text.len(),
                MAX_PLACEHOLDER_LENGTH
            )));
        }
        self.placeholder = Some(TextObject::plain(text)?);
        Ok(self)
    }

    /// Sets the initial option.
    pub fn with_initial_option(mut self, option: SlackOption) -> Self {
        self.initial_option = Some(option);
        self
    }

    /// Sets the confirmation dialog.
    pub fn with_confirm(mut self, confirm: ConfirmObject) -> Self {
        self.confirm = Some(confirm);
        self
    }
}

/// A static multi-select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticMultiSelectElement {
    /// The type of element (always "multi_static_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Options for the select menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<SlackOption>>,

    /// Option groups for the select menu.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option_groups: Option<Vec<OptionGroup>>,

    /// Initial options selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_options: Option<Vec<SlackOption>>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Maximum number of items that can be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_selected_items: Option<usize>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl StaticMultiSelectElement {
    /// Creates a new static multi-select element with options.
    pub fn new(action_id: impl Into<String>, options: Vec<SlackOption>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        if options.len() > MAX_SELECT_OPTIONS {
            return Err(SlackError::Validation(format!(
                "Options count {} exceeds maximum {}",
                options.len(),
                MAX_SELECT_OPTIONS
            )));
        }

        Ok(Self {
            element_type: "multi_static_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            options: Some(options),
            option_groups: None,
            initial_options: None,
            confirm: None,
            max_selected_items: None,
            focus_on_load: None,
        })
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Result<Self> {
        let text = placeholder.into();
        if text.len() > MAX_PLACEHOLDER_LENGTH {
            return Err(SlackError::Validation(format!(
                "Placeholder length {} exceeds maximum {}",
                text.len(),
                MAX_PLACEHOLDER_LENGTH
            )));
        }
        self.placeholder = Some(TextObject::plain(text)?);
        Ok(self)
    }

    /// Sets the initial options.
    pub fn with_initial_options(mut self, options: Vec<SlackOption>) -> Result<Self> {
        if options.len() > MAX_MULTI_SELECT_INITIAL_OPTIONS {
            return Err(SlackError::Validation(format!(
                "Initial options count {} exceeds maximum {}",
                options.len(),
                MAX_MULTI_SELECT_INITIAL_OPTIONS
            )));
        }
        self.initial_options = Some(options);
        Ok(self)
    }

    /// Sets the maximum number of selected items.
    pub fn with_max_selected_items(mut self, max: usize) -> Self {
        self.max_selected_items = Some(max);
        self
    }
}

/// A user select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSelectElement {
    /// The type of element (always "users_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial user ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_user: Option<String>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl UserSelectElement {
    /// Creates a new user select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "users_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_user: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the placeholder text.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Result<Self> {
        let text = placeholder.into();
        if text.len() > MAX_PLACEHOLDER_LENGTH {
            return Err(SlackError::Validation(format!(
                "Placeholder length {} exceeds maximum {}",
                text.len(),
                MAX_PLACEHOLDER_LENGTH
            )));
        }
        self.placeholder = Some(TextObject::plain(text)?);
        Ok(self)
    }

    /// Sets the initial user.
    pub fn with_initial_user(mut self, user_id: impl Into<String>) -> Self {
        self.initial_user = Some(user_id.into());
        self
    }
}

/// A user multi-select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMultiSelectElement {
    /// The type of element (always "multi_users_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial user IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_users: Option<Vec<String>>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Maximum number of items that can be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_selected_items: Option<usize>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl UserMultiSelectElement {
    /// Creates a new user multi-select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "multi_users_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_users: None,
            confirm: None,
            max_selected_items: None,
            focus_on_load: None,
        })
    }
}

/// A conversation select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationSelectElement {
    /// The type of element (always "conversations_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial conversation ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_conversation: Option<String>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to include all public channels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_to_current_conversation: Option<bool>,

    /// Filter for conversation types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl ConversationSelectElement {
    /// Creates a new conversation select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "conversations_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_conversation: None,
            confirm: None,
            default_to_current_conversation: None,
            filter: None,
            focus_on_load: None,
        })
    }
}

/// A conversation multi-select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversationMultiSelectElement {
    /// The type of element (always "multi_conversations_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial conversation IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_conversations: Option<Vec<String>>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Maximum number of items that can be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_selected_items: Option<usize>,

    /// Filter for conversation types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Value>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl ConversationMultiSelectElement {
    /// Creates a new conversation multi-select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "multi_conversations_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_conversations: None,
            confirm: None,
            max_selected_items: None,
            filter: None,
            focus_on_load: None,
        })
    }
}

/// A channel select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelSelectElement {
    /// The type of element (always "channels_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial channel ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_channel: Option<String>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl ChannelSelectElement {
    /// Creates a new channel select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "channels_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_channel: None,
            confirm: None,
            focus_on_load: None,
        })
    }
}

/// A channel multi-select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelMultiSelectElement {
    /// The type of element (always "multi_channels_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial channel IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_channels: Option<Vec<String>>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Maximum number of items that can be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_selected_items: Option<usize>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl ChannelMultiSelectElement {
    /// Creates a new channel multi-select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "multi_channels_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_channels: None,
            confirm: None,
            max_selected_items: None,
            focus_on_load: None,
        })
    }
}

/// An external data select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalDataSelectElement {
    /// The type of element (always "external_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_option: Option<SlackOption>,

    /// Minimum query length before search starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_query_length: Option<usize>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl ExternalDataSelectElement {
    /// Creates a new external data select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "external_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_option: None,
            min_query_length: None,
            confirm: None,
            focus_on_load: None,
        })
    }
}

/// An external data multi-select menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalDataMultiSelectElement {
    /// The type of element (always "multi_external_select").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_options: Option<Vec<SlackOption>>,

    /// Minimum query length before search starts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_query_length: Option<usize>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Maximum number of items that can be selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_selected_items: Option<usize>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl ExternalDataMultiSelectElement {
    /// Creates a new external data multi-select element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "multi_external_select".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_options: None,
            min_query_length: None,
            confirm: None,
            max_selected_items: None,
            focus_on_load: None,
        })
    }
}

/// A date picker element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatePickerElement {
    /// The type of element (always "datepicker").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial date in YYYY-MM-DD format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_date: Option<String>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl DatePickerElement {
    /// Creates a new date picker element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "datepicker".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_date: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the initial date (must be in YYYY-MM-DD format).
    pub fn with_initial_date(mut self, date: impl Into<String>) -> Result<Self> {
        let date_str = date.into();
        // Simple validation for YYYY-MM-DD format
        if !date_str.is_empty() && date_str.len() != 10 {
            return Err(SlackError::Validation(
                "Date must be in YYYY-MM-DD format".to_string(),
            ));
        }
        self.initial_date = Some(date_str);
        Ok(self)
    }
}

/// A time picker element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimePickerElement {
    /// The type of element (always "timepicker").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Placeholder text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<TextObject>,

    /// Initial time in HH:mm format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_time: Option<String>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl TimePickerElement {
    /// Creates a new time picker element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "timepicker".to_string(),
            action_id: action_id_str,
            placeholder: None,
            initial_time: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the initial time (must be in HH:mm format).
    pub fn with_initial_time(mut self, time: impl Into<String>) -> Result<Self> {
        let time_str = time.into();
        // Simple validation for HH:mm format
        if !time_str.is_empty() && time_str.len() != 5 {
            return Err(SlackError::Validation(
                "Time must be in HH:mm format".to_string(),
            ));
        }
        self.initial_time = Some(time_str);
        Ok(self)
    }
}

/// A date-time picker element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DateTimePickerElement {
    /// The type of element (always "datetimepicker").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Initial date-time as Unix timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_date_time: Option<i64>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl DateTimePickerElement {
    /// Creates a new date-time picker element.
    pub fn new(action_id: impl Into<String>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        Ok(Self {
            element_type: "datetimepicker".to_string(),
            action_id: action_id_str,
            initial_date_time: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the initial date-time as Unix timestamp.
    pub fn with_initial_date_time(mut self, timestamp: i64) -> Self {
        self.initial_date_time = Some(timestamp);
        self
    }
}

/// A checkboxes element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckboxesElement {
    /// The type of element (always "checkboxes").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Options for the checkboxes (max 10).
    pub options: Vec<SlackOption>,

    /// Initially selected options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_options: Option<Vec<SlackOption>>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl CheckboxesElement {
    /// Creates a new checkboxes element.
    pub fn new(action_id: impl Into<String>, options: Vec<SlackOption>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        if options.len() > MAX_CHECKBOX_OPTIONS {
            return Err(SlackError::Validation(format!(
                "Checkboxes count {} exceeds maximum {}",
                options.len(),
                MAX_CHECKBOX_OPTIONS
            )));
        }

        Ok(Self {
            element_type: "checkboxes".to_string(),
            action_id: action_id_str,
            options,
            initial_options: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the initial options.
    pub fn with_initial_options(mut self, options: Vec<SlackOption>) -> Self {
        self.initial_options = Some(options);
        self
    }
}

/// A radio buttons element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadioButtonsElement {
    /// The type of element (always "radio_buttons").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Options for the radio buttons (max 10).
    pub options: Vec<SlackOption>,

    /// Initially selected option.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_option: Option<SlackOption>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,

    /// Whether to focus on load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_on_load: Option<bool>,
}

impl RadioButtonsElement {
    /// Creates a new radio buttons element.
    pub fn new(action_id: impl Into<String>, options: Vec<SlackOption>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        if options.len() > MAX_RADIO_BUTTON_OPTIONS {
            return Err(SlackError::Validation(format!(
                "Radio buttons count {} exceeds maximum {}",
                options.len(),
                MAX_RADIO_BUTTON_OPTIONS
            )));
        }

        Ok(Self {
            element_type: "radio_buttons".to_string(),
            action_id: action_id_str,
            options,
            initial_option: None,
            confirm: None,
            focus_on_load: None,
        })
    }

    /// Sets the initial option.
    pub fn with_initial_option(mut self, option: SlackOption) -> Self {
        self.initial_option = Some(option);
        self
    }
}

/// An overflow menu element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OverflowMenuElement {
    /// The type of element (always "overflow").
    #[serde(rename = "type")]
    pub element_type: String,

    /// The action ID.
    pub action_id: String,

    /// Options for the overflow menu (min 2, max 5).
    pub options: Vec<SlackOption>,

    /// Confirmation dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<ConfirmObject>,
}

impl OverflowMenuElement {
    /// Creates a new overflow menu element.
    pub fn new(action_id: impl Into<String>, options: Vec<SlackOption>) -> Result<Self> {
        let action_id_str = action_id.into();

        if action_id_str.len() > MAX_ACTION_ID_LENGTH {
            return Err(SlackError::Validation(format!(
                "Action ID length {} exceeds maximum {}",
                action_id_str.len(),
                MAX_ACTION_ID_LENGTH
            )));
        }

        if options.len() < 2 {
            return Err(SlackError::Validation(
                "Overflow menu must have at least 2 options".to_string(),
            ));
        }

        if options.len() > MAX_OVERFLOW_OPTIONS {
            return Err(SlackError::Validation(format!(
                "Overflow menu count {} exceeds maximum {}",
                options.len(),
                MAX_OVERFLOW_OPTIONS
            )));
        }

        Ok(Self {
            element_type: "overflow".to_string(),
            action_id: action_id_str,
            options,
            confirm: None,
        })
    }

    /// Sets the confirmation dialog.
    pub fn with_confirm(mut self, confirm: ConfirmObject) -> Self {
        self.confirm = Some(confirm);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // Button tests
    #[test]
    fn test_button_element_basic() {
        let button = ButtonElement::new("Click me", "button_1").unwrap();
        let json_val = serde_json::to_value(&button).unwrap();

        assert_eq!(json_val["type"], "button");
        assert_eq!(json_val["action_id"], "button_1");
        assert_eq!(json_val["text"]["text"], "Click me");
    }

    #[test]
    fn test_button_element_with_value() {
        let button = ButtonElement::new("Click", "btn")
            .unwrap()
            .with_value("value_1")
            .unwrap();

        let json_val = serde_json::to_value(&button).unwrap();
        assert_eq!(json_val["value"], "value_1");
    }

    #[test]
    fn test_button_element_with_style() {
        let button = ButtonElement::new("Click", "btn")
            .unwrap()
            .with_style(ButtonStyle::Primary);

        let json_val = serde_json::to_value(&button).unwrap();
        assert_eq!(json_val["style"], "primary");
    }

    #[test]
    fn test_button_element_with_url() {
        let button = ButtonElement::new("Click", "btn")
            .unwrap()
            .with_url("https://example.com")
            .unwrap();

        let json_val = serde_json::to_value(&button).unwrap();
        assert_eq!(json_val["url"], "https://example.com");
    }

    #[test]
    fn test_button_element_round_trip() {
        let input = json!({
            "type": "button",
            "text": {"type": "plain_text", "text": "Click"},
            "action_id": "btn_1",
            "value": "val",
            "style": "danger"
        });

        let button: ButtonElement = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&button).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_button_text_length_validation() {
        let long_text = "a".repeat(76);
        let result = ButtonElement::new(&long_text, "btn");
        assert!(result.is_err());
    }

    #[test]
    fn test_button_value_length_validation() {
        let long_value = "a".repeat(2001);
        let result = ButtonElement::new("Click", "btn")
            .unwrap()
            .with_value(&long_value);
        assert!(result.is_err());
    }

    // Image element tests
    #[test]
    fn test_image_element_basic() {
        let image = ImageElement::new("https://example.com/image.png", "An image").unwrap();
        let json_val = serde_json::to_value(&image).unwrap();

        assert_eq!(json_val["type"], "image");
        assert_eq!(json_val["image_url"], "https://example.com/image.png");
        assert_eq!(json_val["alt_text"], "An image");
    }

    #[test]
    fn test_image_element_round_trip() {
        let input = json!({
            "type": "image",
            "image_url": "https://example.com/img.png",
            "alt_text": "Alt text"
        });

        let image: ImageElement = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&image).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_image_element_url_length_validation() {
        let long_url = format!("https://example.com/{}", "a".repeat(3000));
        let result = ImageElement::new(&long_url, "alt");
        assert!(result.is_err());
    }

    #[test]
    fn test_image_element_alt_text_length_validation() {
        let long_alt = "a".repeat(2001);
        let result = ImageElement::new("https://example.com/img.png", &long_alt);
        assert!(result.is_err());
    }

    // Plain text input tests
    #[test]
    fn test_plain_text_input_basic() {
        let input = PlainTextInputElement::new("input_1").unwrap();
        let json_val = serde_json::to_value(&input).unwrap();

        assert_eq!(json_val["type"], "plain_text_input");
        assert_eq!(json_val["action_id"], "input_1");
    }

    #[test]
    fn test_plain_text_input_with_placeholder() {
        let input = PlainTextInputElement::new("input_1")
            .unwrap()
            .with_placeholder("Enter text...")
            .unwrap();

        let json_val = serde_json::to_value(&input).unwrap();
        assert_eq!(json_val["placeholder"]["text"], "Enter text...");
    }

    #[test]
    fn test_plain_text_input_multiline() {
        let input = PlainTextInputElement::new("input_1")
            .unwrap()
            .with_multiline(true);

        let json_val = serde_json::to_value(&input).unwrap();
        assert_eq!(json_val["multiline"], true);
    }

    #[test]
    fn test_plain_text_input_with_lengths() {
        let input = PlainTextInputElement::new("input_1")
            .unwrap()
            .with_min_length(10)
            .with_max_length(100)
            .unwrap();

        let json_val = serde_json::to_value(&input).unwrap();
        assert_eq!(json_val["min_length"], 10);
        assert_eq!(json_val["max_length"], 100);
    }

    #[test]
    fn test_plain_text_input_round_trip() {
        let input_json = json!({
            "type": "plain_text_input",
            "action_id": "input_1",
            "placeholder": {"type": "plain_text", "text": "Enter"},
            "multiline": true
        });

        let input: PlainTextInputElement = serde_json::from_value(input_json.clone()).unwrap();
        let output = serde_json::to_value(&input).unwrap();

        assert_eq!(input_json, output);
    }

    // Static select tests
    #[test]
    fn test_static_select_basic() {
        let options = vec![
            SlackOption::new("Option 1", "opt1").unwrap(),
            SlackOption::new("Option 2", "opt2").unwrap(),
        ];
        let select = StaticSelectElement::new("select_1", options).unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "static_select");
        assert_eq!(json_val["action_id"], "select_1");
        assert_eq!(json_val["options"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_static_select_with_placeholder() {
        let options = vec![SlackOption::new("Opt", "opt").unwrap()];
        let select = StaticSelectElement::new("select_1", options)
            .unwrap()
            .with_placeholder("Choose...")
            .unwrap();

        let json_val = serde_json::to_value(&select).unwrap();
        assert_eq!(json_val["placeholder"]["text"], "Choose...");
    }

    #[test]
    fn test_static_select_options_validation() {
        let options: Vec<SlackOption> = (0..101)
            .map(|i| SlackOption::new(format!("Opt {}", i), format!("opt{}", i)).unwrap())
            .collect();
        let result = StaticSelectElement::new("select_1", options);
        assert!(result.is_err());
    }

    #[test]
    fn test_static_multi_select_basic() {
        let options = vec![
            SlackOption::new("Option 1", "opt1").unwrap(),
            SlackOption::new("Option 2", "opt2").unwrap(),
        ];
        let select = StaticMultiSelectElement::new("multi_select_1", options).unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "multi_static_select");
        assert_eq!(json_val["action_id"], "multi_select_1");
    }

    #[test]
    fn test_static_multi_select_with_max_selected() {
        let options = vec![SlackOption::new("Opt", "opt").unwrap()];
        let select = StaticMultiSelectElement::new("select", options)
            .unwrap()
            .with_max_selected_items(5);

        let json_val = serde_json::to_value(&select).unwrap();
        assert_eq!(json_val["max_selected_items"], 5);
    }

    // User select tests
    #[test]
    fn test_user_select_basic() {
        let select = UserSelectElement::new("user_select").unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "users_select");
        assert_eq!(json_val["action_id"], "user_select");
    }

    #[test]
    fn test_user_multi_select_basic() {
        let select = UserMultiSelectElement::new("multi_user_select").unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "multi_users_select");
    }

    // Channel select tests
    #[test]
    fn test_channel_select_basic() {
        let select = ChannelSelectElement::new("channel_select").unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "channels_select");
    }

    // Conversation select tests
    #[test]
    fn test_conversation_select_basic() {
        let select = ConversationSelectElement::new("conv_select").unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "conversations_select");
    }

    // External select tests
    #[test]
    fn test_external_select_basic() {
        let select = ExternalDataSelectElement::new("ext_select").unwrap();
        let json_val = serde_json::to_value(&select).unwrap();

        assert_eq!(json_val["type"], "external_select");
    }

    // Date picker tests
    #[test]
    fn test_date_picker_basic() {
        let picker = DatePickerElement::new("date_picker").unwrap();
        let json_val = serde_json::to_value(&picker).unwrap();

        assert_eq!(json_val["type"], "datepicker");
        assert_eq!(json_val["action_id"], "date_picker");
    }

    #[test]
    fn test_date_picker_with_initial_date() {
        let picker = DatePickerElement::new("date_picker")
            .unwrap()
            .with_initial_date("2023-12-25")
            .unwrap();

        let json_val = serde_json::to_value(&picker).unwrap();
        assert_eq!(json_val["initial_date"], "2023-12-25");
    }

    #[test]
    fn test_date_picker_date_validation() {
        let picker = DatePickerElement::new("date_picker").unwrap();
        let result = picker.with_initial_date("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_date_picker_round_trip() {
        let input = json!({
            "type": "datepicker",
            "action_id": "dp",
            "initial_date": "2023-01-01"
        });

        let picker: DatePickerElement = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&picker).unwrap();

        assert_eq!(input, output);
    }

    // Time picker tests
    #[test]
    fn test_time_picker_basic() {
        let picker = TimePickerElement::new("time_picker").unwrap();
        let json_val = serde_json::to_value(&picker).unwrap();

        assert_eq!(json_val["type"], "timepicker");
    }

    #[test]
    fn test_time_picker_with_initial_time() {
        let picker = TimePickerElement::new("time_picker")
            .unwrap()
            .with_initial_time("14:30")
            .unwrap();

        let json_val = serde_json::to_value(&picker).unwrap();
        assert_eq!(json_val["initial_time"], "14:30");
    }

    #[test]
    fn test_time_picker_time_validation() {
        let picker = TimePickerElement::new("time_picker").unwrap();
        let result = picker.with_initial_time("invalid");
        assert!(result.is_err());
    }

    // DateTime picker tests
    #[test]
    fn test_datetime_picker_basic() {
        let picker = DateTimePickerElement::new("datetime_picker").unwrap();
        let json_val = serde_json::to_value(&picker).unwrap();

        assert_eq!(json_val["type"], "datetimepicker");
    }

    #[test]
    fn test_datetime_picker_with_timestamp() {
        let picker = DateTimePickerElement::new("datetime_picker")
            .unwrap()
            .with_initial_date_time(1640444400);

        let json_val = serde_json::to_value(&picker).unwrap();
        assert_eq!(json_val["initial_date_time"], 1640444400);
    }

    // Checkboxes tests
    #[test]
    fn test_checkboxes_basic() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let checkboxes = CheckboxesElement::new("checkboxes", options).unwrap();
        let json_val = serde_json::to_value(&checkboxes).unwrap();

        assert_eq!(json_val["type"], "checkboxes");
        assert_eq!(json_val["options"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_checkboxes_max_options_validation() {
        let options: Vec<SlackOption> = (0..11)
            .map(|i| SlackOption::new(format!("Opt {}", i), format!("opt{}", i)).unwrap())
            .collect();
        let result = CheckboxesElement::new("checkboxes", options);
        assert!(result.is_err());
    }

    #[test]
    fn test_checkboxes_with_initial_options() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let initial = vec![SlackOption::new("Opt 1", "opt1").unwrap()];
        let checkboxes = CheckboxesElement::new("checkboxes", options)
            .unwrap()
            .with_initial_options(initial);

        let json_val = serde_json::to_value(&checkboxes).unwrap();
        assert_eq!(json_val["initial_options"].as_array().unwrap().len(), 1);
    }

    // Radio buttons tests
    #[test]
    fn test_radio_buttons_basic() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let radio = RadioButtonsElement::new("radio", options).unwrap();
        let json_val = serde_json::to_value(&radio).unwrap();

        assert_eq!(json_val["type"], "radio_buttons");
        assert_eq!(json_val["options"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_radio_buttons_max_options_validation() {
        let options: Vec<SlackOption> = (0..11)
            .map(|i| SlackOption::new(format!("Opt {}", i), format!("opt{}", i)).unwrap())
            .collect();
        let result = RadioButtonsElement::new("radio", options);
        assert!(result.is_err());
    }

    #[test]
    fn test_radio_buttons_with_initial_option() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let initial = SlackOption::new("Opt 1", "opt1").unwrap();
        let radio = RadioButtonsElement::new("radio", options)
            .unwrap()
            .with_initial_option(initial);

        let json_val = serde_json::to_value(&radio).unwrap();
        assert!(json_val["initial_option"].is_object());
    }

    // Overflow menu tests
    #[test]
    fn test_overflow_menu_basic() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let overflow = OverflowMenuElement::new("overflow", options).unwrap();
        let json_val = serde_json::to_value(&overflow).unwrap();

        assert_eq!(json_val["type"], "overflow");
        assert_eq!(json_val["options"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_overflow_menu_min_options_validation() {
        let options = vec![SlackOption::new("Opt 1", "opt1").unwrap()];
        let result = OverflowMenuElement::new("overflow", options);
        assert!(result.is_err());
    }

    #[test]
    fn test_overflow_menu_max_options_validation() {
        let options: Vec<SlackOption> = (0..6)
            .map(|i| SlackOption::new(format!("Opt {}", i), format!("opt{}", i)).unwrap())
            .collect();
        let result = OverflowMenuElement::new("overflow", options);
        assert!(result.is_err());
    }

    #[test]
    fn test_overflow_menu_exactly_5_options_ok() {
        let options: Vec<SlackOption> = (0..5)
            .map(|i| SlackOption::new(format!("Opt {}", i), format!("opt{}", i)).unwrap())
            .collect();
        let result = OverflowMenuElement::new("overflow", options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_overflow_menu_with_confirm() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let confirm = ConfirmObject::new("Sure?", "This is important").unwrap();
        let overflow = OverflowMenuElement::new("overflow", options)
            .unwrap()
            .with_confirm(confirm);

        assert!(overflow.confirm.is_some());
    }

    #[test]
    fn test_user_select_with_placeholder() {
        let select = UserSelectElement::new("user_select")
            .unwrap()
            .with_placeholder("Choose user")
            .unwrap();
        assert!(select.placeholder.is_some());
    }

    #[test]
    fn test_user_select_with_initial_user() {
        let select = UserSelectElement::new("user_select")
            .unwrap()
            .with_initial_user("U12345");
        assert_eq!(select.initial_user, Some("U12345".to_string()));
    }

    #[test]
    fn test_static_select_with_initial_option() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let initial = SlackOption::new("Opt 1", "opt1").unwrap();
        let select = StaticSelectElement::new("select", options)
            .unwrap()
            .with_initial_option(initial);
        assert!(select.initial_option.is_some());
    }

    #[test]
    fn test_static_select_with_confirm() {
        let options = vec![SlackOption::new("Opt", "opt").unwrap()];
        let confirm = ConfirmObject::new("Sure?", "Confirm this").unwrap();
        let select = StaticSelectElement::new("select", options)
            .unwrap()
            .with_confirm(confirm);
        assert!(select.confirm.is_some());
    }

    #[test]
    fn test_static_multi_select_with_initial_options() {
        let options = vec![
            SlackOption::new("Opt 1", "opt1").unwrap(),
            SlackOption::new("Opt 2", "opt2").unwrap(),
        ];
        let initial = vec![SlackOption::new("Opt 1", "opt1").unwrap()];
        let select = StaticMultiSelectElement::new("select", options)
            .unwrap()
            .with_initial_options(initial)
            .unwrap();
        assert!(select.initial_options.is_some());
    }

    #[test]
    fn test_static_multi_select_with_placeholder() {
        let options = vec![SlackOption::new("Opt", "opt").unwrap()];
        let select = StaticMultiSelectElement::new("select", options)
            .unwrap()
            .with_placeholder("Choose")
            .unwrap();
        assert!(select.placeholder.is_some());
    }

    #[test]
    fn test_button_with_accessibility_label() {
        let button = ButtonElement::new("Click", "btn")
            .unwrap()
            .with_accessibility_label("Click button");
        assert_eq!(
            button.accessibility_label,
            Some("Click button".to_string())
        );
    }

    #[test]
    fn test_button_with_confirm() {
        let confirm = ConfirmObject::new("Sure?", "Are you sure?").unwrap();
        let button = ButtonElement::new("Click", "btn")
            .unwrap()
            .with_confirm(confirm);
        assert!(button.confirm.is_some());
    }

    #[test]
    fn test_image_element_from_slack_file() {
        let slack_file = serde_json::json!({"id": "F123"});
        let image = ImageElement::from_slack_file(slack_file.clone(), "Alt").unwrap();
        assert_eq!(image.slack_file, Some(slack_file));
        assert_eq!(image.image_url, None);
    }

    #[test]
    fn test_plain_text_input_with_initial_value() {
        let input = PlainTextInputElement::new("input")
            .unwrap()
            .with_initial_value("initial")
            .unwrap();
        assert_eq!(input.initial_value, Some("initial".to_string()));
    }

    #[test]
    fn test_plain_text_input_with_focus_on_load() {
        let input = PlainTextInputElement::new("input")
            .unwrap()
            .with_focus_on_load(true);
        assert_eq!(input.focus_on_load, Some(true));
    }

    #[test]
    fn test_static_select_with_option_groups() {
        let group = OptionGroup::new(
            "Group",
            vec![SlackOption::new("Opt", "opt").unwrap()],
        )
        .unwrap();
        let select = StaticSelectElement::with_option_groups("select", vec![group]).unwrap();
        assert!(select.option_groups.is_some());
        assert!(select.options.is_none());
    }

    #[test]
    fn test_checkboxes_with_confirm() {
        let options = vec![SlackOption::new("Opt", "opt").unwrap()];
        let confirm = ConfirmObject::new("Sure?", "Confirm").unwrap();
        let mut checkboxes = CheckboxesElement::new("checkboxes", options).unwrap();
        checkboxes.confirm = Some(confirm);
        assert!(checkboxes.confirm.is_some());
    }

    #[test]
    fn test_radio_buttons_with_confirm() {
        let options = vec![SlackOption::new("Opt", "opt").unwrap()];
        let confirm = ConfirmObject::new("Sure?", "Confirm").unwrap();
        let mut radio = RadioButtonsElement::new("radio", options).unwrap();
        radio.confirm = Some(confirm);
        assert!(radio.confirm.is_some());
    }

    #[test]
    fn test_date_picker_with_confirm() {
        let confirm = ConfirmObject::new("Sure?", "Confirm").unwrap();
        let mut picker = DatePickerElement::new("date").unwrap();
        picker.confirm = Some(confirm);
        assert!(picker.confirm.is_some());
    }

    #[test]
    fn test_date_picker_with_placeholder() {
        let mut picker = DatePickerElement::new("date").unwrap();
        picker.placeholder = Some(TextObject::plain("Choose date").unwrap());
        assert!(picker.placeholder.is_some());
    }

    #[test]
    fn test_time_picker_with_confirm() {
        let confirm = ConfirmObject::new("Sure?", "Confirm").unwrap();
        let mut picker = TimePickerElement::new("time").unwrap();
        picker.confirm = Some(confirm);
        assert!(picker.confirm.is_some());
    }

    #[test]
    fn test_time_picker_with_placeholder() {
        let mut picker = TimePickerElement::new("time").unwrap();
        picker.placeholder = Some(TextObject::plain("Choose time").unwrap());
        assert!(picker.placeholder.is_some());
    }

    #[test]
    fn test_datetime_picker_with_confirm() {
        let confirm = ConfirmObject::new("Sure?", "Confirm").unwrap();
        let mut picker = DateTimePickerElement::new("datetime").unwrap();
        picker.confirm = Some(confirm);
        assert!(picker.confirm.is_some());
    }
}
