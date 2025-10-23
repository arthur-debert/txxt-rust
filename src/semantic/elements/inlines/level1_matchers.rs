//! # Level 1: Delimiter Matchers
//!
//! Generic delimiter matcher that can be configured for any inline element type
//! with start and end delimiters. This eliminates code duplication across all
//! simple delimiter-based inline elements.
//!
//! ## Architecture
//!
//! Instead of separate matcher structs for each inline type, we use a single
//! `GenericDelimiterMatcher` configured with:
//! - Name: Identifier for the inline type
//! - Start predicate: Function to check if token is a start delimiter
//! - End predicate: Function to check if token is an end delimiter
//!
//! ## Examples
//!
//! - Bold: start=`*`, end=`*` (same delimiter)
//! - Reference: start=`[`, end=`]` (different delimiters)

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

/// Generic delimiter matcher - matches any `start...end` pattern
///
/// This single matcher handles all delimiter-based inline elements by taking
/// predicates for start and end delimiters. This eliminates code duplication
/// and makes it easy to add new inline types.
///
/// # Examples
///
/// ```rust,ignore
/// // Bold: same delimiter for start and end
/// let bold = GenericDelimiterMatcher::new(
///     "bold",
///     |t| t.is_bold_delimiter(),
///     |t| t.is_bold_delimiter(),
/// );
///
/// // Reference: different delimiters
/// let reference = GenericDelimiterMatcher::new(
///     "reference",
///     |t| matches!(t, ScannerToken::Text { content, .. } if content == "["),
///     |t| matches!(t, ScannerToken::Text { content, .. } if content == "]"),
/// );
/// ```
pub struct GenericDelimiterMatcher {
    name: String,
    start_predicate: fn(&ScannerToken) -> bool,
    end_predicate: fn(&ScannerToken) -> bool,
}

impl GenericDelimiterMatcher {
    /// Create a new generic delimiter matcher
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the inline type (e.g., "bold", "reference")
    /// * `start_predicate` - Function to check if token is a start delimiter
    /// * `end_predicate` - Function to check if token is an end delimiter
    pub fn new(
        name: &str,
        start_predicate: fn(&ScannerToken) -> bool,
        end_predicate: fn(&ScannerToken) -> bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            start_predicate,
            end_predicate,
        }
    }
}

impl DelimiterMatcher for GenericDelimiterMatcher {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_start(&self, token: &ScannerToken) -> bool {
        (self.start_predicate)(token)
    }

    fn match_span(&self, tokens: &[ScannerToken], start: usize) -> Option<SpanMatch> {
        // Find closing delimiter
        if let Some(end) = find_closing(tokens, start + 1, self.end_predicate) {
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
                matcher_name: self.name.clone(),
                inner_tokens,
                full_tokens: tokens[start..=end].to_vec(),
            })
        } else {
            None
        }
    }
}

// Factory functions for common matchers

/// Create a bold delimiter matcher (matches `*...*`)
pub fn bold_matcher() -> GenericDelimiterMatcher {
    GenericDelimiterMatcher::new(
        "bold",
        ScannerToken::is_bold_delimiter,
        ScannerToken::is_bold_delimiter,
    )
}

/// Create an italic delimiter matcher (matches `_..._`)
pub fn italic_matcher() -> GenericDelimiterMatcher {
    GenericDelimiterMatcher::new(
        "italic",
        ScannerToken::is_italic_delimiter,
        ScannerToken::is_italic_delimiter,
    )
}

/// Create a code delimiter matcher (matches `` `...` ``)
pub fn code_matcher() -> GenericDelimiterMatcher {
    GenericDelimiterMatcher::new(
        "code",
        ScannerToken::is_code_delimiter,
        ScannerToken::is_code_delimiter,
    )
}

/// Create a math delimiter matcher (matches `#...#`)
pub fn math_matcher() -> GenericDelimiterMatcher {
    GenericDelimiterMatcher::new(
        "math",
        ScannerToken::is_math_delimiter,
        ScannerToken::is_math_delimiter,
    )
}

