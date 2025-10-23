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
}
