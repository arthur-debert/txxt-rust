//! Phase 1c: Scanner Token Tree Building
//!
//! This module implements the scanner token tree building phase that creates hierarchical
//! token structures from flat scanner token streams using indentation analysis.
//!
//! The scanner token tree builder transforms a flat sequence of scanner tokens (with Indent/Dedent markers)
//! into a hierarchical scanner token tree that represents the document's nesting. This is a
//! purely structural transformation - no parsing or element identification happens here.

use crate::ast::scanner_tokens::ScannerToken;

/// Phase 1c Scanner Token Tree Builder
///
/// Transforms flat scanner token streams into hierarchical scanner token tree structures
/// based on indentation patterns detected by the lexer.
pub struct ScannerTokenTreeBuilder;

impl ScannerTokenTreeBuilder {
    /// Create a new scanner token tree builder instance
    pub fn new() -> Self {
        Self
    }

    /// Build hierarchical scanner token tree from flat scanner token stream
    ///
    /// Takes a flat stream of scanner tokens with Indent/Dedent markers and
    /// produces a nested scanner token tree reflecting document hierarchy.
    pub fn build_tree(&self, tokens: Vec<ScannerToken>) -> Result<ScannerTokenTree, ScannerTokenTreeError> {
        let mut builder = ScannerTokenTreeBuilderInternal::new();

        for token in tokens {
            match &token {
                ScannerToken::Indent { .. } => {
                    builder.push_indent();
                }
                ScannerToken::Dedent { .. } => {
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

impl Default for ScannerTokenTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Scanner token tree structure representing hierarchical scanner token organization
///
/// This structure preserves the exact scanner token sequences while organizing them
/// into a hierarchy based on indentation levels. Each ScannerTokenTree represents
/// a single indentation level and can contain child trees for nested content.
#[derive(Debug, Clone)]
pub struct ScannerTokenTree {
    /// Scanner tokens at this indentation level (excluding Indent/Dedent tokens)
    pub tokens: Vec<ScannerToken>,
    /// Child scanner token trees representing indented content
    /// Each child starts after an Indent token and ends at the corresponding Dedent
    pub children: Vec<ScannerTokenTree>,
}

impl Default for ScannerTokenTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ScannerTokenTree {
    /// Create a new empty scanner token tree
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create placeholder scanner token tree for compilation
    pub fn placeholder() -> Self {
        Self::new()
    }

    /// Check if this scanner token tree is empty (no tokens or children)
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty() && self.children.is_empty()
    }

    /// Add a scanner token to this scanner token tree
    pub fn add_token(&mut self, token: ScannerToken) {
        self.tokens.push(token);
    }

    /// Add a child scanner token tree
    pub fn add_child(&mut self, child: ScannerTokenTree) {
        self.children.push(child);
    }
}

/// Internal builder for constructing scanner token trees
struct ScannerTokenTreeBuilderInternal {
    /// Stack of scanner token trees at each indentation level
    /// The bottom of the stack (index 0) is always the root level
    stack: Vec<ScannerTokenTree>,
    /// Current scanner tokens being accumulated before the next indent
    current_tokens: Vec<ScannerToken>,
}

impl ScannerTokenTreeBuilderInternal {
    fn new() -> Self {
        Self {
            stack: vec![ScannerTokenTree::new()],
            current_tokens: Vec::new(),
        }
    }

    fn add_token(&mut self, token: ScannerToken) {
        self.current_tokens.push(token);
    }

    fn push_indent(&mut self) {
        // Flush current tokens to the current level
        self.flush_current_tokens();

        // Create a new scanner token tree for the indented content
        let new_tree = ScannerTokenTree::new();
        self.stack.push(new_tree);
    }

    fn pop_dedent(&mut self) -> Result<(), ScannerTokenTreeError> {
        if self.stack.len() <= 1 {
            return Err(ScannerTokenTreeError::InvalidIndentation(
                "Dedent without matching indent".to_string(),
            ));
        }

        // Flush any remaining tokens to the current level
        self.flush_current_tokens();

        // Pop the completed scanner token tree and add it as a child to the parent
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

    fn finalize(mut self) -> Result<ScannerTokenTree, ScannerTokenTreeError> {
        // Flush any remaining tokens
        self.flush_current_tokens();

        // Should end with exactly one item in stack (the root)
        if self.stack.len() != 1 {
            return Err(ScannerTokenTreeError::InvalidIndentation(format!(
                "Unclosed indentation levels: expected 1, found {}",
                self.stack.len()
            )));
        }

        Ok(self.stack.pop().unwrap())
    }
}

/// Scanner token tree building error types
#[derive(Debug, Clone)]
pub enum ScannerTokenTreeError {
    /// Invalid indentation structure
    InvalidIndentation(String),
    /// Malformed container boundaries
    MalformedContainer(String),
    /// Unexpected scanner token in tree building context
    UnexpectedToken(String),
}

impl std::fmt::Display for ScannerTokenTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScannerTokenTreeError::InvalidIndentation(msg) => write!(f, "Invalid indentation: {}", msg),
            ScannerTokenTreeError::MalformedContainer(msg) => write!(f, "Malformed container: {}", msg),
            ScannerTokenTreeError::UnexpectedToken(msg) => write!(f, "Unexpected token: {}", msg),
        }
    }
}

impl std::error::Error for ScannerTokenTreeError {}
