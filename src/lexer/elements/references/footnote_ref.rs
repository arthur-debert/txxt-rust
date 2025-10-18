//! Footnote reference tokenizer
//!
//! Handles parsing of footnote references in TXXT syntax:
//! - Naked numerical format: [1], [2], [42]
//! - Labeled format: [^note1], [^detailed-explanation]
//!
//! Footnote references provide links to footnote content and supplementary information.

use crate::ast::scanner_tokens::{Position, SourceSpan, ScannerToken};

/// Trait for footnote reference lexing
pub trait FootnoteRefLexer {
    /// Get current position
    fn current_position(&self) -> Position;

    /// Advance to next character and return it
    fn advance(&mut self) -> Option<char>;

    /// Peek at current character
    fn peek(&self) -> Option<char>;

    /// Peek at character at offset from current position
    fn peek_at(&self, offset: usize) -> Option<char>;

    /// Get current row (line number)
    fn row(&self) -> usize;

    /// Get current column
    fn column(&self) -> usize;

    /// Get current position index
    fn position(&self) -> usize;

    /// Backtrack to a saved position
    fn backtrack(&mut self, position: usize, row: usize, column: usize);
}

/// Read a footnote reference token ([1], [2], [^label])
pub fn read_footnote_ref<L>(lexer: &mut L) -> Option<ScannerToken>
where
    L: FootnoteRefLexer,
{
    let start_pos = lexer.current_position();

    // Must start with [
    if lexer.peek() != Some('[') {
        return None;
    }

    let saved_position = lexer.position();
    let saved_row = lexer.row();
    let saved_column = lexer.column();

    lexer.advance(); // Consume [

    let mut content = String::new();
    let mut found_closing = false;

    // Read content until ] or end of line
    while let Some(ch) = lexer.peek() {
        if ch == ']' {
            lexer.advance(); // Consume ]
            found_closing = true;
            break;
        } else if ch == '\n' || ch == '\r' {
            // Footnote references cannot span lines
            break;
        } else {
            content.push(ch);
            lexer.advance();
        }
    }

    if !found_closing || content.is_empty() {
        // Not a valid footnote reference, backtrack
        lexer.backtrack(saved_position, saved_row, saved_column);
        return None;
    }

    // Check if this is a valid footnote reference
    if let Some(footnote_type) = classify_footnote_content(&content) {
        Some(ScannerToken::FootnoteRef {
            footnote_type,
            span: SourceSpan {
                start: start_pos,
                end: lexer.current_position(),
            },
        })
    } else {
        // Not a footnote reference, backtrack
        lexer.backtrack(saved_position, saved_row, saved_column);
        None
    }
}

/// Footnote reference type classification
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FootnoteType {
    /// Naked numerical footnote: [1], [2], [42]
    Naked(u32),
    /// Labeled footnote: [^note1], [^explanation]
    Labeled(String),
}

/// Classify footnote content to determine if it's a valid footnote reference
fn classify_footnote_content(content: &str) -> Option<FootnoteType> {
    if content.is_empty() {
        return None;
    }

    // Check for labeled footnote format: ^label
    if let Some(label) = content.strip_prefix('^') {
        if is_valid_footnote_label(label) {
            return Some(FootnoteType::Labeled(label.to_string()));
        }
        return None;
    }

    // Check for naked numerical format: pure digits
    if content.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(number) = content.parse::<u32>() {
            if number > 0 {
                // Valid footnote number (must be > 0)
                return Some(FootnoteType::Naked(number));
            }
        }
    }

    None
}

/// Validate footnote label format
fn is_valid_footnote_label(label: &str) -> bool {
    if label.is_empty() {
        return false;
    }

    // Label must start with letter or underscore
    let first_char = label.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    // Rest can be alphanumeric, underscore, or dash
    label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_footnote_content() {
        // Naked numerical footnotes
        assert_eq!(classify_footnote_content("1"), Some(FootnoteType::Naked(1)));
        assert_eq!(
            classify_footnote_content("42"),
            Some(FootnoteType::Naked(42))
        );
        assert_eq!(
            classify_footnote_content("123"),
            Some(FootnoteType::Naked(123))
        );

        // Labeled footnotes
        assert_eq!(
            classify_footnote_content("^note1"),
            Some(FootnoteType::Labeled("note1".to_string()))
        );
        assert_eq!(
            classify_footnote_content("^detailed-explanation"),
            Some(FootnoteType::Labeled("detailed-explanation".to_string()))
        );
        assert_eq!(
            classify_footnote_content("^methodology_note"),
            Some(FootnoteType::Labeled("methodology_note".to_string()))
        );

        // Invalid cases
        assert_eq!(classify_footnote_content(""), None);
        assert_eq!(classify_footnote_content("0"), None); // Zero not allowed
        assert_eq!(classify_footnote_content("^"), None); // Empty label
        assert_eq!(classify_footnote_content("^123"), None); // Label can't start with digit
        assert_eq!(classify_footnote_content("^-invalid"), None); // Label can't start with dash
        assert_eq!(classify_footnote_content("text"), None); // Not a footnote pattern
        assert_eq!(classify_footnote_content("1a"), None); // Mixed alphanumeric
    }

    #[test]
    fn test_is_valid_footnote_label() {
        // Valid labels
        assert!(is_valid_footnote_label("note1"));
        assert!(is_valid_footnote_label("detailed-explanation"));
        assert!(is_valid_footnote_label("methodology_note"));
        assert!(is_valid_footnote_label("_private"));
        assert!(is_valid_footnote_label("Note123"));

        // Invalid labels
        assert!(!is_valid_footnote_label(""));
        assert!(!is_valid_footnote_label("123note"));
        assert!(!is_valid_footnote_label("-invalid"));
        assert!(!is_valid_footnote_label("invalid.label"));
        assert!(!is_valid_footnote_label("invalid label"));
    }
}
