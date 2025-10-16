//! Phase 2a: Block Grouping
//!
//! This module implements the block grouping phase that creates hierarchical
//! structure from flat token streams using indentation analysis.
//!
//! The block grouper transforms a flat sequence of tokens (with Indent/Dedent markers)
//! into a hierarchical structure that represents the document's nesting. This is a
//! purely structural transformation - no parsing or element identification happens here.

use crate::ast::tokens::Token;

/// Phase 2a Block Grouper
///
/// Transforms flat token streams into hierarchical block structures
/// based on indentation patterns detected by the lexer.
pub struct BlockGrouper;

impl BlockGrouper {
    /// Create a new block grouper instance
    pub fn new() -> Self {
        Self
    }

    /// Group tokens into hierarchical blocks based on indentation
    ///
    /// Takes a flat stream of tokens with Indent/Dedent markers and
    /// produces a nested structure reflecting document hierarchy.
    pub fn group_blocks(&self, tokens: Vec<Token>) -> Result<BlockGroup, BlockGroupError> {
        let mut builder = BlockGroupBuilder::new();

        for token in tokens {
            match &token {
                Token::Indent { .. } => {
                    builder.push_indent();
                }
                Token::Dedent { .. } => {
                    builder.pop_dedent()?;
                }
                _ => {
                    builder.add_token(token);
                }
            }
        }

        builder.finalize()
    }
}

impl Default for BlockGrouper {
    fn default() -> Self {
        Self::new()
    }
}

/// Block group structure representing hierarchical token organization
///
/// This structure preserves the exact token sequences while organizing them
/// into a hierarchy based on indentation levels. Each BlockGroup represents
/// a single indentation level and can contain child groups for nested content.
#[derive(Debug, Clone)]
pub struct BlockGroup {
    /// Tokens at this indentation level (excluding Indent/Dedent tokens)
    pub tokens: Vec<Token>,
    /// Child block groups representing indented content
    /// Each child starts after an Indent token and ends at the corresponding Dedent
    pub children: Vec<BlockGroup>,
}

impl Default for BlockGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockGroup {
    /// Create a new empty block group
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create placeholder block group for compilation
    pub fn placeholder() -> Self {
        Self::new()
    }

    /// Check if this block group is empty (no tokens or children)
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.children.is_empty()
    }

    /// Add a token to this block group
    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Add a child block group
    pub fn add_child(&mut self, child: BlockGroup) {
        self.children.push(child);
    }
}

/// Internal builder for constructing block groups
struct BlockGroupBuilder {
    /// Stack of block groups at each indentation level
    /// The bottom of the stack (index 0) is always the root level
    stack: Vec<BlockGroup>,
    /// Current tokens being accumulated before the next indent
    current_tokens: Vec<Token>,
}

impl BlockGroupBuilder {
    fn new() -> Self {
        Self {
            stack: vec![BlockGroup::new()],
            current_tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token: Token) {
        self.current_tokens.push(token);
    }

    fn push_indent(&mut self) {
        // Flush current tokens to the current level
        self.flush_current_tokens();

        // Create a new block group for the indented content
        let new_group = BlockGroup::new();
        self.stack.push(new_group);
    }

    fn pop_dedent(&mut self) -> Result<(), BlockGroupError> {
        if self.stack.len() <= 1 {
            return Err(BlockGroupError::InvalidIndentation(
                "Dedent without matching indent".to_string(),
            ));
        }

        // Flush any remaining tokens to the current level
        self.flush_current_tokens();

        // Pop the completed block group and add it as a child to the parent
        let completed = self.stack.pop().unwrap();
        self.stack.last_mut().unwrap().add_child(completed);

        Ok(())
    }

    fn flush_current_tokens(&mut self) {
        if !self.current_tokens.is_empty() {
            let current_level = self.stack.last_mut().unwrap();
            current_level.tokens.append(&mut self.current_tokens);
        }
    }

    fn finalize(mut self) -> Result<BlockGroup, BlockGroupError> {
        // Flush any remaining tokens
        self.flush_current_tokens();

        // Should end with exactly one item in stack (the root)
        if self.stack.len() != 1 {
            return Err(BlockGroupError::InvalidIndentation(format!(
                "Unclosed indentation levels: expected 1, found {}",
                self.stack.len()
            )));
        }

        Ok(self.stack.pop().unwrap())
    }
}

/// Block grouping error types
#[derive(Debug, Clone)]
pub enum BlockGroupError {
    /// Invalid indentation structure
    InvalidIndentation(String),
    /// Malformed container boundaries
    MalformedContainer(String),
    /// Unexpected token in grouping context
    UnexpectedToken(String),
}

impl std::fmt::Display for BlockGroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockGroupError::InvalidIndentation(msg) => write!(f, "Invalid indentation: {}", msg),
            BlockGroupError::MalformedContainer(msg) => write!(f, "Malformed container: {}", msg),
            BlockGroupError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
        }
    }
}

impl std::error::Error for BlockGroupError {}
