//! Indentation tracking and indent/dedent token generation
//!
//! This module implements the indentation-based container architecture
//! fundamental to TXXT. It tracks indentation levels and generates
//! Indent and Dedent tokens as defined in the specification.
//!
//! ## Key Principles
//!
//! - **Container-based structure**: Indentation determines container boundaries
//! - **Line-oriented processing**: Each line's indentation is processed once at column 0
//! - **Verbatim awareness**: Skips indentation processing for verbatim content
//! - **Multiple dedent support**: Generates multiple Dedent tokens for multi-level decreases

use crate::ast::tokens::{Position, SourceSpan, Token};
use std::collections::VecDeque;

/// Standard indentation size (4 spaces) - matches verbatim_scanner.rs
pub const INDENT_SIZE: usize = 4;

/// Tab width for tab-to-space conversion  
pub const TAB_WIDTH: usize = 4;

/// Indentation tracker that generates Indent and Dedent tokens
#[derive(Debug, Clone)]
pub struct IndentationTracker {
    /// Stack of indentation levels (in spaces)
    indent_stack: Vec<usize>,
    /// Pending dedent tokens to emit
    pending_dedents: VecDeque<Token>,
    /// Current position in source
    current_position: Position,
}

impl IndentationTracker {
    /// Create a new indentation tracker
    pub fn new() -> Self {
        Self {
            indent_stack: vec![0], // Start with base level 0
            pending_dedents: VecDeque::new(),
            current_position: Position { row: 0, column: 0 },
        }
    }

    /// Update current position
    pub fn set_position(&mut self, position: Position) {
        self.current_position = position;
    }

    /// Process a line's indentation and generate appropriate tokens
    ///
    /// This is the main entry point for indentation processing. It should be called
    /// for each line at column 0, but only for lines that are not part of verbatim content.
    pub fn process_line_indentation(&mut self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();

        // First emit any pending dedents
        while let Some(dedent) = self.pending_dedents.pop_front() {
            tokens.push(dedent);
        }

        // Skip empty lines - they don't affect indentation structure
        if line.trim().is_empty() {
            return tokens;
        }

        // Normalize tabs to spaces and calculate indentation level
        let normalized_line = normalize_indentation(line, TAB_WIDTH);
        let current_indent = count_leading_spaces(&normalized_line);
        let previous_indent = *self.indent_stack.last().unwrap_or(&0);

        if current_indent > previous_indent {
            // Increased indentation - emit Indent token
            self.indent_stack.push(current_indent);
            tokens.push(Token::Indent {
                span: SourceSpan {
                    start: Position {
                        row: self.current_position.row,
                        column: 0,
                    },
                    end: Position {
                        row: self.current_position.row,
                        column: current_indent,
                    },
                },
            });
        } else if current_indent < previous_indent {
            // Decreased indentation - emit Dedent tokens for each level we've left
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= current_indent {
                    break;
                }

                self.indent_stack.pop();
                self.pending_dedents.push_back(Token::Dedent {
                    span: SourceSpan {
                        start: Position {
                            row: self.current_position.row,
                            column: current_indent,
                        },
                        end: Position {
                            row: self.current_position.row,
                            column: stack_indent,
                        },
                    },
                });
            }

            // Emit pending dedents immediately
            while let Some(dedent) = self.pending_dedents.pop_front() {
                tokens.push(dedent);
            }

            // Check if we need to record a new indentation level
            if !self.indent_stack.contains(&current_indent) && current_indent > 0 {
                self.indent_stack.push(current_indent);
                tokens.push(Token::Indent {
                    span: SourceSpan {
                        start: Position {
                            row: self.current_position.row,
                            column: 0,
                        },
                        end: Position {
                            row: self.current_position.row,
                            column: current_indent,
                        },
                    },
                });
            }
        }
        // If current_indent == previous_indent, no tokens needed

        tokens
    }

    /// Finalize indentation processing (emit remaining dedents)
    ///
    /// This should be called at the end of document processing to ensure
    /// all indentation levels are properly closed.
    pub fn finalize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        // Emit pending dedents
        while let Some(dedent) = self.pending_dedents.pop_front() {
            tokens.push(dedent);
        }

        // Emit dedents for all remaining levels except base level (0)
        while self.indent_stack.len() > 1 {
            let stack_indent = self.indent_stack.pop().unwrap();
            tokens.push(Token::Dedent {
                span: SourceSpan {
                    start: Position {
                        row: self.current_position.row,
                        column: 0,
                    },
                    end: Position {
                        row: self.current_position.row,
                        column: stack_indent,
                    },
                },
            });
        }

        tokens
    }

    /// Get current indentation level
    pub fn current_level(&self) -> usize {
        *self.indent_stack.last().unwrap_or(&0)
    }

    /// Check if we're at base indentation level
    pub fn is_at_base_level(&self) -> bool {
        self.indent_stack.len() <= 1
    }

    /// Get the current indentation stack depth (for debugging)
    pub fn stack_depth(&self) -> usize {
        self.indent_stack.len()
    }
}

impl Default for IndentationTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Count leading spaces in a line
fn count_leading_spaces(line: &str) -> usize {
    line.chars().take_while(|&c| c == ' ').count()
}

/// Normalize tabs to spaces
///
/// Converts tabs to the appropriate number of spaces based on tab_width.
/// This ensures consistent indentation calculation regardless of whether
/// the source uses tabs or spaces.
pub fn normalize_indentation(line: &str, tab_width: usize) -> String {
    line.chars()
        .map(|c| {
            if c == '\t' {
                " ".repeat(tab_width)
            } else {
                c.to_string()
            }
        })
        .collect()
}

