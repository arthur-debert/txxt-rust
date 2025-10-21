//! Phase 2c: Inline Parsing
//!
//! Handles inline elements within block content. This is the second step
//! of Phase 2 parsing, where we take the AST block elements and process
//! any inline formatting, references, and other inline elements.
//!
//! src/parser/mod.rs has the full architecture overview.

use crate::ast::ElementNode;
use crate::semantic::elements::formatting::parse_formatting_elements;

/// Inline parser for processing inline elements within blocks
///
/// This parser takes AST block elements and processes any inline
/// formatting, references, and other inline elements within them.
pub struct InlineParser;

impl Default for InlineParser {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineParser {
    /// Create a new inline parser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse inline elements within block AST nodes
    ///
    /// Takes AST block elements and processes any inline formatting,
    /// references, and other inline elements within their content.
    /// Returns the same AST structure but with inlines processed.
    pub fn parse_inlines(
        &self,
        blocks: Vec<ElementNode>,
    ) -> Result<Vec<ElementNode>, InlineParseError> {
        blocks
            .into_iter()
            .map(|node| self.parse_inlines_in_node(node))
            .collect()
    }

    fn parse_inlines_in_node(&self, node: ElementNode) -> Result<ElementNode, InlineParseError> {
        match node {
            ElementNode::ParagraphBlock(mut block) => {
                // Use existing scanner tokens from the block instead of re-tokenizing
                block.content = parse_formatting_elements(&block.tokens.tokens)?;
                Ok(ElementNode::ParagraphBlock(block))
            }
            _ => Ok(node),
        }
    }
}

/// Errors that can occur during inline parsing
#[derive(Debug)]
pub enum InlineParseError {
    /// Invalid inline structure detected
    InvalidStructure(String),
    /// Unsupported inline type encountered
    UnsupportedInlineType(String),
    /// Parse error at specific position
    ParseError {
        position: crate::cst::Position,
        message: String,
    },
    /// Reference resolution error
    ReferenceError(String),
    /// Generic parse error
    GenericParseError(String),
}

impl From<crate::semantic::elements::inlines::InlineParseError> for InlineParseError {
    fn from(err: crate::semantic::elements::inlines::InlineParseError) -> Self {
        InlineParseError::GenericParseError(err.to_string())
    }
}

impl std::fmt::Display for InlineParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineParseError::InvalidStructure(msg) => {
                write!(f, "Invalid inline structure: {}", msg)
            }
            InlineParseError::UnsupportedInlineType(inline_type) => {
                write!(f, "Unsupported inline type: {}", inline_type)
            }
            InlineParseError::ParseError { position, message } => {
                write!(f, "Parse error at position {:?}: {}", position, message)
            }
            InlineParseError::ReferenceError(reference) => {
                write!(f, "Reference resolution error: {}", reference)
            }
            InlineParseError::GenericParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
        }
    }
}

impl std::error::Error for InlineParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_parser_creation() {
        let _parser = InlineParser::new();
        // Basic test to ensure parser can be created
        // The test passes if we reach this point without panicking
    }

    #[test]
    fn test_parse_inlines_placeholder() {
        let parser = InlineParser::new();
        let blocks = vec![];

        // This should return the blocks unchanged until Phase 2 is implemented
        let result = parser.parse_inlines(blocks.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), blocks);
    }
}
