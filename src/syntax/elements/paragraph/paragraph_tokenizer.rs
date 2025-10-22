//! Paragraph element tokenization
//!
//! Implements tokenization for paragraph elements as defined in
//! docs/specs/elements/paragraph/paragraph.txxt
//!
//! Paragraphs are the fundamental text blocks containing inline content.
//! They serve as the default element type when no other structure is detected.

use crate::cst::{ScannerToken, SourceSpan};

/// Represents a paragraph with its constituent text lines
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// The text content lines that make up this paragraph
    pub lines: Vec<String>,
    /// The combined text content with normalized whitespace
    pub content: String,
    /// Source span covering the entire paragraph
    pub span: SourceSpan,
}

/// Result of paragraph parsing
#[derive(Debug, Clone, PartialEq)]
pub enum ParagraphParseResult {
    /// Valid paragraph was parsed
    ValidParagraph(Paragraph),
    /// Not a paragraph (matches another element type)
    NotParagraph,
    /// Invalid paragraph with error
    Invalid(String),
}

/// Detects if a sequence of tokens represents a paragraph
///
/// Paragraph recognition (default element type):
/// - Line contains text content (not whitespace-only)
/// - Line does not match any other block element pattern
/// - Continues until blank line or indentation change
pub fn detect_paragraph(tokens: &[ScannerToken]) -> ParagraphParseResult {
    if tokens.is_empty() {
        return ParagraphParseResult::NotParagraph;
    }

    // Check if this looks like another block element type
    if looks_like_other_element(tokens) {
        return ParagraphParseResult::NotParagraph;
    }

    // Extract text content from tokens
    let content = extract_paragraph_content(tokens);

    if content.trim().is_empty() {
        return ParagraphParseResult::NotParagraph;
    }

    // Calculate span
    let span = if tokens.is_empty() {
        return ParagraphParseResult::NotParagraph;
    } else {
        SourceSpan {
            start: tokens[0].span().start,
            end: tokens.last().unwrap().span().end,
        }
    };

    ParagraphParseResult::ValidParagraph(Paragraph {
        lines: vec![content.clone()], // Single line for now
        content,
        span,
    })
}

