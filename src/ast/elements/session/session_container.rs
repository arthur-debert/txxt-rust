//! Session Container
//!
//! Session containers can hold any block elements including sessions.
//! They are used for:
//! - Document root content
//! - Session content (nested sections)

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    scanner_tokens::ScannerTokenSequence,
};

use super::super::core::{ContainerElement, ContainerType, ElementType, TxxtElement};

/// Session container - holds any blocks including sessions
///
/// From `docs/specs/core/terminology.txxt`:
/// "Session Container: Can hold any block element including sessions.
/// Must be surrounded by blank lines. Used for session content and document root."
///
/// Session containers are the only containers that can host new document
/// sessions, making them distinct from content containers.
///
/// Example:
/// ```txxt
/// 1. Introduction
///
///     This is a paragraph in a session container.
///     
///     2. Subsection
///     
///         More content here.
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionContainer {
    /// Child elements (can include sessions)
    pub content: Vec<SessionContainerElement>,

    /// Annotations attached to this container
    pub annotations: Vec<Annotation>,

    /// Parameters for metadata
    pub parameters: Parameters,

    /// Source position information
    pub tokens: ScannerTokenSequence,
}

/// Elements that can be contained in a session container
///
/// Session containers can hold all block types including sessions,
/// making them the most permissive container type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionContainerElement {
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

    /// Session blocks (only allowed in session containers!)
    Session(super::block::SessionBlock),

    /// Nested content containers
    ContentContainer(super::super::containers::content::ContentContainer),

    /// Nested session containers
    SessionContainer(SessionContainer),

    /// Blank lines (structural separators)
    BlankLine(super::super::core::BlankLine),
}

impl TxxtElement for SessionContainer {
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

impl ContainerElement for SessionContainer {
    fn container_type(&self) -> ContainerType {
        ContainerType::Session
    }

    fn can_contain_sessions(&self) -> bool {
        true // Session containers can contain sessions
    }

    fn child_elements(&self) -> Vec<&dyn TxxtElement> {
        self.content
            .iter()
            .map(|element| match element {
                SessionContainerElement::Paragraph(p) => p as &dyn TxxtElement,
                SessionContainerElement::List(l) => l as &dyn TxxtElement,
                SessionContainerElement::Definition(d) => d as &dyn TxxtElement,
                SessionContainerElement::Verbatim(v) => v as &dyn TxxtElement,
                SessionContainerElement::Annotation(a) => a as &dyn TxxtElement,
                SessionContainerElement::Session(s) => s as &dyn TxxtElement,
                SessionContainerElement::ContentContainer(c) => c as &dyn TxxtElement,
                SessionContainerElement::SessionContainer(s) => s as &dyn TxxtElement,
                SessionContainerElement::BlankLine(b) => b as &dyn TxxtElement,
            })
            .collect()
    }
}

impl SessionContainer {
    /// Create a new session container
    pub fn new(
        content: Vec<SessionContainerElement>,
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
    pub fn add_element(&mut self, element: SessionContainerElement) {
        self.content.push(element);
    }

    /// Find all sessions within this container
    pub fn sessions(&self) -> Vec<&super::block::SessionBlock> {
        self.content
            .iter()
            .filter_map(|element| match element {
                SessionContainerElement::Session(session) => Some(session),
                _ => None,
            })
            .collect()
    }
}
