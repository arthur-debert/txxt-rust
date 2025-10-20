//! List Block Element
//!
//! List blocks with sophisticated styling support.

use serde::{Deserialize, Serialize};

use crate::cst::ScannerTokenSequence;
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    ScannerTokenSequence,
use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
};

    containers::ContentContainer,
    core::{BlockElement, ElementType, TxxtElement},
    inlines::TextTransform,
use super::super::{
    containers::ContentContainer,
    core::{BlockElement, ElementType, TxxtElement},
    inlines::TextTransform,
};

/// List block with sophisticated styling support
///
/// TODO: Migrate existing list logic from src/ast/blocks.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListBlock {
    /// Decoration/styling for this list (from first item)
    pub decoration_type: ListDecorationType,

    /// List items with their original markers
    pub items: Vec<ListItem>,

    /// Annotations attached to this list
    pub annotations: Vec<Annotation>,

    /// Parameters for this list
    pub parameters: Parameters,

    /// Raw tokens for source reconstruction
    pub tokens: ScannerTokenSequence,
}

/// List decoration/styling information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListDecorationType {
    /// The numbering/marker style
    pub style: NumberingStyle,

    /// Short form vs full form
    pub form: NumberingForm,
}

/// Numbering styles supported for lists
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberingStyle {
    Plain,
    Numerical,
    Alphabetical,
    Roman,
}

/// Numbering form affects hierarchical display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberingForm {
    Short,
    Full,
}

/// Individual list item with preserved marker
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    /// The actual marker text as it appears in source
    pub marker: String,

    /// List item content (inline elements)
    pub content: Vec<TextTransform>,

    /// Nested content (if any) goes in a Container
    pub nested: Option<ContentContainer>,

    /// Annotations attached to this specific list item
    pub annotations: Vec<Annotation>,

    /// Parameters for this list item
    pub parameters: Parameters,

    /// Raw tokens for precise reconstruction
    pub tokens: ScannerTokenSequence,
}

impl TxxtElement for ListBlock {
    fn element_type(&self) -> ElementType {
        ElementType::Block
    }

    fn tokens(&self) -> &ScannerTokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl BlockElement for ListBlock {
    fn content_summary(&self) -> String {
        format!("List with {} items", self.items.len())
    }
}

impl TxxtElement for ListItem {
    fn element_type(&self) -> ElementType {
        ElementType::Block // List items are block-level within lists
    }

    fn tokens(&self) -> &ScannerTokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    fn parameters(&self) -> &Parameters {
        &self.parameters
    }
}

impl ListBlock {
    /// Create a new list block
    pub fn new(
        decoration_type: ListDecorationType,
        items: Vec<ListItem>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            decoration_type,
            items,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Get all items
    pub fn items(&self) -> &[ListItem] {
        &self.items
    }

    /// Add a new item to the list
    pub fn add_item(&mut self, item: ListItem) {
        self.items.push(item);
    }
}

impl ListItem {
    /// Create a new list item
    pub fn new(
        marker: String,
        content: Vec<TextTransform>,
        nested: Option<ContentContainer>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            marker,
            content,
            nested,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Get the text content of the item
    pub fn text_content(&self) -> String {
        self.content
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if this item has nested content
    pub fn has_nested_content(&self) -> bool {
        self.nested.is_some()
    }
}

impl Default for ListDecorationType {
    fn default() -> Self {
        Self {
            style: NumberingStyle::Plain,
            form: NumberingForm::Short,
        }
    }
}