/// Checks if tokens match patterns for other block elements
///
/// This helps ensure paragraphs serve as the fallback element type
fn looks_like_other_element(tokens: &[ScannerToken]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Check for sequence markers that might indicate lists or sessions
    if let Some(first_token) = tokens.first() {
        match first_token {
            ScannerToken::SequenceMarker { .. } => {
                // Could be list item or session - let other parsers handle it
                true
            }
            ScannerToken::TxxtMarker { .. } => {
                // Annotation or definition block
                true
            }
            ScannerToken::VerbatimBlockStart { .. } => {
                // Verbatim block
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Extracts text content from paragraph tokens
fn extract_paragraph_content(tokens: &[ScannerToken]) -> String {
    tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::Text { content, .. } => Some(content.as_str()),
            ScannerToken::Identifier { content, .. } => Some(content.as_str()),
            ScannerToken::SequenceMarker { marker_type, .. } => Some(marker_type.content()),
            // Include most content-bearing tokens but skip structural ones
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Collects multiple lines into a single paragraph
///
/// Handles line continuation rules:
/// - Lines at same indentation continue the paragraph
/// - Blank line terminates the paragraph
/// - Indentation change ends the paragraph
pub fn collect_paragraph_lines(line_tokens: &[Vec<ScannerToken>]) -> Option<Paragraph> {
    if line_tokens.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    let mut combined_content = Vec::new();

    for line in line_tokens {
        let line_content = extract_paragraph_content(line);
        if line_content.trim().is_empty() {
            // Blank line terminates paragraph
            break;
        }

        lines.push(line_content.clone());
        combined_content.push(line_content);
    }

    if lines.is_empty() {
        return None;
    }

    // Normalize whitespace: join lines with spaces, collapse multiple spaces
    let content = normalize_paragraph_whitespace(&combined_content.join(" "));

    // Calculate span from first to last token
    let first_line = &line_tokens[0];
    let last_line = &line_tokens[lines.len() - 1];

    if first_line.is_empty() || last_line.is_empty() {
        return None;
    }

    let span = SourceSpan {
        start: first_line[0].span().start,
        end: last_line.last().unwrap().span().end,
    };

    Some(Paragraph {
        lines,
        content,
        span,
    })
}

/// Normalizes whitespace in paragraph content
///
/// Rules:
/// - Collapse multiple consecutive spaces to single space
/// - Trim leading and trailing whitespace
/// - Preserve formatting-significant whitespace in inline elements
fn normalize_paragraph_whitespace(text: &str) -> String {
    // Split into words and rejoin with single spaces
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Checks if a line should terminate the current paragraph
///
/// Termination conditions:
/// - Blank line (whitespace-only)
/// - Line matching another block element pattern
/// - Indentation change
pub fn should_terminate_paragraph(tokens: &[ScannerToken]) -> bool {
    if tokens.is_empty() {
        return true; // Empty line terminates
    }

    // Check for whitespace-only line
    let has_content = tokens.iter().any(|token| {
        matches!(
            token,
            ScannerToken::Text { .. }
                | ScannerToken::Identifier { .. }
                | ScannerToken::SequenceMarker { .. }
                | ScannerToken::TxxtMarker { .. }
        )
    });

    if !has_content {
        return true; // Whitespace-only line
    }

    // Check for indentation change
    if tokens.iter().any(|token| {
        matches!(
            token,
            ScannerToken::Indent { .. } | ScannerToken::Dedent { .. }
        )
    }) {
        return true;
    }

    // Check if this looks like start of another element
    looks_like_other_element(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::Position;

    fn create_test_span() -> SourceSpan {
        SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 10 },
        }
    }

    fn create_text_token(content: &str) -> ScannerToken {
        ScannerToken::Text {
            content: content.to_string(),
            span: create_test_span(),
        }
    }

    fn create_sequence_token(content: &str) -> ScannerToken {
        ScannerToken::SequenceMarker {
            marker_type: crate::cst::SequenceMarkerType::Plain(content.to_string()),
            span: create_test_span(),
        }
    }

    #[test]
    fn test_detect_simple_paragraph() {
        let tokens = vec![create_text_token("This is a simple paragraph.")];

        match detect_paragraph(&tokens) {
            ParagraphParseResult::ValidParagraph(paragraph) => {
                assert_eq!(paragraph.content, "This is a simple paragraph.");
                assert_eq!(paragraph.lines.len(), 1);
            }
            _ => panic!("Expected valid paragraph"),
        }
    }

    #[test]
    fn test_detect_non_paragraph_sequence_marker() {
        let tokens = vec![
            create_sequence_token("1."),
            create_text_token("Session title"),
        ];

        match detect_paragraph(&tokens) {
            ParagraphParseResult::NotParagraph => {
                // Correctly identified as not a paragraph
            }
            _ => panic!("Expected not paragraph due to sequence marker"),
        }
    }

    #[test]
    fn test_extract_paragraph_content() {
        let tokens = vec![
            create_text_token("This"),
            create_text_token("is"),
            create_text_token("content"),
        ];

        let content = extract_paragraph_content(&tokens);
        assert_eq!(content, "This is content");
    }

    #[test]
    fn test_normalize_paragraph_whitespace() {
        assert_eq!(
            normalize_paragraph_whitespace("  hello   world  "),
            "hello world"
        );
        assert_eq!(normalize_paragraph_whitespace("single"), "single");
        assert_eq!(normalize_paragraph_whitespace(""), "");
        assert_eq!(normalize_paragraph_whitespace("   "), "");
    }

    #[test]
    fn test_should_terminate_paragraph() {
        // Empty tokens should terminate
        assert!(should_terminate_paragraph(&[]));

        // Content tokens should not terminate
        let content_tokens = vec![create_text_token("content")];
        assert!(!should_terminate_paragraph(&content_tokens));

        // Sequence marker should terminate (indicates other element)
        let sequence_tokens = vec![create_sequence_token("1.")];
        assert!(should_terminate_paragraph(&sequence_tokens));

        // Indentation should terminate
        let indent_tokens = vec![ScannerToken::Indent {
            span: create_test_span(),
        }];
        assert!(should_terminate_paragraph(&indent_tokens));
    }

    #[test]
    fn test_collect_paragraph_lines() {
        let line1 = vec![create_text_token("First line")];
        let line2 = vec![create_text_token("Second line")];
        let lines = vec![line1, line2];

        let paragraph = collect_paragraph_lines(&lines).unwrap();
        assert_eq!(paragraph.lines.len(), 2);
        assert_eq!(paragraph.content, "First line Second line");
    }
}
