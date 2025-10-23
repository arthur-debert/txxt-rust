//! # Level 1: Delimiter Matchers
//!
//! Implementations of DelimiterMatcher for all inline element types.
//! These matchers identify spans enclosed by specific delimiters without
//! interpreting their content or building AST nodes.
//!
//! ## Matchers
//!
//! - `BoldMatcher`: Matches `*...*` for bold/strong text
//! - `ItalicMatcher`: Matches `_..._` for italic/emphasis text
//! - `CodeMatcher`: Matches `` `...` `` for code spans
//! - `MathMatcher`: Matches `#...#` for math expressions
//! - `ReferenceMatcher`: Matches `[...]` for all reference types

use crate::cst::ScannerToken;
use crate::semantic::elements::inlines::pipeline::{DelimiterMatcher, SpanMatch};

/// Check if token sequence contains newlines (violates single-line constraint)
fn contains_newline(tokens: &[ScannerToken]) -> bool {
    tokens.iter().any(|token| {
        matches!(
            token,
            ScannerToken::Newline { .. } | ScannerToken::BlankLine { .. }
        )
    })
}

/// Find matching closing delimiter in token stream
fn find_closing<P>(tokens: &[ScannerToken], start: usize, predicate: P) -> Option<usize>
where
    P: Fn(&ScannerToken) -> bool,
{
    tokens[start..]
        .iter()
        .position(predicate)
        .map(|pos| start + pos)
}

/// Bold delimiter matcher - matches `*...*`
pub struct BoldMatcher;

impl DelimiterMatcher for BoldMatcher {
    fn name(&self) -> &str {
        "bold"
    }

    fn can_start(&self, token: &ScannerToken) -> bool {
        token.is_bold_delimiter()
    }

    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch> {
        // Find closing delimiter
        if let Some(end) = find_closing(tokens, start + 1, |t| t.is_bold_delimiter()) {
            let inner_tokens = tokens[start + 1..end].to_vec();

            // Enforce single-line constraint
            if contains_newline(&inner_tokens) {
                return None;
            }

            // Must have content
            if inner_tokens.is_empty() {
                return None;
            }

            Some(SpanMatch {
                start,
                end: end + 1, // Include closing delimiter
                matcher_name: self.name().to_string(),
                inner_tokens,
                full_tokens: tokens[start..=end].to_vec(),
            })
        } else {
            None
        }
    }
}

/// Italic delimiter matcher - matches `_..._`
pub struct ItalicMatcher;

impl DelimiterMatcher for ItalicMatcher {
    fn name(&self) -> &str {
        "italic"
    }

    fn can_start(&self, token: &ScannerToken) -> bool {
        token.is_italic_delimiter()
    }

    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch> {
        // Find closing delimiter
        if let Some(end) = find_closing(tokens, start + 1, |t| t.is_italic_delimiter()) {
            let inner_tokens = tokens[start + 1..end].to_vec();

            // Enforce single-line constraint
            if contains_newline(&inner_tokens) {
                return None;
            }

            // Must have content
            if inner_tokens.is_empty() {
                return None;
            }

            Some(SpanMatch {
                start,
                end: end + 1,
                matcher_name: self.name().to_string(),
                inner_tokens,
                full_tokens: tokens[start..=end].to_vec(),
            })
        } else {
            None
        }
    }
}

/// Code delimiter matcher - matches `` `...` ``
pub struct CodeMatcher;

impl DelimiterMatcher for CodeMatcher {
    fn name(&self) -> &str {
        "code"
    }

    fn can_start(&self, token: &ScannerToken) -> bool {
        token.is_code_delimiter()
    }

    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch> {
        // Find closing delimiter
        if let Some(end) = find_closing(tokens, start + 1, |t| t.is_code_delimiter()) {
            let inner_tokens = tokens[start + 1..end].to_vec();

            // Enforce single-line constraint
            if contains_newline(&inner_tokens) {
                return None;
            }

            // Must have content
            if inner_tokens.is_empty() {
                return None;
            }

            Some(SpanMatch {
                start,
                end: end + 1,
                matcher_name: self.name().to_string(),
                inner_tokens,
                full_tokens: tokens[start..=end].to_vec(),
            })
        } else {
            None
        }
    }
}

/// Math delimiter matcher - matches `#...#`
pub struct MathMatcher;

impl DelimiterMatcher for MathMatcher {
    fn name(&self) -> &str {
        "math"
    }

    fn can_start(&self, token: &ScannerToken) -> bool {
        token.is_math_delimiter()
    }

    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch> {
        // Find closing delimiter
        if let Some(end) = find_closing(tokens, start + 1, |t| t.is_math_delimiter()) {
            let inner_tokens = tokens[start + 1..end].to_vec();

            // Enforce single-line constraint
            if contains_newline(&inner_tokens) {
                return None;
            }

            // Must have content
            if inner_tokens.is_empty() {
                return None;
            }

            Some(SpanMatch {
                start,
                end: end + 1,
                matcher_name: self.name().to_string(),
                inner_tokens,
                full_tokens: tokens[start..=end].to_vec(),
            })
        } else {
            None
        }
    }
}

/// Reference delimiter matcher - matches `[...]`
pub struct ReferenceMatcher;

impl DelimiterMatcher for ReferenceMatcher {
    fn name(&self) -> &str {
        "reference"
    }

    fn can_start(&self, token: &ScannerToken) -> bool {
        matches!(token, ScannerToken::Text { content, .. } if content == "[")
    }

    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch> {
        // Find closing bracket
        if let Some(end) = find_closing(
            tokens,
            start + 1,
            |t| matches!(t, ScannerToken::Text { content, .. } if content == "]"),
        ) {
            let inner_tokens = tokens[start + 1..end].to_vec();

            // Enforce single-line constraint
            if contains_newline(&inner_tokens) {
                return None;
            }

            // Must have content
            if inner_tokens.is_empty() {
                return None;
            }

            Some(SpanMatch {
                start,
                end: end + 1,
                matcher_name: self.name().to_string(),
                inner_tokens,
                full_tokens: tokens[start..=end].to_vec(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn create_bold_delimiter() -> ScannerToken {
        ScannerToken::BoldDelimiter {
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 1 },
            },
        }
    }

    fn create_text(content: &str) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position {
                    row: 0,
                    column: content.len(),
                },
            },
        }
    }

    fn create_newline() -> ScannerToken {
        ScannerToken::Newline {
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 1 },
            },
        }
    }

    #[test]
    fn test_bold_matcher_simple() {
        let tokens = vec![
            create_bold_delimiter(),
            create_text("hello"),
            create_bold_delimiter(),
        ];

        let matcher = BoldMatcher;
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 3);
        assert_eq!(span.inner_tokens.len(), 1);
    }

    #[test]
    fn test_bold_matcher_rejects_newline() {
        let tokens = vec![
            create_bold_delimiter(),
            create_text("hello"),
            create_newline(),
            create_text("world"),
            create_bold_delimiter(),
        ];

        let matcher = BoldMatcher;
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_none()); // Should reject due to newline
    }

    #[test]
    fn test_bold_matcher_rejects_empty() {
        let tokens = vec![create_bold_delimiter(), create_bold_delimiter()];

        let matcher = BoldMatcher;
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_none()); // Should reject empty content
    }

    #[test]
    fn test_reference_matcher_simple() {
        let tokens = vec![create_text("["), create_text("@citation"), create_text("]")];

        let matcher = ReferenceMatcher;
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 3);
        assert_eq!(span.inner_tokens.len(), 1);
    }
}
