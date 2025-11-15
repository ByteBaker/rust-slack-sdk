//! Divider block implementation.

use serde::{Deserialize, Serialize};

/// A visual divider to split up blocks.
///
/// This is equivalent to a horizontal rule (`<hr>`) in HTML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DividerBlock {
    /// The type of block (always "divider").
    #[serde(rename = "type")]
    pub block_type: String,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl Default for DividerBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl DividerBlock {
    /// Creates a new divider block.
    pub fn new() -> Self {
        Self {
            block_type: "divider".to_string(),
            block_id: None,
        }
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
    fn test_divider_block_basic() {
        let block = DividerBlock::new();
        let json_val = serde_json::to_value(&block).unwrap();

        assert_eq!(json_val, json!({"type": "divider"}));
    }

    #[test]
    fn test_divider_block_with_id() {
        let block = DividerBlock::new().with_block_id("foo");
        let json_val = serde_json::to_value(&block).unwrap();

        assert_eq!(
            json_val,
            json!({
                "type": "divider",
                "block_id": "foo"
            })
        );
    }

    #[test]
    fn test_divider_block_round_trip() {
        let input = json!({"type": "divider", "block_id": "test_id"});

        let block: DividerBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_divider_block_default() {
        let block = DividerBlock::default();
        assert_eq!(block.block_type, "divider");
        assert!(block.block_id.is_none());
    }

    #[test]
    fn test_divider_block_clone() {
        let block1 = DividerBlock::new().with_block_id("id1");
        let block2 = block1.clone();

        assert_eq!(block1, block2);
    }
}
