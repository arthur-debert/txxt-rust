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
use crate::parser::elements::{
    paragraph::paragraph::parse_paragraph, session::session::parse_session,
};

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
    pub fn parse_blocks(&self, token_tree: TokenTree) -> Result<Vec<ElementNode>, BlockParseError> {
        let mut elements = Vec::new();
        let mut root_tokens = token_tree.tokens.as_slice();
        let mut children = token_tree.children.as_slice();

        while !root_tokens.is_empty() {
            // Try to parse a session
            if !children.is_empty() {
                // Find the end of the first line of tokens
                let line_end = root_tokens
                    .iter()
                    .position(|t| matches!(t, crate::ast::tokens::Token::Newline { .. }))
                    .map(|p| p + 1)
                    .unwrap_or(root_tokens.len());

                let (line_tokens, rest_tokens) = root_tokens.split_at(line_end);
                root_tokens = rest_tokens;

                // Combine line tokens with the child's tokens for the session parser
                let mut session_tokens = line_tokens.to_vec();
                session_tokens.push(crate::ast::tokens::Token::BlankLine {
                    whitespace: String::new(),
                    span: children[0].tokens[0].span().clone(), // Placeholder span
                });
                session_tokens.extend(children[0].tokens.clone());

                match parse_session(&session_tokens) {
                    Ok(session) => {
                        elements.push(ElementNode::SessionBlock(session));
                        children = &children[1..];
                        continue;
                    }
                    Err(_) => {
                        // Not a session, fall back to paragraph
                        root_tokens = line_tokens; // backtrack
                    }
                }
            }

            // If not a session, parse as paragraphs
            let paragraph_groups = self.split_into_paragraphs(root_tokens);
            for paragraph_tokens in paragraph_groups {
                if !paragraph_tokens.is_empty() {
                    match parse_paragraph(&paragraph_tokens) {
                        Ok(paragraph) => {
                            elements.push(ElementNode::ParagraphBlock(paragraph));
                        }
                        Err(_) => {
                            // Skip unrecognized tokens for now
                        }
                    }
                }
            }
            break; // All root tokens consumed by paragraph parser
        }

        // Recursively parse any remaining children
        for child_tree in children {
            elements.extend(self.parse_blocks(child_tree.clone())?);
        }

        Ok(elements)
    }

    /// Split tokens into paragraph groups based on BlankLine boundaries
    fn split_into_paragraphs(
        &self,
        tokens: &[crate::ast::tokens::Token],
    ) -> Vec<Vec<crate::ast::tokens::Token>> {
        tokens
            .split(|token| matches!(token, crate::ast::tokens::Token::BlankLine { .. }))
            .map(|s| s.to_vec())
            .filter(|s| !s.is_empty())
            .collect()
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
