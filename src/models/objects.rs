//! Composition objects for Block Kit.
//!
//! This module contains the fundamental building blocks used throughout
//! the Block Kit API, including text objects, options, and confirmation dialogs.

use crate::error::{Result, SlackError};
use serde::{Deserialize, Serialize};

/// Maximum text length for most text objects (3000 characters).
pub const MAX_TEXT_LENGTH: usize = 3000;

/// Maximum length for option labels (75 characters).
pub const MAX_OPTION_LABEL_LENGTH: usize = 75;

/// Maximum length for option values (75 characters).
pub const MAX_OPTION_VALUE_LENGTH: usize = 75;

/// Maximum length for confirm dialog titles (100 characters).
pub const MAX_CONFIRM_TITLE_LENGTH: usize = 100;

/// Maximum length for confirm dialog text (300 characters).
pub const MAX_CONFIRM_TEXT_LENGTH: usize = 300;

/// A text object that can be either plain text or markdown.
///
/// This is used in many places throughout the Block Kit API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextObject {
    /// Plain text with optional emoji support.
    #[serde(rename = "plain_text")]
    PlainText {
        /// The text content.
        text: String,

        /// Whether to escape emoji. Defaults to true.
        #[serde(skip_serializing_if = "Option::is_none")]
        emoji: Option<bool>,
    },

    /// Markdown-formatted text.
    #[serde(rename = "mrkdwn")]
    Markdown {
        /// The markdown text content.
        text: String,

        /// When set to false, URLs, user mentions, and channel mentions will be auto-linked.
        #[serde(skip_serializing_if = "Option::is_none")]
        verbatim: Option<bool>,
    },
}

impl TextObject {
    /// Creates a new plain text object.
    ///
    /// # Arguments
    /// * `text` - The text content (max 3000 characters)
    ///
    /// # Errors
    /// Returns an error if the text exceeds maximum length.
    pub fn plain(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.len() > MAX_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Text length {} exceeds maximum {}",
                text.len(),
                MAX_TEXT_LENGTH
            )));
        }

        Ok(Self::PlainText { text, emoji: None })
    }

    /// Creates a new plain text object with emoji setting.
    pub fn plain_with_emoji(text: impl Into<String>, emoji: bool) -> Result<Self> {
        let text = text.into();
        if text.len() > MAX_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Text length {} exceeds maximum {}",
                text.len(),
                MAX_TEXT_LENGTH
            )));
        }

        Ok(Self::PlainText {
            text,
            emoji: Some(emoji),
        })
    }

    /// Creates a new markdown text object.
    ///
    /// # Arguments
    /// * `text` - The markdown text content (max 3000 characters)
    pub fn markdown(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.len() > MAX_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Text length {} exceeds maximum {}",
                text.len(),
                MAX_TEXT_LENGTH
            )));
        }

        Ok(Self::Markdown {
            text,
            verbatim: None,
        })
    }

    /// Creates a new markdown text object with verbatim setting.
    pub fn markdown_with_verbatim(text: impl Into<String>, verbatim: bool) -> Result<Self> {
        let text = text.into();
        if text.len() > MAX_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Text length {} exceeds maximum {}",
                text.len(),
                MAX_TEXT_LENGTH
            )));
        }

        Ok(Self::Markdown {
            text,
            verbatim: Some(verbatim),
        })
    }

    /// Gets the text content, regardless of type.
    pub fn text(&self) -> &str {
        match self {
            Self::PlainText { text, .. } => text,
            Self::Markdown { text, .. } => text,
        }
    }
}

/// An option for select menus and other choice-based elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlackOption {
    /// A text object that defines the text shown in the option.
    pub text: TextObject,

    /// The string value that will be passed when this option is chosen.
    pub value: String,

    /// A plain text description of the option (for radio buttons and checkboxes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<TextObject>,

    /// A URL to load in the user's browser when the option is clicked (for overflow menus).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl SlackOption {
    /// Creates a new option with plain text label.
    ///
    /// # Arguments
    /// * `label` - The text label (max 75 characters)
    /// * `value` - The option value (max 75 characters)
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Result<Self> {
        let label = label.into();
        let value = value.into();

        if label.len() > MAX_OPTION_LABEL_LENGTH {
            return Err(SlackError::Validation(format!(
                "Option label length {} exceeds maximum {}",
                label.len(),
                MAX_OPTION_LABEL_LENGTH
            )));
        }

        if value.len() > MAX_OPTION_VALUE_LENGTH {
            return Err(SlackError::Validation(format!(
                "Option value length {} exceeds maximum {}",
                value.len(),
                MAX_OPTION_VALUE_LENGTH
            )));
        }

        Ok(Self {
            text: TextObject::plain(label)?,
            value,
            description: None,
            url: None,
        })
    }

    /// Creates an option where the label and value are the same.
    ///
    /// This is a convenience method for simple options.
    pub fn from_single_value(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        Self::new(value.clone(), value)
    }

    /// Sets the description for this option.
    pub fn with_description(mut self, description: impl Into<String>) -> Result<Self> {
        self.description = Some(TextObject::plain(description)?);
        Ok(self)
    }

    /// Sets the URL for this option.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

