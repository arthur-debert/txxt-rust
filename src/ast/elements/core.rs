//! Core Element Types and Traits
//!
//! This module defines the foundational type system for TXXT AST elements,
//! implementing the element hierarchy from `docs/specs/core/terminology.txxt`.

use serde::{Deserialize, Serialize};

use crate::ast::elements::{
    annotation::annotation_content::Annotation, components::parameters::Parameters,
    tokens::TokenSequence,
};

/// Element type classification according to specification terminology
///
/// From `docs/specs/core/terminology.txxt`:
/// - Span: Smallest unit, no line breaks (words, phrases, inline formatting)
/// - Line: Full line of text, can host multiple spans  
/// - Block: Contains one or more lines (paragraphs, lists, sessions)
/// - Container: Holds child elements of different types (what gets indented)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementType {
    /// Span elements - inline content within lines
    Span,

    /// Line elements - complete lines of text
    Line,

    /// Block elements - structural document units
    Block,

    /// Container elements - hierarchical content holders
    Container,
}

/// Base trait for all TXXT AST elements
///
/// Provides common functionality required across all element types:
/// - Element type classification for traversal and validation
/// - Token sequence access for language server features  
/// - Annotation access for metadata handling
/// - Parameter access for element configuration
pub trait TxxtElement {
    /// Get the element type for this node
    fn element_type(&self) -> ElementType;

    /// Access the token sequence for source reconstruction and language server features
    fn tokens(&self) -> &TokenSequence;

    /// Access annotations attached to this element
    fn annotations(&self) -> &[Annotation];

    /// Access parameters attached to this element
    fn parameters(&self) -> &Parameters;
}

/// Trait for span elements (inline content)
///
/// Span elements cannot contain line breaks and represent the smallest
/// semantic units within lines.
pub trait SpanElement: TxxtElement {
    /// Get the text content of this span
    fn text_content(&self) -> String;

    /// Check if this span contains formatting
    fn is_formatted(&self) -> bool {
        false // Default: plain text
    }
}

/// Trait for line elements  
///
/// Line elements encompass complete lines and may contain multiple spans.
pub trait LineElement: TxxtElement {
    /// Get all spans within this line
    fn spans(&self) -> Vec<&dyn SpanElement>;

    /// Get the complete text content of the line
    fn line_content(&self) -> String;
}

/// Trait for block elements
///
/// Block elements are the primary structural units containing one or more lines.
pub trait BlockElement: TxxtElement {
    /// Check if this block can contain other blocks (i.e., is it a container)
    fn can_contain_blocks(&self) -> bool {
        false // Default: leaf block
    }

    /// Get the content summary for this block
    fn content_summary(&self) -> String;
}

/// Trait for container elements
///
/// Container elements hold child elements and implement the indentation architecture.
pub trait ContainerElement: TxxtElement {
    /// Get the container type for content validation
    fn container_type(&self) -> ContainerType;

    /// Check if this container can hold sessions
    fn can_contain_sessions(&self) -> bool;

    /// Get child elements (type-erased for traversal)
    fn child_elements(&self) -> Vec<&dyn TxxtElement>;
}

/// Container type for type-safe content restrictions
///
/// From `docs/specs/core/terminology.txxt`:
/// - Content Container: Cannot contain sessions (list items, definitions, etc.)
/// - Session Container: Can contain sessions (document root, session content)  
/// - Ignore Container: Verbatim content only
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerType {
    /// Content container: holds any blocks except sessions
    Content,

    /// Session container: holds any blocks including sessions
    Session,

    /// Ignore container: holds verbatim content only
    Ignore,
}

/// Unified element node type for the complete AST
///
/// This enum provides a type-safe way to handle all element types while
/// maintaining the spec-aligned terminology and hierarchy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementNode {
    // Span elements (inline)
    TextSpan(super::inlines::TextSpan),
    BoldSpan(super::formatting::BoldSpan),
    ItalicSpan(super::formatting::ItalicSpan),
    CodeSpan(super::formatting::CodeSpan),
    MathSpan(super::formatting::MathSpan),
    ReferenceSpan(super::inlines::Reference),
    CitationSpan(super::references::CitationSpan),
    PageReferenceSpan(super::references::PageReferenceSpan),
    SessionReferenceSpan(super::references::SessionReferenceSpan),
    FootnoteReferenceSpan(super::references::FootnoteReferenceSpan),

    // Line elements
    TextLine(super::inlines::TextLine),
    BlankLine(BlankLine),

    // Block elements
    ParagraphBlock(super::paragraph::ParagraphBlock),
    ListBlock(super::list::ListBlock),
    DefinitionBlock(super::definition::DefinitionBlock),
    VerbatimBlock(super::verbatim::VerbatimBlock),
    SessionBlock(super::session::SessionBlock),
    AnnotationBlock(super::annotation::AnnotationBlock),

    // Container elements
    ContentContainer(super::containers::ContentContainer),
    SessionContainer(super::session::SessionContainer),
    IgnoreContainer(super::verbatim::IgnoreContainer),
}

impl ElementNode {
    /// Get the element type for this node
    pub fn element_type(&self) -> ElementType {
        match self {
            // Span elements
            ElementNode::TextSpan(_)
            | ElementNode::BoldSpan(_)
            | ElementNode::ItalicSpan(_)
            | ElementNode::CodeSpan(_)
            | ElementNode::MathSpan(_)
            | ElementNode::ReferenceSpan(_)
            | ElementNode::CitationSpan(_)
            | ElementNode::PageReferenceSpan(_)
            | ElementNode::SessionReferenceSpan(_)
            | ElementNode::FootnoteReferenceSpan(_) => ElementType::Span,

            // Line elements
            ElementNode::TextLine(_) | ElementNode::BlankLine(_) => ElementType::Line,

            // Block elements
            ElementNode::ParagraphBlock(_)
            | ElementNode::ListBlock(_)
            | ElementNode::DefinitionBlock(_)
            | ElementNode::VerbatimBlock(_)
            | ElementNode::SessionBlock(_)
            | ElementNode::AnnotationBlock(_) => ElementType::Block,

            // Container elements
            ElementNode::ContentContainer(_)
            | ElementNode::SessionContainer(_)
            | ElementNode::IgnoreContainer(_) => ElementType::Container,
        }
    }
}

/// Helper type for blank lines (structural separators)
///
/// Blank lines are significant in TXXT as they separate paragraphs, end blocks,
/// and affect annotation attachment rules.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlankLine {
    /// Source position information
    pub tokens: TokenSequence,
}

impl TxxtElement for BlankLine {
    fn element_type(&self) -> ElementType {
        ElementType::Line
    }

    fn tokens(&self) -> &TokenSequence {
        &self.tokens
    }

    fn annotations(&self) -> &[Annotation] {
        &[] // Blank lines don't have annotations
    }

    fn parameters(&self) -> &Parameters {
        // Blank lines don't have parameters - use a static empty instance
        use std::sync::OnceLock;
        static EMPTY_PARAMS: OnceLock<Parameters> = OnceLock::new();
        EMPTY_PARAMS.get_or_init(Parameters::default)
    }
}
