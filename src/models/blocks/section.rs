//! Section block implementation.

use crate::error::{Result, SlackError};
use crate::models::objects::TextObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum number of fields in a section block (10).
pub const MAX_SECTION_FIELDS: usize = 10;

/// A section block for displaying text and an optional accessory element.
///
/// One of the most commonly used blocks. Can contain text, fields, and an accessory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionBlock {
    /// The type of block (always "section").
    #[serde(rename = "type")]
    pub block_type: String,

    /// The main text of the section (markdown or plain text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextObject>,

    /// An array of text objects (max 10, markdown or plain text).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<TextObject>>,

    /// An accessory element (button, select, image, datepicker, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessory: Option<Value>,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl SectionBlock {
    /// Creates a new section block with text.
    ///
    /// # Arguments
    /// * `text` - The section text
    pub fn new(text: impl Into<String>) -> Result<Self> {
        Ok(Self {
            block_type: "section".to_string(),
            text: Some(TextObject::markdown(text)?),
            fields: None,
            accessory: None,
            block_id: None,
        })
    }

    /// Creates a new section block builder.
    pub fn builder() -> SectionBlockBuilder {
        SectionBlockBuilder::default()
    }

    /// Validates that the block has either text or fields.
    pub fn validate(&self) -> Result<()> {
        if self.text.is_none() && self.fields.is_none() {
            return Err(SlackError::Validation(
                "SectionBlock must have either text or fields".to_string(),
            ));
        }

        if let Some(fields) = &self.fields {
            if fields.len() > MAX_SECTION_FIELDS {
                return Err(SlackError::Validation(format!(
                    "SectionBlock has {} fields, maximum is {}",
                    fields.len(),
                    MAX_SECTION_FIELDS
                )));
            }
        }

        Ok(())
    }
}

/// Builder for section blocks.
#[derive(Debug, Default)]
pub struct SectionBlockBuilder {
    text: Option<TextObject>,
    fields: Option<Vec<TextObject>>,
    accessory: Option<Value>,
    block_id: Option<String>,
}

impl SectionBlockBuilder {
    /// Sets the text field (markdown).
    pub fn text(mut self, text: impl Into<String>) -> Result<Self> {
        self.text = Some(TextObject::markdown(text)?);
        Ok(self)
    }

    /// Sets the text field with a custom text object.
    pub fn text_object(mut self, text: TextObject) -> Self {
        self.text = Some(text);
        self
    }

    /// Sets the fields array.
    pub fn fields<I, S>(mut self, fields: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let field_objs: Result<Vec<TextObject>> = fields
            .into_iter()
            .map(|s| TextObject::markdown(s))
            .collect();

        self.fields = Some(field_objs?);
        Ok(self)
    }

    /// Sets the fields with custom text objects.
    pub fn field_objects(mut self, fields: Vec<TextObject>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Sets the accessory element.
    pub fn accessory(mut self, accessory: Value) -> Self {
        self.accessory = Some(accessory);
        self
    }

    /// Sets the block ID.
    pub fn block_id(mut self, block_id: impl Into<String>) -> Self {
        self.block_id = Some(block_id.into());
        self
    }

    /// Builds the section block.
    pub fn build(self) -> Result<SectionBlock> {
        let block = SectionBlock {
            block_type: "section".to_string(),
            text: self.text,
            fields: self.fields,
            accessory: self.accessory,
            block_id: self.block_id,
        };

        block.validate()?;
        Ok(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_section_block_with_text() {
        let input = json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": "A message *with some bold text*"
            }
        });

        let block: SectionBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_section_block_new() {
        let block = SectionBlock::new("some text").unwrap();

        assert_eq!(block.block_type, "section");
        assert!(block.text.is_some());
        assert_eq!(block.text.unwrap().text(), "some text");
    }

    #[test]
    fn test_section_block_with_plain_text() {
        let input = json!({
            "type": "section",
            "text": {
                "type": "plain_text",
                "text": "Hello"
            }
        });

        let block: SectionBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_section_block_with_fields() {
        let block = SectionBlock::builder()
            .fields(vec!["field0", "field1", "field2", "field3", "field4"])
            .unwrap()
            .build()
            .unwrap();

        let json_val = serde_json::to_value(&block).unwrap();
        assert_eq!(json_val["fields"].as_array().unwrap().len(), 5);
    }

    #[test]
    fn test_section_block_fields_length_validation() {
        let result = SectionBlock::builder()
            .fields((0..11).map(|i| format!("field{}", i)))
            .unwrap()
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_section_block_max_fields_ok() {
        let result = SectionBlock::builder()
            .fields((0..10).map(|i| format!("field{}", i)))
            .unwrap()
            .build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_section_block_with_block_id() {
        let block = SectionBlock::builder()
            .text("text")
            .unwrap()
            .block_id("section_1")
            .build()
            .unwrap();

        assert_eq!(block.block_id, Some("section_1".to_string()));
    }

    #[test]
    fn test_section_block_validation_no_text_or_fields() {
        let block = SectionBlock {
            block_type: "section".to_string(),
            text: None,
            fields: None,
            accessory: None,
            block_id: None,
        };

        let result = block.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_section_block_validation_with_text_only() {
        let block = SectionBlock {
            block_type: "section".to_string(),
            text: Some(TextObject::plain("text").unwrap()),
            fields: None,
            accessory: None,
            block_id: None,
        };

        assert!(block.validate().is_ok());
    }

    #[test]
    fn test_section_block_validation_with_fields_only() {
        let block = SectionBlock {
            block_type: "section".to_string(),
            text: None,
            fields: Some(vec![TextObject::plain("field").unwrap()]),
            accessory: None,
            block_id: None,
        };

        assert!(block.validate().is_ok());
    }

    #[test]
    fn test_section_block_with_accessory() {
        let block = SectionBlock::builder()
            .text("Section with button")
            .unwrap()
            .accessory(json!({
                "type": "button",
                "text": {"type": "plain_text", "text": "Click"},
                "action_id": "btn"
            }))
            .build()
            .unwrap();

        assert!(block.accessory.is_some());
    }

    #[test]
    fn test_section_block_clone() {
        let block1 = SectionBlock::new("test").unwrap();
        let block2 = block1.clone();

        assert_eq!(block1, block2);
    }
}
