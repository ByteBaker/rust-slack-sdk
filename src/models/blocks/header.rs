//! Header block implementation.

use crate::constants::limits::MAX_HEADER_TEXT_LENGTH;
use crate::error::{Result, SlackError};
use crate::models::objects::TextObject;
use serde::{Deserialize, Serialize};

/// A header block for displaying prominent text.
///
/// Header blocks can only contain plain text (not markdown).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderBlock {
    /// The type of block (always "header").
    #[serde(rename = "type")]
    pub block_type: String,

    /// The text for the header (plain text only, max 150 characters).
    pub text: TextObject,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl HeaderBlock {
    /// Creates a new header block.
    ///
    /// # Arguments
    /// * `text` - The header text (max 150 characters, plain text only)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Text exceeds 150 characters
    /// - Text is not plain text
    pub fn new(text: impl Into<String>) -> Result<Self> {
        let text_str = text.into();

        if text_str.len() > MAX_HEADER_TEXT_LENGTH {
            return Err(SlackError::Validation(format!(
                "Header text length {} exceeds maximum {}",
                text_str.len(),
                MAX_HEADER_TEXT_LENGTH
            )));
        }

        Ok(Self {
            block_type: "header".to_string(),
            text: TextObject::plain(text_str)?,
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
    fn test_header_block_basic() {
        let block = HeaderBlock::new("Budget Performance").unwrap();
        let json_val = serde_json::to_value(&block).unwrap();

        assert_eq!(
            json_val,
            json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": "Budget Performance"
                }
            })
        );
    }

    #[test]
    fn test_header_block_with_id() {
        let block = HeaderBlock::new("Header")
            .unwrap()
            .with_block_id("header_1");

        let json_val = serde_json::to_value(&block).unwrap();
        assert_eq!(json_val["block_id"], "header_1");
    }

    #[test]
    fn test_header_block_round_trip() {
        let input = json!({
            "type": "header",
            "text": {"type": "plain_text", "text": "Test Header"},
            "block_id": "h1"
        });

        let block: HeaderBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_header_text_length_validation() {
        let long_text = "a".repeat(151);
        let result = HeaderBlock::new(&long_text);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::Validation(_)));
    }

    #[test]
    fn test_header_max_length_ok() {
        let max_text = "a".repeat(150);
        let result = HeaderBlock::new(&max_text);

        assert!(result.is_ok());
    }

    #[test]
    fn test_header_block_clone() {
        let block1 = HeaderBlock::new("Test").unwrap();
        let block2 = block1.clone();

        assert_eq!(block1, block2);
    }
}
