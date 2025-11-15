//! Input block implementation.

use crate::error::{Result, SlackError};
use crate::models::objects::TextObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum length for input labels (2000 characters).
pub const MAX_LABEL_LENGTH: usize = 2000;

/// Maximum length for hints (2000 characters).
pub const MAX_HINT_LENGTH: usize = 2000;

/// An input block for collecting user input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputBlock {
    /// The type of block (always "input").
    #[serde(rename = "type")]
    pub block_type: String,

    /// A label that appears above the input element (required, max 2000 characters).
    pub label: TextObject,

    /// An interactive input element (plain_text_input, select menus, etc.).
    pub element: Value,

    /// Whether the input element may be empty when submitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,

    /// An optional hint that appears below the input element (max 2000 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<TextObject>,

    /// Whether to dispatch a block_actions payload when the element is interacted with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatch_action: Option<bool>,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl InputBlock {
    /// Creates a new input block.
    ///
    /// # Arguments
    /// * `label` - The label text (max 2000 characters)
    /// * `element` - The input element
    pub fn new(label: impl Into<String>, element: Value) -> Result<Self> {
        let label_str = label.into();

        if label_str.len() > MAX_LABEL_LENGTH {
            return Err(SlackError::Validation(format!(
                "Input label length {} exceeds maximum {}",
                label_str.len(),
                MAX_LABEL_LENGTH
            )));
        }

        Ok(Self {
            block_type: "input".to_string(),
            label: TextObject::plain(label_str)?,
            element,
            optional: None,
            hint: None,
            dispatch_action: None,
            block_id: None,
        })
    }

    /// Sets whether the input is optional.
    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = Some(optional);
        self
    }

    /// Sets the hint text.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Result<Self> {
        let hint_str = hint.into();
        if hint_str.len() > MAX_HINT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Input hint length {} exceeds maximum {}",
                hint_str.len(),
                MAX_HINT_LENGTH
            )));
        }
        self.hint = Some(TextObject::plain(hint_str)?);
        Ok(self)
    }

    /// Sets whether to dispatch actions.
    pub fn with_dispatch_action(mut self, dispatch: bool) -> Self {
        self.dispatch_action = Some(dispatch);
        self
    }

    /// Sets the block ID.
    pub fn with_block_id(mut self, block_id: impl Into<String>) -> Self {
        self.block_id = Some(block_id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_input_block_basic() {
        let element = json!({
            "type": "plain_text_input",
            "action_id": "input_1"
        });

        let block = InputBlock::new("Label", element).unwrap();

        assert_eq!(block.block_type, "input");
        assert_eq!(block.label.text(), "Label");
    }

    #[test]
    fn test_input_block_serialization() {
        let input = json!({
            "type": "input",
            "label": {
                "type": "plain_text",
                "text": "Label"
            },
            "element": {
                "type": "plain_text_input",
                "action_id": "input_1"
            }
        });

        let block: InputBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_input_block_with_optional() {
        let element = json!({"type": "plain_text_input", "action_id": "input_1"});
        let block = InputBlock::new("Label", element)
            .unwrap()
            .with_optional(true);

        let json_val = serde_json::to_value(&block).unwrap();
        assert_eq!(json_val["optional"], true);
    }

    #[test]
    fn test_input_block_with_hint() {
        let element = json!({"type": "plain_text_input", "action_id": "input_1"});
        let block = InputBlock::new("Label", element)
            .unwrap()
            .with_hint("This is a hint")
            .unwrap();

        let json_val = serde_json::to_value(&block).unwrap();
        assert_eq!(json_val["hint"]["text"], "This is a hint");
    }

    #[test]
    fn test_input_block_with_dispatch_action() {
        let element = json!({"type": "plain_text_input", "action_id": "input_1"});
        let block = InputBlock::new("Label", element)
            .unwrap()
            .with_dispatch_action(true);

        let json_val = serde_json::to_value(&block).unwrap();
        assert_eq!(json_val["dispatch_action"], true);
    }

    #[test]
    fn test_input_block_with_block_id() {
        let element = json!({"type": "plain_text_input", "action_id": "input_1"});
        let block = InputBlock::new("Label", element)
            .unwrap()
            .with_block_id("input_block_1");

        assert_eq!(block.block_id, Some("input_block_1".to_string()));
    }

    #[test]
    fn test_input_block_label_length_validation() {
        let long_label = "a".repeat(2001);
        let element = json!({"type": "plain_text_input", "action_id": "input_1"});
        let result = InputBlock::new(&long_label, element);
        assert!(result.is_err());
    }

    #[test]
    fn test_input_block_hint_length_validation() {
        let element = json!({"type": "plain_text_input", "action_id": "input_1"});
        let block = InputBlock::new("Label", element).unwrap();
        let long_hint = "a".repeat(2001);
        let result = block.with_hint(&long_hint);
        assert!(result.is_err());
    }

    #[test]
    fn test_input_block_round_trip() {
        let input = json!({
            "type": "input",
            "label": {"type": "plain_text", "text": "Label"},
            "element": {"type": "plain_text_input", "action_id": "input_1"},
            "optional": true,
            "hint": {"type": "plain_text", "text": "Hint"},
            "block_id": "input_1"
        });

        let block: InputBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }
}
