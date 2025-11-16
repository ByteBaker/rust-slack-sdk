//! Actions block implementation.

use crate::constants::limits::MAX_ACTIONS_ELEMENTS;
use crate::error::{Result, SlackError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An actions block for holding interactive elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionsBlock {
    /// The type of block (always "actions").
    #[serde(rename = "type")]
    pub block_type: String,

    /// An array of interactive element objects (max 25).
    pub elements: Vec<Value>,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl ActionsBlock {
    /// Creates a new actions block.
    ///
    /// # Arguments
    /// * `elements` - Array of interactive elements (max 25)
    pub fn new(elements: Vec<Value>) -> Result<Self> {
        if elements.is_empty() {
            return Err(SlackError::Validation(
                "ActionsBlock must have at least one element".to_string(),
            ));
        }

        if elements.len() > MAX_ACTIONS_ELEMENTS {
            return Err(SlackError::Validation(format!(
                "ActionsBlock has {} elements, maximum is {}",
                elements.len(),
                MAX_ACTIONS_ELEMENTS
            )));
        }

        Ok(Self {
            block_type: "actions".to_string(),
            elements,
            block_id: None,
        })
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
    fn test_actions_block_basic() {
        let elements = vec![json!({
            "type": "button",
            "text": {"type": "plain_text", "text": "Click"},
            "action_id": "btn_1"
        })];

        let block = ActionsBlock::new(elements).unwrap();

        assert_eq!(block.block_type, "actions");
        assert_eq!(block.elements.len(), 1);
    }

    #[test]
    fn test_actions_block_serialization() {
        let input = json!({
            "type": "actions",
            "elements": [
                {
                    "type": "button",
                    "text": {"type": "plain_text", "text": "Click"},
                    "action_id": "btn_1"
                }
            ]
        });

        let block: ActionsBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_actions_block_with_multiple_elements() {
        let elements = vec![
            json!({"type": "button", "text": {"type": "plain_text", "text": "Button 1"}, "action_id": "btn_1"}),
            json!({"type": "button", "text": {"type": "plain_text", "text": "Button 2"}, "action_id": "btn_2"}),
            json!({"type": "button", "text": {"type": "plain_text", "text": "Button 3"}, "action_id": "btn_3"}),
        ];

        let block = ActionsBlock::new(elements).unwrap();
        assert_eq!(block.elements.len(), 3);
    }

    #[test]
    fn test_actions_block_empty_elements_validation() {
        let elements = vec![];
        let result = ActionsBlock::new(elements);
        assert!(result.is_err());
    }

    #[test]
    fn test_actions_block_max_elements_validation() {
        let elements: Vec<Value> = (0..26)
            .map(|i| {
                json!({
                    "type": "button",
                    "text": {"type": "plain_text", "text": format!("Button {}", i)},
                    "action_id": format!("btn_{}", i)
                })
            })
            .collect();

        let result = ActionsBlock::new(elements);
        assert!(result.is_err());
    }

    #[test]
    fn test_actions_block_exactly_25_elements_ok() {
        let elements: Vec<Value> = (0..25)
            .map(|i| {
                json!({
                    "type": "button",
                    "text": {"type": "plain_text", "text": format!("Button {}", i)},
                    "action_id": format!("btn_{}", i)
                })
            })
            .collect();

        let result = ActionsBlock::new(elements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_actions_block_with_block_id() {
        let elements = vec![json!({
            "type": "button",
            "text": {"type": "plain_text", "text": "Click"},
            "action_id": "btn_1"
        })];

        let block = ActionsBlock::new(elements)
            .unwrap()
            .with_block_id("actions_1");

        assert_eq!(block.block_id, Some("actions_1".to_string()));
    }

    #[test]
    fn test_actions_block_round_trip() {
        let input = json!({
            "type": "actions",
            "block_id": "actions_1",
            "elements": [
                {
                    "type": "button",
                    "text": {"type": "plain_text", "text": "Click"},
                    "action_id": "btn_1"
                }
            ]
        });

        let block: ActionsBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }
}