/// A group of options with a label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptionGroup {
    /// A plain text label for the group.
    pub label: TextObject,

    /// The options in this group (max 100).
    pub options: Vec<SlackOption>,
}

impl OptionGroup {
    /// Creates a new option group.
    ///
    /// # Arguments
    /// * `label` - The group label
    /// * `options` - The options in this group (max 100)
    pub fn new(label: impl Into<String>, options: Vec<SlackOption>) -> Result<Self> {
        if options.len() > 100 {
            return Err(SlackError::Validation(format!(
                "Option group has {} options, maximum is 100",
                options.len()
            )));
        }

        Ok(Self {
            label: TextObject::plain(label)?,
            options,
        })
    }
}

/// A confirmation dialog for dangerous or irreversible actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmObject {
    /// The title of the confirmation dialog (max 100 characters, plain text only).
    pub title: TextObject,

    /// The explanatory text in the dialog (max 300 characters).
    pub text: TextObject,

    /// The text of the confirm button (max 24 characters, defaults to "Confirm").
    pub confirm: TextObject,

    /// The text of the deny button (max 24 characters, defaults to "Cancel").
    pub deny: TextObject,

    /// The style of the confirm button ("primary" or "danger").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ConfirmStyle>,
}

/// The style for a confirmation button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmStyle {
    /// A primary (green) button.
    Primary,

    /// A danger (red) button.
    Danger,
}

