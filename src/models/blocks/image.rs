//! Image block implementation.

use crate::error::{Result, SlackError};
use crate::models::objects::TextObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum length for image URLs (3000 characters).
pub const MAX_IMAGE_URL_LENGTH: usize = 3000;

/// Maximum length for image alt text (2000 characters).
pub const MAX_IMAGE_ALT_TEXT_LENGTH: usize = 2000;

/// Maximum length for image title (2000 characters).
pub const MAX_IMAGE_TITLE_LENGTH: usize = 2000;

/// An image block for displaying images.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageBlock {
    /// The type of block (always "image").
    #[serde(rename = "type")]
    pub block_type: String,

    /// The URL of the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    /// Alternative representation using Slack file object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slack_file: Option<Value>,

    /// Alt text for the image (required, max 2000 characters).
    pub alt_text: String,

    /// An optional title for the image (max 2000 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<TextObject>,

    /// An optional unique identifier for the block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
}

impl ImageBlock {
    /// Creates a new image block with a URL.
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
            block_type: "image".to_string(),
            image_url: Some(url),
            slack_file: None,
            alt_text: alt,
            title: None,
            block_id: None,
        })
    }

    /// Creates a new image block with a Slack file.
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
            block_type: "image".to_string(),
            image_url: None,
            slack_file: Some(slack_file),
            alt_text: alt,
            title: None,
            block_id: None,
        })
    }

    /// Sets the title for the image.
    pub fn with_title(mut self, title: impl Into<String>) -> Result<Self> {
        let title_str = title.into();
        if title_str.len() > MAX_IMAGE_TITLE_LENGTH {
            return Err(SlackError::Validation(format!(
                "Image title length {} exceeds maximum {}",
                title_str.len(),
                MAX_IMAGE_TITLE_LENGTH
            )));
        }
        self.title = Some(TextObject::plain(title_str)?);
        Ok(self)
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
    fn test_image_block_basic() {
        let block = ImageBlock::new("https://example.com/image.png", "An image").unwrap();

        assert_eq!(block.block_type, "image");
        assert_eq!(
            block.image_url,
            Some("https://example.com/image.png".to_string())
        );
        assert_eq!(block.alt_text, "An image");
    }

    #[test]
    fn test_image_block_serialization() {
        let input = json!({
            "type": "image",
            "image_url": "https://example.com/img.png",
            "alt_text": "Alt text"
        });

        let block: ImageBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_image_block_with_title() {
        let block = ImageBlock::new("https://example.com/img.png", "Alt")
            .unwrap()
            .with_title("Image Title")
            .unwrap();

        let json_val = serde_json::to_value(&block).unwrap();
        assert_eq!(json_val["title"]["text"], "Image Title");
    }

    #[test]
    fn test_image_block_with_block_id() {
        let block = ImageBlock::new("https://example.com/img.png", "Alt")
            .unwrap()
            .with_block_id("img_1");

        assert_eq!(block.block_id, Some("img_1".to_string()));
    }

    #[test]
    fn test_image_block_url_length_validation() {
        let long_url = format!("https://example.com/{}", "a".repeat(3000));
        let result = ImageBlock::new(&long_url, "alt");
        assert!(result.is_err());
    }

    #[test]
    fn test_image_block_alt_text_length_validation() {
        let long_alt = "a".repeat(2001);
        let result = ImageBlock::new("https://example.com/img.png", &long_alt);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_block_title_length_validation() {
        let long_title = "a".repeat(2001);
        let block = ImageBlock::new("https://example.com/img.png", "alt").unwrap();
        let result = block.with_title(&long_title);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_block_round_trip() {
        let input = json!({
            "type": "image",
            "image_url": "https://example.com/img.png",
            "alt_text": "Alt",
            "title": {"type": "plain_text", "text": "Title"},
            "block_id": "img_1"
        });

        let block: ImageBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_image_block_from_slack_file() {
        let slack_file = json!({"id": "F123"});
        let block = ImageBlock::from_slack_file(slack_file.clone(), "Alt").unwrap();

        assert_eq!(block.slack_file, Some(slack_file));
        assert_eq!(block.image_url, None);
    }
}