/// Check if an indentation level is valid (multiple of INDENT_SIZE)
pub fn is_valid_indentation_level(level: usize) -> bool {
    #[allow(clippy::assertions_on_constants)]
    {
        debug_assert!(
            INDENT_SIZE > 0,
            "INDENT_SIZE must be greater than zero to avoid division by zero"
        );
    }
    level % INDENT_SIZE == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_leading_spaces() {
        assert_eq!(count_leading_spaces(""), 0);
        assert_eq!(count_leading_spaces("hello"), 0);
        assert_eq!(count_leading_spaces("  hello"), 2);
        assert_eq!(count_leading_spaces("    indented"), 4);
        assert_eq!(count_leading_spaces("        deeply indented"), 8);
    }

    #[test]
    fn test_normalize_indentation() {
        assert_eq!(normalize_indentation("\thello", 4), "    hello");
        assert_eq!(normalize_indentation("  \tworld", 4), "      world");
        assert_eq!(normalize_indentation("normal", 4), "normal");
        assert_eq!(normalize_indentation("\t\tdeep", 4), "        deep");
    }

    #[test]
    fn test_is_valid_indentation_level() {
        assert!(is_valid_indentation_level(0));
        assert!(is_valid_indentation_level(4));
        assert!(is_valid_indentation_level(8));
        assert!(!is_valid_indentation_level(2));
        assert!(!is_valid_indentation_level(6));
        assert!(!is_valid_indentation_level(1));
    }

    #[test]
    fn test_basic_indentation() {
        let mut tracker = IndentationTracker::new();
        tracker.set_position(Position { row: 0, column: 0 });

        // Line with no indentation
        let tokens = tracker.process_line_indentation("hello");
        assert!(tokens.is_empty());

        // Line with indentation
        tracker.set_position(Position { row: 1, column: 0 });
        let tokens = tracker.process_line_indentation("    indented");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Token::Indent { span } => {
                assert_eq!(span.start.row, 1);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.column, 4);
            }
            _ => panic!("Expected Indent token, got {:?}", tokens[0]),
        }
    }

    #[test]
    fn test_dedentation() {
        let mut tracker = IndentationTracker::new();

        // Increase indentation step by step
        tracker.set_position(Position { row: 0, column: 0 });
        tracker.process_line_indentation("    level 1");

        tracker.set_position(Position { row: 1, column: 0 });
        tracker.process_line_indentation("        level 2");

        // Decrease indentation back to base
        tracker.set_position(Position { row: 2, column: 0 });
        let tokens = tracker.process_line_indentation("back to base");
        assert_eq!(tokens.len(), 2); // Two dedents

        for token in &tokens {
            match token {
                Token::Dedent { .. } => {}
                _ => panic!("Expected Dedent token, got {:?}", token),
            }
        }
    }

    #[test]
    fn test_empty_lines() {
        let mut tracker = IndentationTracker::new();
        tracker.set_position(Position { row: 0, column: 0 });

        // Empty lines shouldn't affect indentation
        let tokens = tracker.process_line_indentation("");
        assert!(tokens.is_empty());

        let tokens = tracker.process_line_indentation("   ");
        assert!(tokens.is_empty());

        let tokens = tracker.process_line_indentation("\t  ");
        assert!(tokens.is_empty());

        // Non-empty line should still work
        tracker.set_position(Position { row: 1, column: 0 });
        let tokens = tracker.process_line_indentation("    content");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Token::Indent { .. } => {}
            _ => panic!("Expected Indent token"),
        }
    }

    #[test]
    fn test_finalize() {
        let mut tracker = IndentationTracker::new();

        // Create nested indentation
        tracker.set_position(Position { row: 0, column: 0 });
        tracker.process_line_indentation("    level 1");

        tracker.set_position(Position { row: 1, column: 0 });
        tracker.process_line_indentation("        level 2");

        // Finalize should emit dedents for all levels
        tracker.set_position(Position { row: 2, column: 0 });
        let tokens = tracker.finalize();
        assert_eq!(tokens.len(), 2); // Two dedents to close all levels

        for token in &tokens {
            match token {
                Token::Dedent { .. } => {}
                _ => panic!("Expected Dedent token from finalize"),
            }
        }
    }

    #[test]
    fn test_tab_normalization_integration() {
        let mut tracker = IndentationTracker::new();
        tracker.set_position(Position { row: 0, column: 0 });

        // Line with tab indentation (should be treated as 4 spaces)
        let tokens = tracker.process_line_indentation("\tcontent");
        assert_eq!(tokens.len(), 1);

        match &tokens[0] {
            Token::Indent { span } => {
                assert_eq!(span.end.column, 4); // Tab normalized to 4 spaces
            }
            _ => panic!("Expected Indent token"),
        }
    }

    #[test]
    fn test_multiple_dedent_levels() {
        let mut tracker = IndentationTracker::new();

        // Build up multiple levels
        tracker.set_position(Position { row: 0, column: 0 });
        tracker.process_line_indentation("    level 1");

        tracker.set_position(Position { row: 1, column: 0 });
        tracker.process_line_indentation("        level 2");

        tracker.set_position(Position { row: 2, column: 0 });
        tracker.process_line_indentation("            level 3");

        // Jump back to base level - should generate 3 dedents
        tracker.set_position(Position { row: 3, column: 0 });
        let tokens = tracker.process_line_indentation("base level");
        assert_eq!(tokens.len(), 3);

        for token in &tokens {
            match token {
                Token::Dedent { .. } => {}
                _ => panic!("Expected Dedent token"),
            }
        }
    }
}