impl ConfirmObject {
    /// Creates a new confirmation dialog.
    ///
    /// # Arguments
    /// * `title` - The dialog title (max 100 characters)
    /// * `text` - The explanatory text (max 300 characters)
    pub fn new(title: impl Into<String>, text: impl Into<String>) -> Result<Self> {
        let title_str = title.into();
        let text_str = text.into();

        if title_str.len() > MAX_CONFIRM_TITLE_LENGTH {
            return Err(SlackError::Validation(format!(
                "Confirm title length {} exceeds maximum {}",
                title_str.len(),
                MAX_CONFIRM_TITLE_LENGTH
            )));
        }

        if text_str.len() > MAX_CONFIRM_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Confirm text length {} exceeds maximum {}",
                text_str.len(),
                MAX_CONFIRM_TEXT_LENGTH
            )));
        }

        Ok(Self {
            title: TextObject::plain(title_str)?,
            text: TextObject::plain(text_str)?,
            confirm: TextObject::plain("Confirm")?,
            deny: TextObject::plain("Cancel")?,
            style: None,
        })
    }

    /// Sets the confirm button text.
    pub fn with_confirm(mut self, text: impl Into<String>) -> Result<Self> {
        self.confirm = TextObject::plain(text)?;
        Ok(self)
    }

    /// Sets the deny button text.
    pub fn with_deny(mut self, text: impl Into<String>) -> Result<Self> {
        self.deny = TextObject::plain(text)?;
        Ok(self)
    }

    /// Sets the confirm button style.
    pub fn with_style(mut self, style: ConfirmStyle) -> Self {
        self.style = Some(style);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // Text Object Tests

    #[test]
    fn test_plain_text_object_basic() {
        let obj = TextObject::plain("some text").unwrap();
        let json_val = serde_json::to_value(&obj).unwrap();

        assert_eq!(
            json_val,
            json!({
                "text": "some text",
                "type": "plain_text"
            })
        );
    }

    #[test]
    fn test_plain_text_object_with_emoji() {
        let obj = TextObject::plain_with_emoji("some text", false).unwrap();
        let json_val = serde_json::to_value(&obj).unwrap();

        assert_eq!(
            json_val,
            json!({
                "text": "some text",
                "type": "plain_text",
                "emoji": false
            })
        );
    }

    #[test]
    fn test_plain_text_object_emoji_true_omitted() {
        let obj = TextObject::plain_with_emoji("text", true).unwrap();
        let json_str = serde_json::to_string(&obj).unwrap();

        // emoji: true should be serialized (Python SDK includes it)
        assert!(json_str.contains("\"emoji\":true"));
    }

    #[test]
    fn test_markdown_text_object_basic() {
        let obj = TextObject::markdown("some text").unwrap();
        let json_val = serde_json::to_value(&obj).unwrap();

        assert_eq!(
            json_val,
            json!({
                "text": "some text",
                "type": "mrkdwn"
            })
        );
    }

    #[test]
    fn test_markdown_text_object_with_verbatim() {
        let obj = TextObject::markdown_with_verbatim("some text", true).unwrap();
        let json_val = serde_json::to_value(&obj).unwrap();

        assert_eq!(
            json_val,
            json!({
                "text": "some text",
                "type": "mrkdwn",
                "verbatim": true
            })
        );
    }

    #[test]
    fn test_text_object_deserialization_plain() {
        let input = json!({
            "type": "plain_text",
            "text": "hello world",
            "emoji": false
        });

        let obj: TextObject = serde_json::from_value(input).unwrap();

        match obj {
            TextObject::PlainText { text, emoji } => {
                assert_eq!(text, "hello world");
                assert_eq!(emoji, Some(false));
            }
            _ => panic!("Expected PlainText variant"),
        }
    }

    #[test]
    fn test_text_object_deserialization_markdown() {
        let input = json!({
            "type": "mrkdwn",
            "text": "*bold* text",
            "verbatim": true
        });

        let obj: TextObject = serde_json::from_value(input).unwrap();

        match obj {
            TextObject::Markdown { text, verbatim } => {
                assert_eq!(text, "*bold* text");
                assert_eq!(verbatim, Some(true));
            }
            _ => panic!("Expected Markdown variant"),
        }
    }

    #[test]
    fn test_text_object_round_trip() {
        let input = json!({
            "type": "plain_text",
            "text": "test",
            "emoji": false
        });

        let obj: TextObject = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&obj).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_text_object_text_getter() {
        let plain = TextObject::plain("plain text").unwrap();
        let markdown = TextObject::markdown("markdown text").unwrap();

        assert_eq!(plain.text(), "plain text");
        assert_eq!(markdown.text(), "markdown text");
    }

    #[test]
    fn test_text_object_validation_length() {
        let long_text = "a".repeat(3001);
        let result = TextObject::plain(&long_text);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::Validation(_)));
    }

    #[test]
    fn test_text_object_max_length_ok() {
        let max_text = "a".repeat(3000);
        let result = TextObject::plain(&max_text);

        assert!(result.is_ok());
    }

    // Option Tests

    #[test]
    fn test_option_basic() {
        let option = SlackOption::new("an option", "option_1").unwrap();

        let json_val = serde_json::to_value(&option).unwrap();
        assert_eq!(
            json_val,
            json!({
                "text": {
                    "type": "plain_text",
                    "text": "an option"
                },
                "value": "option_1"
            })
        );
    }

    #[test]
    fn test_option_from_single_value() {
        let option = SlackOption::from_single_value("option_1").unwrap();

        assert_eq!(option.text.text(), "option_1");
        assert_eq!(option.value, "option_1");
    }

    #[test]
    fn test_option_round_trip() {
        let input = json!({
            "text": {
                "type": "plain_text",
                "text": "Option 1"
            },
            "value": "opt1"
        });

        let option: SlackOption = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&option).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_option_with_description() {
        let option = SlackOption::new("label", "value")
            .unwrap()
            .with_description("this is an option")
            .unwrap();

        let json_val = serde_json::to_value(&option).unwrap();
        assert_eq!(
            json_val,
            json!({
                "text": {
                    "type": "plain_text",
                    "text": "label"
                },
                "value": "value",
                "description": {
                    "type": "plain_text",
                    "text": "this is an option"
                }
            })
        );
    }

    #[test]
    fn test_option_with_url() {
        let option = SlackOption::new("Click here", "click")
            .unwrap()
            .with_url("https://example.com");

        let json_val = serde_json::to_value(&option).unwrap();
        assert!(json_val["url"] == "https://example.com");
    }

    #[test]
    fn test_option_label_length_validation() {
        let long_label = "a".repeat(76);
        let result = SlackOption::new(&long_label, "value");

        assert!(result.is_err());
    }

    #[test]
    fn test_option_value_length_validation() {
        let long_value = "a".repeat(76);
        let result = SlackOption::new("label", &long_value);

        assert!(result.is_err());
    }

    #[test]
    fn test_option_max_lengths_ok() {
        let max_label = "a".repeat(75);
        let max_value = "b".repeat(75);
        let result = SlackOption::new(&max_label, &max_value);

        assert!(result.is_ok());
    }

    // OptionGroup Tests

    #[test]
    fn test_option_group_basic() {
        let options = vec![
            SlackOption::from_single_value("one").unwrap(),
            SlackOption::from_single_value("two").unwrap(),
        ];

        let group = OptionGroup::new("a group", options).unwrap();

        let json_val = serde_json::to_value(&group).unwrap();
        assert_eq!(
            json_val,
            json!({
                "label": {
                    "type": "plain_text",
                    "text": "a group"
                },
                "options": [
                    {
                        "text": {"type": "plain_text", "text": "one"},
                        "value": "one"
                    },
                    {
                        "text": {"type": "plain_text", "text": "two"},
                        "value": "two"
                    }
                ]
            })
        );
    }

    #[test]
    fn test_option_group_round_trip() {
        let input = json!({
            "label": {"type": "plain_text", "text": "Group"},
            "options": [
                {"text": {"type": "plain_text", "text": "Opt 1"}, "value": "opt1"}
            ]
        });

        let group: OptionGroup = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&group).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_option_group_max_options_validation() {
        let options: Vec<SlackOption> = (0..101)
            .map(|i| SlackOption::from_single_value(format!("opt{}", i)).unwrap())
            .collect();

        let result = OptionGroup::new("too many", options);

        assert!(result.is_err());
    }

    #[test]
    fn test_option_group_100_options_ok() {
        let options: Vec<SlackOption> = (0..100)
            .map(|i| SlackOption::from_single_value(format!("opt{}", i)).unwrap())
            .collect();

        let result = OptionGroup::new("exactly 100", options);

        assert!(result.is_ok());
    }

    // ConfirmObject Tests

    #[test]
    fn test_confirm_object_basic() {
        let confirm = ConfirmObject::new("Are you sure?", "This action is irreversible").unwrap();

        let json_val = serde_json::to_value(&confirm).unwrap();
        assert_eq!(
            json_val,
            json!({
                "title": {
                    "type": "plain_text",
                    "text": "Are you sure?"
                },
                "text": {
                    "type": "plain_text",
                    "text": "This action is irreversible"
                },
                "confirm": {
                    "type": "plain_text",
                    "text": "Confirm"
                },
                "deny": {
                    "type": "plain_text",
                    "text": "Cancel"
                }
            })
        );
    }

    #[test]
    fn test_confirm_object_custom_buttons() {
        let confirm = ConfirmObject::new("Title", "Text")
            .unwrap()
            .with_confirm("Do it!")
            .unwrap()
            .with_deny("No way")
            .unwrap();

        let json_val = serde_json::to_value(&confirm).unwrap();
        assert_eq!(json_val["confirm"]["text"], "Do it!");
        assert_eq!(json_val["deny"]["text"], "No way");
    }

    #[test]
    fn test_confirm_object_with_style() {
        let confirm = ConfirmObject::new("Delete?", "This will delete everything")
            .unwrap()
            .with_style(ConfirmStyle::Danger);

        let json_val = serde_json::to_value(&confirm).unwrap();
        assert_eq!(json_val["style"], "danger");
    }

    #[test]
    fn test_confirm_object_title_length_validation() {
        let long_title = "a".repeat(101);
        let result = ConfirmObject::new(&long_title, "text");

        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_object_text_length_validation() {
        let long_text = "a".repeat(301);
        let result = ConfirmObject::new("title", &long_text);

        assert!(result.is_err());
    }

    #[test]
    fn test_confirm_object_max_lengths_ok() {
        let max_title = "a".repeat(100);
        let max_text = "b".repeat(300);
        let result = ConfirmObject::new(&max_title, &max_text);

        assert!(result.is_ok());
    }

    #[test]
    fn test_confirm_object_round_trip() {
        let input = json!({
            "title": {"type": "plain_text", "text": "Title"},
            "text": {"type": "plain_text", "text": "Text"},
            "confirm": {"type": "plain_text", "text": "Yes"},
            "deny": {"type": "plain_text", "text": "No"},
            "style": "danger"
        });

        let confirm: ConfirmObject = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&confirm).unwrap();

        assert_eq!(input, output);
    }
}
