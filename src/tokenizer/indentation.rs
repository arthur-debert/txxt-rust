//! Indentation tracking and indent/dedent token generation
//!
//! This module implements the indentation-based container architecture
//! fundamental to TXXT. It tracks indentation levels and generates
//! Indent and Dedent tokens as defined in docs/specs/elements/container.txxt

use crate::ast::tokens::{Position, SourceSpan, Token};
use std::collections::VecDeque;

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
    pub fn process_line_indentation(&mut self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();

        // First emit any pending dedents
        while let Some(dedent) = self.pending_dedents.pop_front() {
            tokens.push(dedent);
        }

        // Skip empty lines - they don't affect indentation
        if line.trim().is_empty() {
            return tokens;
        }

        // Calculate indentation level (count leading spaces)
        let current_indent = count_leading_spaces(line);
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
            // Decreased indentation - emit Dedent tokens
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= current_indent {
                    break;
                }

                self.indent_stack.pop();
                tokens.push(Token::Dedent {
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

            // If we dedented to a level that's not on the stack, it's an error
            // but we'll handle it gracefully by adjusting the stack
            if current_indent > 0 && !self.indent_stack.contains(&current_indent) {
                self.indent_stack.push(current_indent);
            }
        }
        // If current_indent == previous_indent, no indentation change

        tokens
    }

    /// Finalize indentation tracking (emit final dedents)
    pub fn finalize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        // Emit any pending dedents
        while let Some(dedent) = self.pending_dedents.pop_front() {
            tokens.push(dedent);
        }

        // Emit dedents for all remaining indentation levels
        while self.indent_stack.len() > 1 {
            if let Some(_level) = self.indent_stack.pop() {
                tokens.push(Token::Dedent {
                    span: SourceSpan {
                        start: self.current_position,
                        end: self.current_position,
                    },
                });
            }
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

/// Normalize tabs to spaces (tabs count as 4 spaces by default)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_leading_spaces() {
        assert_eq!(count_leading_spaces(""), 0);
        assert_eq!(count_leading_spaces("hello"), 0);
        assert_eq!(count_leading_spaces("  hello"), 2);
        assert_eq!(count_leading_spaces("    indented"), 4);
        assert_eq!(count_leading_spaces("      deeply indented"), 6);
    }

    #[test]
    fn test_basic_indentation() {
        let mut tracker = IndentationTracker::new();

        // Line with no indentation
        let tokens = tracker.process_line_indentation("hello");
        assert!(tokens.is_empty());

        // Line with indentation
        let tokens = tracker.process_line_indentation("  indented");
        assert_eq!(tokens.len(), 1);
        match &tokens[0] {
            Token::Indent { .. } => {} // Just check it's an Indent token
            _ => panic!("Expected Indent token"),
        }
    }

    #[test]
    fn test_dedentation() {
        let mut tracker = IndentationTracker::new();

        // Increase indentation
        tracker.process_line_indentation("  level 1");
        tracker.process_line_indentation("    level 2");

        // Decrease indentation
        let tokens = tracker.process_line_indentation("back to base");
        assert_eq!(tokens.len(), 2); // Two dedents

        for token in &tokens {
            match token {
                Token::Dedent { .. } => {}
                _ => panic!("Expected Dedent token"),
            }
        }
    }

    #[test]
    fn test_normalize_indentation() {
        assert_eq!(normalize_indentation("\thello", 4), "    hello");
        assert_eq!(normalize_indentation("  \tworld", 4), "      world");
        assert_eq!(normalize_indentation("normal", 4), "normal");
    }

    #[test]
    fn test_empty_lines() {
        let mut tracker = IndentationTracker::new();

        // Empty lines shouldn't affect indentation
        let tokens = tracker.process_line_indentation("");
        assert!(tokens.is_empty());

        let tokens = tracker.process_line_indentation("   ");
        assert!(tokens.is_empty());

        // Non-empty line should still work
        let tokens = tracker.process_line_indentation("  content");
        assert_eq!(tokens.len(), 1);
    }
}
