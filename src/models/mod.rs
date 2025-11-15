//! Block Kit models for building Slack UI components.
//!
//! This module provides type-safe representations of Slack's Block Kit
//! components, including blocks, elements, composition objects, and views.

pub mod blocks;
pub mod elements;
pub mod objects;
pub mod views;

// Re-export blocks
pub use blocks::{
    ActionsBlock, ContextBlock, DividerBlock, HeaderBlock, ImageBlock, InputBlock, SectionBlock,
};

// Re-export elements
pub use elements::{
    ButtonElement, ButtonStyle, ChannelMultiSelectElement, ChannelSelectElement, CheckboxesElement,
    ConversationMultiSelectElement, ConversationSelectElement, DatePickerElement,
    DateTimePickerElement, DispatchActionConfig, ExternalDataMultiSelectElement,
    ExternalDataSelectElement, ImageElement, OverflowMenuElement, PlainTextInputElement,
    RadioButtonsElement, StaticMultiSelectElement, StaticSelectElement, TimePickerElement,
    UserMultiSelectElement, UserSelectElement,
};

// Re-export objects
pub use objects::{ConfirmObject, ConfirmStyle, OptionGroup, SlackOption, TextObject};

// Re-export views
pub use views::{View, ViewState, ViewStateValue};
