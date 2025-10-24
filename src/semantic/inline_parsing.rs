//! Phase 2b: Inline Parsing
//!
//! ============================================================================
//! OVERVIEW
//! ============================================================================
//!
//! Inline parsing is the second step of Phase 2 (Semantic Analysis). It takes
//! AST block elements with raw scanner tokens and processes inline elements
//! within their content, producing complete AST with both block and inline
//! structure.
//!
//! Inline elements represent formatting and references within text spans:
//! - Formatting: bold, italic, code, math
//! - References: citations, footnotes, sections, URLs, files
//!
//! Inlines operate on text spans within blocks and produce specialized text
//! spans. They require no document context, enabling independent processing
//! and testing.
//!
//! Inlines operate on a registry, and can leverage the infrastructure for most things
//! ============================================================================
//! ARCHITECTURE: REGISTRATION-BASED ENGINE
//! ============================================================================
//!
//! The inline parsing system uses a registration-based engine where inline
//! types register themselves with delimiter specifications and processing
//! pipelines. The engine orchestrates delimiter matching and pipeline execution.
//!
//! The first step is always to match the inline input, which is then passed to the first

//! ============================================================================
//! RELATED SPECIFICATIONS
//! ============================================================================
//!
//! - docs/specs/elements/formatting/inlines-general.txxt
//! - docs/specs/elements/formatting/formatting.txxt
//! - docs/specs/elements/references/references-general.txxt
//! - docs/specs/elements/references/citations.txxt
//!
//!
//! ============================================================================
//! SEE ALSO
//! ============================================================================
//!
//! Complete architecture: src/lib.rs
//! Engine implementation: src/semantic/elements/inlines/engine/mod.rs
//! Reference example: src/semantic/elements/inlines/engine/reference_example.rs
//! Registration: src/semantic/elements/inlines/engine/registration.rs
//! Block parsing: src/semantic/mod.rs
//! Tokenization: src/syntax/mod.rs

use crate::ast::elements::formatting::inlines::{Inline, Text, TextTransform};
use crate::ast::ElementNode;
use crate::semantic::elements::inlines::engine::{create_standard_engine, InlineEngine};

/// Inline parser for processing inline elements within blocks
///
/// This parser takes AST block elements and processes any inline
/// formatting, references, and other inline elements within them.
pub struct InlineParser {
    engine: InlineEngine,
}

impl Default for InlineParser {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineParser {
    /// Create a new inline parser instance
    pub fn new() -> Self {
        // Create engine with all standard inline types registered
        let engine = create_standard_engine()
            .expect("Failed to create standard inline engine - this is a bug");

        Self { engine }
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
                // Use the generic inline engine to parse all inline elements
                let inlines = self.engine.parse(&block.tokens.tokens);

                // Convert to TextTransform for backward compatibility
                // TODO: Update ParagraphBlock to support Vec<Inline> directly
                block.content = inlines_to_text_transforms(inlines);
                Ok(ElementNode::ParagraphBlock(block))
            }
            _ => Ok(node),
        }
    }
}

/// Convert Vec<Inline> to Vec<TextTransform> for backward compatibility
///
/// This helper function extracts TextTransform elements from Inline::TextLine variants.
/// Reference elements are currently converted to plain text since the ParagraphBlock
/// structure doesn't yet support mixed Inline content.
///
/// TODO: Update ParagraphBlock.content to Vec<Inline> to properly support references
fn inlines_to_text_transforms(inlines: Vec<Inline>) -> Vec<TextTransform> {
    inlines
        .into_iter()
        .map(|inline| match inline {
            Inline::TextLine(transform) => transform,
            Inline::Reference(reference) => {
                // Convert reference to plain text for now
                // Eventually ParagraphBlock should support Vec<Inline>
                let text = reference.target.display_text();
                TextTransform::Identity(Text::simple_with_tokens(&text, reference.tokens))
            }
            Inline::Link { target, tokens, .. } => {
                // Convert link to plain text for now
                TextTransform::Identity(Text::simple_with_tokens(&target, tokens))
            }
            Inline::Custom { name, tokens, .. } => {
                // Convert custom inline to plain text
                TextTransform::Identity(Text::simple_with_tokens(&name, tokens))
            }
        })
        .collect()
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
