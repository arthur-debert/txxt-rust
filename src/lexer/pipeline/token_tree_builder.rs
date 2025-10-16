//! Phase 1c: Token Tree Building
//!
//! This module implements the token tree building phase that creates hierarchical
//! token structures from flat token streams using indentation analysis.
//!
//! The token tree builder transforms a flat sequence of tokens (with Indent/Dedent markers)
//! into a hierarchical token tree that represents the document's nesting. This is a
//! purely structural transformation - no parsing or element identification happens here.

use crate::ast::tokens::Token;

/// Phase 1c Token Tree Builder
///
/// Transforms flat token streams into hierarchical token tree structures
/// based on indentation patterns detected by the lexer.
pub struct TokenTreeBuilder;

impl TokenTreeBuilder {
    /// Create a new token tree builder instance
    pub fn new() -> Self {
        Self
    }

    /// Build hierarchical token tree from flat token stream
    ///
    /// Takes a flat stream of tokens with Indent/Dedent markers and
    /// produces a nested token tree reflecting document hierarchy.
    pub fn build_tree(&self, tokens: Vec<Token>) -> Result<TokenTree, TokenTreeError> {
        let mut builder = TokenTreeBuilderInternal::new();

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

impl Default for TokenTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Token tree structure representing hierarchical token organization
///
/// This structure preserves the exact token sequences while organizing them
/// into a hierarchy based on indentation levels. Each TokenTree represents
/// a single indentation level and can contain child trees for nested content.
#[derive(Debug, Clone)]
pub struct TokenTree {
    /// Tokens at this indentation level (excluding Indent/Dedent tokens)
    pub tokens: Vec<Token>,
    /// Child token trees representing indented content
    /// Each child starts after an Indent token and ends at the corresponding Dedent
    pub children: Vec<TokenTree>,
}

impl Default for TokenTree {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenTree {
    /// Create a new empty token tree
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create placeholder token tree for compilation
    pub fn placeholder() -> Self {
        Self::new()
    }

    /// Check if this token tree is empty (no tokens or children)
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.children.is_empty()
    }

    /// Add a token to this token tree
    pub fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    /// Add a child token tree
    pub fn add_child(&mut self, child: TokenTree) {
        self.children.push(child);
    }
}

/// Internal builder for constructing token trees
struct TokenTreeBuilderInternal {
    /// Stack of token trees at each indentation level
    /// The bottom of the stack (index 0) is always the root level
    stack: Vec<TokenTree>,
    /// Current tokens being accumulated before the next indent
    current_tokens: Vec<Token>,
}

impl TokenTreeBuilderInternal {
    fn new() -> Self {
        Self {
            stack: vec![TokenTree::new()],
            current_tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token: Token) {
        self.current_tokens.push(token);
    }

    fn push_indent(&mut self) {
        // Flush current tokens to the current level
        self.flush_current_tokens();

        // Create a new token tree for the indented content
        let new_tree = TokenTree::new();
        self.stack.push(new_tree);
    }

    fn pop_dedent(&mut self) -> Result<(), TokenTreeError> {
        if self.stack.len() <= 1 {
            return Err(TokenTreeError::InvalidIndentation(
                "Dedent without matching indent".to_string(),
            ));
        }

        // Flush any remaining tokens to the current level
        self.flush_current_tokens();

        // Pop the completed token tree and add it as a child to the parent
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

    fn finalize(mut self) -> Result<TokenTree, TokenTreeError> {
        // Flush any remaining tokens
        self.flush_current_tokens();

        // Should end with exactly one item in stack (the root)
        if self.stack.len() != 1 {
            return Err(TokenTreeError::InvalidIndentation(format!(
                "Unclosed indentation levels: expected 1, found {}",
                self.stack.len()
            )));
        }

        Ok(self.stack.pop().unwrap())
    }
}

/// Token tree building error types
#[derive(Debug, Clone)]
pub enum TokenTreeError {
    /// Invalid indentation structure
    InvalidIndentation(String),
    /// Malformed container boundaries
    MalformedContainer(String),
    /// Unexpected token in tree building context
    UnexpectedToken(String),
}

impl std::fmt::Display for TokenTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenTreeError::InvalidIndentation(msg) => write!(f, "Invalid indentation: {}", msg),
            TokenTreeError::MalformedContainer(msg) => write!(f, "Malformed container: {}", msg),
            TokenTreeError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
        }
    }
}

impl std::error::Error for TokenTreeError {}
