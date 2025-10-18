//! Session element tokenization
//!
//! Implements tokenization for session elements as defined in
//! docs/specs/elements/session/session.txxt
//!
//! Sessions are hierarchical content sections. At the tokenizer level,
//! sessions produce the same tokens as paragraphs (SequenceMarker + Text).
//! The session vs paragraph distinction is made later during parsing
//! based on whether the line is followed by indented content.

use crate::ast::scanner_tokens::{SourceSpan, ScannerToken};

/// Represents a session title with optional numbering
#[derive(Debug, Clone, PartialEq)]
pub struct SessionTitle {
    /// Optional numeric sequence (e.g., "1.2.3")
    pub numbering: Option<Vec<u32>>,
    /// The title text content
    pub title: String,
    /// Source span covering the entire session title
    pub span: SourceSpan,
}

/// Result of session title parsing
#[derive(Debug, Clone, PartialEq)]
pub enum SessionParseResult {
    /// Valid session title was found
    ValidSession(SessionTitle),
    /// Line is a paragraph, not a session
    Paragraph,
    /// Invalid session format with error
    Invalid(String),
}

/// Detects if a line represents a session title
///
/// Session recognition rules:
/// - Line contains text content
/// - May start with numeric sequence (1., 2.3., etc.)
/// - Must be followed by indented content to be confirmed as session
/// - Without indented content, treated as paragraph
pub fn detect_session_title(tokens: &[ScannerToken]) -> SessionParseResult {
    if tokens.is_empty() {
        return SessionParseResult::Paragraph;
    }

    let mut numbering = None;
    let mut title_start_idx = 0;

    // Check for optional numeric sequence at start
    if let Some(ScannerToken::SequenceMarker { marker_type, .. }) = tokens.first() {
        if let Some(parsed_numbering) = parse_session_numbering(marker_type.content()) {
            numbering = Some(parsed_numbering);
            title_start_idx = 1; // Skip the sequence marker when extracting title
        }
    }

    // Extract title text from tokens after the sequence marker (if any)
    let title_tokens: Vec<ScannerToken> = tokens[title_start_idx..].to_vec();
    let title_text = extract_title_text(&title_tokens);

    if title_text.trim().is_empty() {
        return SessionParseResult::Paragraph;
    }

    // Calculate span
    let span = if tokens.is_empty() {
        return SessionParseResult::Paragraph;
    } else {
        SourceSpan {
            start: tokens[0].span().start,
            end: tokens.last().unwrap().span().end,
        }
    };

    SessionParseResult::ValidSession(SessionTitle {
        numbering,
        title: title_text,
        span,
    })
}

/// Parses numeric session numbering from a sequence marker
///
/// Converts strings like "1.", "2.3.", "1.2.3." into numeric sequences
fn parse_session_numbering(marker: &str) -> Option<Vec<u32>> {
    // Remove trailing period if present
    let marker = marker.strip_suffix('.').unwrap_or(marker);

    if marker.is_empty() {
        return None;
    }

    // Split by periods and parse each part as number
    let parts: Result<Vec<u32>, _> = marker.split('.').map(|part| part.parse::<u32>()).collect();

    match parts {
        Ok(numbers) if !numbers.is_empty() => Some(numbers),
        _ => None,
    }
}

/// Extracts title text from a sequence of tokens
fn extract_title_text(tokens: &[ScannerToken]) -> String {
    tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::Text { content, .. } => Some(content.as_str()),
            ScannerToken::Identifier { content, .. } => Some(content.as_str()),
            // Skip structural tokens but preserve content tokens
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Validates that a potential session has indented content following it
///
/// This is used during parsing to distinguish sessions from paragraphs.
/// A line with session-like structure is only confirmed as a session
/// if it's followed by indented content.
pub fn confirm_session_with_content(_session_tokens: &[ScannerToken], following_tokens: &[ScannerToken]) -> bool {
    // Check if following tokens contain indentation
    following_tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::Indent { .. }))
}

/// Formats session numbering back to string representation
pub fn format_session_numbering(numbering: &[u32]) -> String {
    if numbering.is_empty() {
        String::new()
    } else {
        format!(
            "{}.",
            numbering
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(".")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::scanner_tokens::Position;

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
            marker_type: crate::ast::scanner_tokens::SequenceMarkerType::Numerical(1, content.to_string()),
            span: create_test_span(),
        }
    }

    #[test]
    fn test_parse_session_numbering() {
        assert_eq!(parse_session_numbering("1."), Some(vec![1]));
        assert_eq!(parse_session_numbering("2.3."), Some(vec![2, 3]));
        assert_eq!(parse_session_numbering("1.2.3."), Some(vec![1, 2, 3]));
        assert_eq!(parse_session_numbering("10."), Some(vec![10]));

        // Invalid cases
        assert_eq!(parse_session_numbering("a."), None);
        assert_eq!(parse_session_numbering(""), None);
        assert_eq!(parse_session_numbering("1.a."), None);
    }

    #[test]
    fn test_detect_numbered_session() {
        let tokens = vec![
            create_sequence_token("1."),
            create_text_token("Introduction"),
        ];

        let result = detect_session_title(&tokens);
        println!("Result: {:?}", result);

        match result {
            SessionParseResult::ValidSession(session) => {
                assert_eq!(session.numbering, Some(vec![1]));
                assert_eq!(session.title, "Introduction");
            }
            _ => panic!("Expected valid session"),
        }
    }

    #[test]
    fn test_detect_unnumbered_session() {
        let tokens = vec![create_text_token("Introduction")];

        match detect_session_title(&tokens) {
            SessionParseResult::ValidSession(session) => {
                assert_eq!(session.numbering, None);
                assert_eq!(session.title, "Introduction");
            }
            _ => panic!("Expected valid session"),
        }
    }

    #[test]
    fn test_detect_hierarchical_numbering() {
        let tokens = vec![
            create_sequence_token("1.2.3."),
            create_text_token("Subsection"),
        ];

        match detect_session_title(&tokens) {
            SessionParseResult::ValidSession(session) => {
                assert_eq!(session.numbering, Some(vec![1, 2, 3]));
                assert_eq!(session.title, "Subsection");
            }
            _ => panic!("Expected valid session"),
        }
    }

    #[test]
    fn test_format_session_numbering() {
        assert_eq!(format_session_numbering(&[1]), "1.");
        assert_eq!(format_session_numbering(&[1, 2]), "1.2.");
        assert_eq!(format_session_numbering(&[1, 2, 3]), "1.2.3.");
        assert_eq!(format_session_numbering(&[]), "");
    }

    #[test]
    fn test_confirm_session_with_content() {
        let session_tokens = vec![create_text_token("Title")];
        let following_with_indent = vec![
            ScannerToken::Indent {
                span: create_test_span(),
            },
            create_text_token("Content"),
        ];
        let following_without_indent = vec![create_text_token("Content")];

        assert!(confirm_session_with_content(
            &session_tokens,
            &following_with_indent
        ));
        assert!(!confirm_session_with_content(
            &session_tokens,
            &following_without_indent
        ));
    }
}
