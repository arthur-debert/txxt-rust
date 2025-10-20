//! Content Container
//!
//! Content containers hold any block elements except sessions. They are used for:
//! - List item content
//! - Definition content  
//! - Annotation content
//! - Any indented content that cannot host new document sessions

use serde::{Deserialize, Serialize};

use crate::cst::ScannerTokenSequence;
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    ScannerTokenSequence,
use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
};

use super::super::core::{ContainerElement, ContainerType, ElementType, TxxtElement};

/// Content container - holds any blocks except sessions
///
/// From `docs/specs/core/terminology.txxt`:
/// "Content Container: Can hold any block element except sessions.
/// Used for list item content, definition content, annotation content."
///
/// The key architectural insight: containers are what get indented,
/// not their parent elements.
///
/// Example:
/// ```txxt
/// - Item 1
/// - Item 2
///     - Nested item    // This creates a ContentContainer
/// ```
///
/// AST Structure:
/// ```text
/// List
/// ├── ListItem("Item 1")  
/// ├── ListItem("Item 2")
/// └── ContentContainer
///     └── List
///         └── ListItem("Nested item")
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentContainer {
    /// Child block elements (cannot include sessions)
    pub content: Vec<ContentContainerElement>,

    /// Annotations attached to this container
    pub annotations: Vec<Annotation>,

    /// Parameters for metadata (rare for containers)
    pub parameters: Parameters,

    /// Source position information
    pub tokens: ScannerTokenSequence,
}

/// Elements that can be contained in a content container
///
/// Enforces the restriction that content containers cannot contain sessions.
/// This type safety ensures spec compliance at compile time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentContainerElement {
    /// Paragraph blocks
    Paragraph(super::super::paragraph::ParagraphBlock),

    /// List blocks  
    List(super::super::list::ListBlock),

    /// Definition blocks
    Definition(super::super::definition::DefinitionBlock),

    /// Verbatim blocks
    Verbatim(super::super::verbatim::VerbatimBlock),

    /// Annotation blocks
    Annotation(super::super::annotation::AnnotationBlock),

    /// Nested content containers
    Container(ContentContainer),

    /// Blank lines (structural separators)
    BlankLine(super::super::core::BlankLine),
    // Note: SessionBlock intentionally NOT included - type safety!
}

impl TxxtElement for ContentContainer {
    fn element_type(&self) -> ElementType {
        ElementType::Container
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

impl ContainerElement for ContentContainer {
    fn container_type(&self) -> ContainerType {
        ContainerType::Content
    }

    fn can_contain_sessions(&self) -> bool {
        false // Content containers cannot contain sessions
    }

    fn child_elements(&self) -> Vec<&dyn TxxtElement> {
        self.content
            .iter()
            .map(|element| match element {
                ContentContainerElement::Paragraph(p) => p as &dyn TxxtElement,
                ContentContainerElement::List(l) => l as &dyn TxxtElement,
                ContentContainerElement::Definition(d) => d as &dyn TxxtElement,
                ContentContainerElement::Verbatim(v) => v as &dyn TxxtElement,
                ContentContainerElement::Annotation(a) => a as &dyn TxxtElement,
                ContentContainerElement::Container(c) => c as &dyn TxxtElement,
                ContentContainerElement::BlankLine(b) => b as &dyn TxxtElement,
            })
            .collect()
    }
}

impl ContentContainer {
    /// Create a new content container
    pub fn new(
        content: Vec<ContentContainerElement>,
        annotations: Vec<Annotation>,
        parameters: Parameters,
        tokens: ScannerTokenSequence,
    ) -> Self {
        Self {
            content,
            annotations,
            parameters,
            tokens,
        }
    }

    /// Check if the container is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Get the number of child elements
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Add a child element
    pub fn add_element(&mut self, element: ContentContainerElement) {
        self.content.push(element);
    }
}
