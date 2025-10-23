//! Block-level elements: verbatim, lists, definitions, and annotations
//!
//! This module defines the content container blocks that structure document
//! content but cannot host new document sessions (unlike sessions).
//!
//! src/parser/mod.rs has the full architecture overview.

use serde::{Deserialize, Serialize};

// All imports now handled by re-exports

// BREAKING CHANGE: Using new spec-aligned elements only
// All types now defined in elements/ module

// Re-export new types as the canonical implementations
pub use super::{
    definition::{DefinitionBlock as Definition, DefinitionTerm},
    list::{ListBlock as List, ListDecorationType, ListItem},
    verbatim::{VerbatimBlock, VerbatimType},
};

/// All block-level elements in TXXT documents
///
/// Blocks represent structural units that can contain other content.
/// Every block can have annotations attached based on proximity rules.
///
/// BREAKING CHANGE: Now uses new spec-aligned element types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Block {
    // Leaf blocks (cannot contain other blocks)
    Paragraph(super::paragraph::ParagraphBlock),
    VerbatimBlock(VerbatimBlock),
    BlankLine(super::core::BlankLine),

    // Content container blocks (cannot host sessions)
    List(List),
    Definition(Definition),
    Annotation(super::annotation::AnnotationBlock),

    // Session container blocks (can host new document sessions)
    Session(super::session::SessionBlock),

    // Container nodes for indented content
    Container(super::containers::ContentContainer),
}

// All block-level types now defined in elements/ - see re-exports above

impl From<super::containers::content::ContentContainerElement> for Block {
    fn from(element: super::containers::content::ContentContainerElement) -> Self {
        use super::containers::content::ContentContainerElement;
        match element {
            ContentContainerElement::Paragraph(p) => Block::Paragraph(p),
            ContentContainerElement::List(l) => Block::List(l),
            ContentContainerElement::Definition(d) => Block::Definition(d),
            ContentContainerElement::Verbatim(v) => Block::VerbatimBlock(v),
            ContentContainerElement::Annotation(a) => Block::Annotation(a),
            ContentContainerElement::Container(c) => Block::Container(c),
            ContentContainerElement::BlankLine(b) => Block::BlankLine(b),
        }
    }
}

impl From<super::containers::simple::SimpleBlockElement> for Block {
    fn from(element: super::containers::simple::SimpleBlockElement) -> Self {
        use super::containers::simple::SimpleBlockElement;
        match element {
            SimpleBlockElement::Paragraph(p) => Block::Paragraph(p),
            SimpleBlockElement::List(l) => Block::List(l),
            SimpleBlockElement::Verbatim(v) => Block::VerbatimBlock(v),
            SimpleBlockElement::BlankLine(b) => Block::BlankLine(b),
        }
    }
}
