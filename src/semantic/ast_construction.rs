//! Phase 2b: AST Construction
//!
//! Converts semantic tokens into AST tree nodes using a regex-based grammar engine
//! with carefully planned precedence rules.
//!
//! ## Implementation Focus
//!
//! This phase focuses ONLY on the core three elements without explicit syntax markers:
//! - **Paragraph**: Catch-all for text lines
//! - **Session**: Blank-line enclosed titles with indented content
//! - **List**: Consecutive sequence markers (at least 2 items)
//!
//! These three elements are the source of all parsing difficulty in txxt due to their
//! lack of explicit syntax markers. We must master their interaction before adding
//! other elements (Definition, Annotation, Verbatim).
//!
//! ## Architecture
//!
//! The parser uses a regex-based pattern matching approach:
//! 1. Serialize token stream to string representation (e.g., "<BlankLine> <TextSpan>")
//! 2. Try grammar rules in precedence order
//! 3. On match, count capture groups to determine tokens consumed
//! 4. Delegate to element constructors in src/semantic/elements/
//! 5. Continue with remaining tokens
//!
//! See docs/proposals/regex-grammar-engine.txxt for complete design.

use crate::cst::{HighLevelToken, HighLevelTokenList};
use crate::semantic::BlockParseError;

/// AST Construction parser for converting semantic tokens to AST nodes
///
/// This parser implements a regex-based grammar engine that matches token patterns
/// and delegates to element constructors for AST node creation.
pub struct AstConstructor<'a> {
    /// The semantic token stream being parsed
    #[allow(dead_code)] // Will be used in Phase 1 implementation
    tokens: &'a [HighLevelToken],
    /// Current parsing position in the token stream
    #[allow(dead_code)] // Will be used in Phase 1 implementation
    position: usize,
}

impl<'a> AstConstructor<'a> {
    /// Create a new AST constructor instance
    pub fn new() -> Self {
        Self {
            tokens: &[],
            position: 0,
        }
    }

    /// Create a new AST constructor instance with token stream
    pub fn with_tokens(tokens: &'a [HighLevelToken]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Parse semantic tokens into AST nodes
    ///
    /// This is the main entry point for AST construction.
    ///
    /// # Arguments
    /// * `semantic_tokens` - The semantic token list to parse
    ///
    /// # Returns
    /// * `Result<Vec<AstNode>, BlockParseError>` - Parsed AST nodes
    pub fn parse(
        &mut self,
        _semantic_tokens: &'a HighLevelTokenList,
    ) -> Result<Vec<AstNode>, BlockParseError> {
        // TODO: Implement regex-based grammar engine
        // Phase 1: Start with paragraph-only parsing
        todo!("Implement AST construction parser")
    }
}

impl<'a> Default for AstConstructor<'a> {
    fn default() -> Self {
        Self::new()
    }
}

/// AST node types that can be constructed from semantic tokens
///
/// Currently supports only the core three elements: Paragraph, Session, List.
/// Other elements (Definition, Annotation, Verbatim) will be added after mastering
/// the core three.
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Paragraph block node
    Paragraph(crate::ast::elements::paragraph::ParagraphBlock),
    /// Session block node
    Session(crate::ast::elements::session::SessionBlock),
    /// List block node
    List(crate::ast::elements::list::ListBlock),
}

impl AstNode {
    /// Convert an AstNode to an ElementNode for integration with the parsing pipeline
    pub fn to_element_node(&self) -> crate::ast::elements::core::ElementNode {
        match self {
            AstNode::Paragraph(block) => {
                crate::ast::elements::core::ElementNode::ParagraphBlock(block.clone())
            }
            AstNode::Session(block) => {
                crate::ast::elements::core::ElementNode::SessionBlock(block.clone())
            }
            AstNode::List(block) => {
                crate::ast::elements::core::ElementNode::ListBlock(block.clone())
            }
        }
    }
}

impl AstConstructor<'_> {
    /// Parse semantic tokens and return ElementNodes for pipeline integration
    pub fn parse_to_element_nodes(
        semantic_tokens: &HighLevelTokenList,
    ) -> Result<Vec<crate::ast::elements::core::ElementNode>, BlockParseError> {
        let mut constructor = AstConstructor::new();
        let ast_nodes = constructor.parse(semantic_tokens)?;
        Ok(ast_nodes
            .into_iter()
            .map(|node| node.to_element_node())
            .collect())
    }
}
