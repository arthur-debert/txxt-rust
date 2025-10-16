//! Phase 2a: Block Parsing
//!
//! Converts token trees into typed AST nodes for block elements.
//! This is the first step of Phase 2 parsing, where we take the hierarchical
//! token structure from the lexer and create proper AST element nodes.
//!
//! # Block Types
//!
//! - Paragraphs
//! - Lists (numbered, bulleted, alphabetical)
//! - Definitions
//! - Annotations
//! - Verbatim blocks
//! - Sessions
//! - Containers
//!
//! # Input/Output
//!
//! - **Input**: `TokenTree` from lexer (Phase 1c)
//! - **Output**: AST tree of `ElementNode` variants

use crate::ast::ElementNode;
use crate::lexer::pipeline::TokenTree;
use crate::parser::elements::paragraph::paragraph::parse_paragraph;

/// Block parser for converting token trees to AST nodes
///
/// This parser takes the hierarchical token structure and creates
/// typed AST nodes for each block element type.
pub struct BlockParser;

impl Default for BlockParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockParser {
    /// Create a new block parser instance
    pub fn new() -> Self {
        Self
    }

    /// Parse token tree into AST block elements
    ///
    /// Takes a hierarchical token tree and converts it into a tree
    /// of typed AST element nodes. Each block type is handled by
    /// its specific parsing logic.
    #[allow(clippy::only_used_in_recursion)]
    pub fn parse_blocks(&self, token_tree: TokenTree) -> Result<Vec<ElementNode>, BlockParseError> {
        let mut elements = Vec::new();

        // For Phase 1 Simple Elements, we focus on parsing paragraphs
        // Other elements will be added in subsequent phases

        // Process tokens at the root level as a group
        if !token_tree.tokens.is_empty() {
            // Try to parse all tokens as a paragraph (default element type)
            match parse_paragraph(&token_tree.tokens) {
                Ok(paragraph) => {
                    elements.push(ElementNode::ParagraphBlock(paragraph));
                }
                Err(_) => {
                    // If it's not a paragraph, we'll handle other element types later
                    // For now, skip unrecognized tokens
                }
            }
        }

        // Process child token trees recursively
        for child_tree in &token_tree.children {
            let child_elements = self.parse_blocks(child_tree.clone())?;
            elements.extend(child_elements);
        }

        Ok(elements)
    }
}

/// Errors that can occur during block parsing
#[derive(Debug)]
pub enum BlockParseError {
    /// Invalid block structure detected
    InvalidStructure(String),
    /// Unsupported block type encountered
    UnsupportedBlockType(String),
    /// Parse error at specific position
    ParseError {
        position: crate::ast::tokens::Position,
        message: String,
    },
}

impl std::fmt::Display for BlockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockParseError::InvalidStructure(msg) => write!(f, "Invalid block structure: {}", msg),
            BlockParseError::UnsupportedBlockType(block_type) => {
                write!(f, "Unsupported block type: {}", block_type)
            }
            BlockParseError::ParseError { position, message } => {
                write!(f, "Parse error at position {:?}: {}", position, message)
            }
        }
    }
}

impl std::error::Error for BlockParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_parser_creation() {
        let _parser = BlockParser::new();
        // Basic test to ensure parser can be created
        // The test passes if we reach this point without panicking
    }

    #[test]
    fn test_parse_blocks_placeholder() {
        let parser = BlockParser::new();
        let token_tree = TokenTree {
            tokens: vec![],
            children: vec![],
        };

        // This should return empty result until Phase 2 is implemented
        let result = parser.parse_blocks(token_tree);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
