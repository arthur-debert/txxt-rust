//! Container element tokenization
//!
//! Implements tokenization for container elements as defined in
//! docs/specs/elements/container.txxt
//!
//! Containers provide structural organization through indentation

use crate::ast::tokens::{Position, Token};
use crate::tokenizer::indentation::IndentationTracker;

/// Container tokenizer that works with indentation tracking
pub struct ContainerTokenizer {
    indent_tracker: IndentationTracker,
}

impl ContainerTokenizer {
    /// Create a new container tokenizer
    pub fn new() -> Self {
        Self {
            indent_tracker: IndentationTracker::new(),
        }
    }

    /// Process a line and generate container-related tokens
    pub fn process_line(&mut self, line: &str, position: Position) -> Vec<Token> {
        self.indent_tracker.set_position(position);
        self.indent_tracker.process_line_indentation(line)
    }

    /// Finalize container processing
    pub fn finalize(&mut self) -> Vec<Token> {
        self.indent_tracker.finalize()
    }

    /// Get current indentation level
    pub fn current_level(&self) -> usize {
        self.indent_tracker.current_level()
    }
}

impl Default for ContainerTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_basic() {
        let mut tokenizer = ContainerTokenizer::new();

        let tokens = tokenizer.process_line("base level", Position { row: 0, column: 0 });
        assert!(tokens.is_empty());

        let tokens = tokenizer.process_line("  indented", Position { row: 1, column: 0 });
        assert_eq!(tokens.len(), 1);

        match &tokens[0] {
            Token::Indent { .. } => {} // Just check it's an Indent token
            _ => panic!("Expected Indent token"),
        }
    }
}
