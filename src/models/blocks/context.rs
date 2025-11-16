//! Context block implementation.

use crate::constants::limits::MAX_CONTEXT_ELEMENTS;
use crate::error::{Result, SlackError};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A context block for displaying contextual information.
///
/// Context blocks are used to add contextual information, typically shown in a muted color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextBlock {
    /// The type of block (always "context").
    #[serde(rename = "type")]
    pub block_type: String,

    /// An array of image elements and text objects (max 10).
    pub elements: Vec<Value>,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl ContextBlock {
    /// Creates a new context block.
    ///
    /// # Arguments
    /// * `elements` - Array of elements (max 10)
    pub fn new(elements: Vec<Value>) -> Result<Self> {
        if elements.is_empty() {
            return Err(SlackError::Validation(
                "ContextBlock must have at least one element".to_string(),
            ));
        }

        if elements.len() > MAX_CONTEXT_ELEMENTS {
            return Err(SlackError::Validation(format!(
                "ContextBlock has {} elements, maximum is {}",
                elements.len(),
                MAX_CONTEXT_ELEMENTS
            )));
        }

        Ok(Self {
            block_type: "context".to_string(),
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
    fn test_context_block_basic() {
        let elements = vec![json!({
            "type": "mrkdwn",
            "text": "Context information"
        })];

        let block = ContextBlock::new(elements).unwrap();

        assert_eq!(block.block_type, "context");
        assert_eq!(block.elements.len(), 1);
    }

    #[test]
    fn test_context_block_serialization() {
        let input = json!({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": "Context text"
                }
            ]
        });

        let block: ContextBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_context_block_with_multiple_elements() {
        let elements = vec![
            json!({"type": "mrkdwn", "text": "Text 1"}),
            json!({"type": "image", "image_url": "https://example.com/img.png", "alt_text": "img"}),
            json!({"type": "mrkdwn", "text": "Text 2"}),
        ];

        let block = ContextBlock::new(elements).unwrap();
        assert_eq!(block.elements.len(), 3);
    }

    #[test]
    fn test_context_block_empty_elements_validation() {
        let elements = vec![];
        let result = ContextBlock::new(elements);
        assert!(result.is_err());
    }

    #[test]
    fn test_context_block_max_elements_validation() {
        let elements: Vec<Value> = (0..11)
            .map(|i| json!({"type": "mrkdwn", "text": format!("Text {}", i)}))
            .collect();

        let result = ContextBlock::new(elements);
        assert!(result.is_err());
    }

    #[test]
    fn test_context_block_exactly_10_elements_ok() {
        let elements: Vec<Value> = (0..10)
            .map(|i| json!({"type": "mrkdwn", "text": format!("Text {}", i)}))
            .collect();

        let result = ContextBlock::new(elements);
        assert!(result.is_ok());
    }

    #[test]
    fn test_context_block_with_block_id() {
        let elements = vec![json!({"type": "mrkdwn", "text": "Text"})];
        let block = ContextBlock::new(elements)
            .unwrap()
            .with_block_id("context_1");

        assert_eq!(block.block_id, Some("context_1".to_string()));
    }

    #[test]
    fn test_context_block_round_trip() {
        let input = json!({
            "type": "context",
            "block_id": "ctx_1",
            "elements": [
                {"type": "mrkdwn", "text": "Text"}
            ]
        });

        let block: ContextBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }
}
