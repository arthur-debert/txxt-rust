//! Structural elements: containers, sessions, and paragraphs
//!
//! BREAKING CHANGE: This module now re-exports the new spec-aligned element types
//! from the elements/ module. All structural elements have been migrated to follow
//! the specification taxonomy exactly.
//!
//! See `src/ast/elements/` for the actual implementations.

use serde::{Deserialize, Serialize};

// Re-export new spec-aligned types as canonical implementations
pub use super::elements::{
    containers::ContentContainer as Container,
    core::{BlankLine, ContainerType},
    list::{NumberingForm, NumberingStyle},
    paragraph::ParagraphBlock as Paragraph,
    session::SessionContainer,
    session::{SessionBlock as Session, SessionNumbering, SessionTitle},
    verbatim::IgnoreContainer,
};

// Import TokenSequence for IgnoreLine
use super::tokens::TokenSequence;

/// Ignore line - raw verbatim content preserved exactly
///
/// NOTE: This might be moved to elements/containers/ignore.rs in the future
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IgnoreLine {
    /// Raw content preserved byte-for-byte
    pub content: String,

    /// Source position information
    pub tokens: TokenSequence,
}