/// Create a reference delimiter matcher (matches `[...]`)
pub fn reference_matcher() -> GenericDelimiterMatcher {
    GenericDelimiterMatcher::new(
        "reference",
        |t| matches!(t, ScannerToken::Text { content, .. } if content == "["),
        |t| matches!(t, ScannerToken::Text { content, .. } if content == "]"),
    )
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

        let matcher = bold_matcher();
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

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_none()); // Should reject due to newline
    }

    #[test]
    fn test_bold_matcher_rejects_empty() {
        let tokens = vec![create_bold_delimiter(), create_bold_delimiter()];

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_none()); // Should reject empty content
    }

    #[test]
    fn test_reference_matcher_simple() {
        let tokens = vec![create_text("["), create_text("@citation"), create_text("]")];

        let matcher = reference_matcher();
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 3);
        assert_eq!(span.inner_tokens.len(), 1);
    }

    // ============================================================================
    // Unit Tests for GenericDelimiterMatcher Infrastructure
    // ============================================================================

    #[test]
    fn test_generic_matcher_with_same_delimiters() {
        // Test that generic matcher works with same start/end delimiter (like bold)
        let tokens = vec![
            create_bold_delimiter(),
            create_text("content"),
            create_bold_delimiter(),
        ];

        let matcher = GenericDelimiterMatcher::new(
            "test-same",
            ScannerToken::is_bold_delimiter,
            ScannerToken::is_bold_delimiter,
        );

        let span = matcher.match_span(&tokens, 0);
        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.matcher_name, "test-same");
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 3);
        assert_eq!(span.inner_tokens.len(), 1);
    }

    #[test]
    fn test_generic_matcher_with_different_delimiters() {
        // Test that generic matcher works with different start/end delimiters (like reference)
        let tokens = vec![create_text("["), create_text("content"), create_text("]")];

        let matcher = GenericDelimiterMatcher::new(
            "test-different",
            |t| matches!(t, ScannerToken::Text { content, .. } if content == "["),
            |t| matches!(t, ScannerToken::Text { content, .. } if content == "]"),
        );

        let span = matcher.match_span(&tokens, 0);
        assert!(span.is_some());
        let span = span.unwrap();
        assert_eq!(span.matcher_name, "test-different");
    }

    #[test]
    fn test_generic_matcher_preserves_full_tokens() {
        // Test that full_tokens includes delimiters
        let tokens = vec![
            create_bold_delimiter(),
            create_text("hello"),
            create_text(" "),
            create_text("world"),
            create_bold_delimiter(),
        ];

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0).unwrap();

        assert_eq!(span.full_tokens.len(), 5); // All tokens including delimiters
        assert_eq!(span.inner_tokens.len(), 3); // Only content tokens
    }

    #[test]
    fn test_generic_matcher_can_start_check() {
        let bold_token = create_bold_delimiter();
        let text_token = create_text("not a delimiter");

        let matcher = bold_matcher();

        assert!(matcher.can_start(&bold_token));
        assert!(!matcher.can_start(&text_token));
    }

    #[test]
    fn test_all_factory_functions_create_working_matchers() {
        // Test that all factory functions produce working matchers
        let factories = vec![
            ("bold", bold_matcher()),
            ("italic", italic_matcher()),
            ("code", code_matcher()),
            ("math", math_matcher()),
            ("reference", reference_matcher()),
        ];

        for (expected_name, matcher) in factories {
            assert_eq!(matcher.name(), expected_name);
        }
    }

    // ============================================================================
    // Edge Case Tests
    // ============================================================================

    #[test]
    fn test_multiple_bold_spans_in_sequence() {
        let tokens = vec![
            create_bold_delimiter(),
            create_text("first"),
            create_bold_delimiter(),
            create_text(" "),
            create_bold_delimiter(),
            create_text("second"),
            create_bold_delimiter(),
        ];

        let matcher = bold_matcher();

        // First span
        let span1 = matcher.match_span(&tokens, 0).unwrap();
        assert_eq!(span1.start, 0);
        assert_eq!(span1.end, 3);

        // Second span starts at position 4
        let span2 = matcher.match_span(&tokens, 4).unwrap();
        assert_eq!(span2.start, 4);
        assert_eq!(span2.end, 7);
    }

    #[test]
    fn test_unmatched_closing_delimiter() {
        // Closing delimiter without opening should not match
        let tokens = vec![create_text("text"), create_bold_delimiter()];

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0); // Start at text, not delimiter

        assert!(span.is_none());
    }

    #[test]
    fn test_nested_same_delimiter_not_matched() {
        // *outer *inner* text* - should match first pair only
        let tokens = vec![
            create_bold_delimiter(),
            create_text("outer "),
            create_bold_delimiter(), // This closes first bold
            create_text("inner"),
            create_bold_delimiter(), // This would be unmatched
            create_text(" text"),
            create_bold_delimiter(), // This would be unmatched
        ];

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0).unwrap();

        // Should match only first pair
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 3); // Just the first "outer " span
    }

    #[test]
    fn test_blank_line_also_rejects_span() {
        let tokens = vec![
            create_bold_delimiter(),
            create_text("line1"),
            ScannerToken::BlankLine {
                whitespace: " ".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 1 },
                },
            },
            create_text("line2"),
            create_bold_delimiter(),
        ];

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_none()); // Should reject due to blank line
    }

    #[test]
    fn test_whitespace_only_content_accepted() {
        // Content with only whitespace should still be valid (not "empty")
        let tokens = vec![
            create_bold_delimiter(),
            create_text("   "),
            create_bold_delimiter(),
        ];

        let matcher = bold_matcher();
        let span = matcher.match_span(&tokens, 0);

        assert!(span.is_some()); // Whitespace is valid content
    }
}
