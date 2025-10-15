//! Reference marker detection and parsing
//!
//! Handles parsing of reference markers in TXXT syntax: [target], [@citation], [#section], [1]
//! Reference markers are inline elements that link to other content.

use crate::ast::elements::references::reference_types::ReferenceClassifier;
use crate::ast::tokens::{Position, SourceSpan, Token};

/// Trait for reference marker lexing
pub trait ReferenceLexer {
    /// Get current position
    fn current_position(&self) -> Position;

    /// Advance to next character and return it
    fn advance(&mut self) -> Option<char>;

    /// Peek at current character
    fn peek(&self) -> Option<char>;

    /// Get current row (line number)
    fn row(&self) -> usize;

    /// Get current column
    fn column(&self) -> usize;

    /// Get current position index
    fn position(&self) -> usize;

    /// Get input as character slice
    fn input(&self) -> &[char];

    /// Get reference classifier
    fn ref_classifier(&self) -> &ReferenceClassifier;

    /// Read reference markers ([target], [@citation], [#section], [1])
    fn read_ref_marker(&mut self) -> Option<Token> {
        let start_pos = self.current_position();

        // Must start with [
        if self.peek() != Some('[') {
            return None;
        }

        let saved_position = self.position();
        let saved_row = self.row();
        let saved_column = self.column();

        self.advance(); // Consume [

        let mut content = String::new();
        let mut found_closing = false;

        // Read content until ] or end of line
        while let Some(ch) = self.peek() {
            if ch == ']' {
                self.advance(); // Consume ]
                found_closing = true;
                break;
            } else if ch == '\n' || ch == '\r' {
                // Reference markers cannot span lines
                break;
            } else {
                content.push(ch);
                self.advance();
            }
        }

        if !found_closing || content.is_empty() {
            // Not a valid reference marker, backtrack
            self.backtrack(saved_position, saved_row, saved_column);
            return None;
        }

        // Validate content patterns
        if self.is_valid_ref_content(&content) {
            Some(Token::RefMarker {
                content,
                span: SourceSpan {
                    start: start_pos,
                    end: self.current_position(),
                },
            })
        } else {
            // Invalid reference content, backtrack
            self.backtrack(saved_position, saved_row, saved_column);
            None
        }
    }

    /// Check if reference content is valid (basic alphanumeric validation only)
    fn is_valid_ref_content(&self, content: &str) -> bool {
        // Only basic validation - at least one alphanumeric character
        // Detailed type classification happens during parsing phase
        self.ref_classifier().is_valid_reference_content(content)
    }

    /// Backtrack to a saved position
    fn backtrack(&mut self, position: usize, row: usize, column: usize);
}
