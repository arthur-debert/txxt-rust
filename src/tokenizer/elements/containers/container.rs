//! Container element tokenization
//!
//! Implements tokenization for container elements as defined in
//! docs/specs/elements/containers/container.txxt
//!
//! Containers provide structural organization through indentation.
//! Container detection is primarily handled through indentation tokens
//! (Indent/Dedent) that are produced by the lexer infrastructure.

use crate::ast::tokens::Token;

/// Determines container type based on parsing context
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerType {
    /// Content Container - holds any block elements except sessions
    Content,
    /// Session Container - holds any block elements including sessions  
    Session,
    /// Ignore Container - holds verbatim content only
    Ignore,
}

/// Container creation context for type determination
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerContext {
    /// Root document context (creates Session container)
    Document,
    /// Session content context (creates Session container)
    SessionContent,
    /// List item context (creates Content container)
    ListItem,
    /// Definition content context (creates Content container)
    Definition,
    /// Annotation content context (creates Content container)
    Annotation,
    /// Verbatim block context (creates Ignore container)
    VerbatimBlock,
}

/// Detects container start based on indentation increase
///
/// Containers are created implicitly when indentation increases.
/// The lexer produces Indent tokens that signal container boundaries.
pub fn detect_container_start(token: &Token) -> bool {
    matches!(token, Token::Indent { .. })
}

/// Detects container end based on indentation decrease
///
/// Containers end when indentation returns to previous level.
/// The lexer produces Dedent tokens that signal container boundaries.
pub fn detect_container_end(token: &Token) -> bool {
    matches!(token, Token::Dedent { .. })
}

/// Determines container type from parsing context
///
/// # Arguments
/// * `context` - The parsing context that determines container type
///
/// # Returns
/// The appropriate container type for the given context
pub fn determine_container_type(context: ContainerContext) -> ContainerType {
    match context {
        ContainerContext::Document | ContainerContext::SessionContent => ContainerType::Session,
        ContainerContext::ListItem
        | ContainerContext::Definition
        | ContainerContext::Annotation => ContainerType::Content,
        ContainerContext::VerbatimBlock => ContainerType::Ignore,
    }
}

/// Validates container content based on container type
///
/// Different container types have different content restrictions:
/// - Content containers cannot contain sessions
/// - Session containers can contain any block type
/// - Ignore containers only contain verbatim content
pub fn validate_container_content(
    container_type: &ContainerType,
    content_tokens: &[Token],
) -> Result<(), String> {
    match container_type {
        ContainerType::Content => {
            // Check for session markers which are not allowed in content containers
            for token in content_tokens {
                if let Token::SequenceMarker { marker_type, .. } = token {
                    // Session markers are numeric sequences like "1.", "2.1.", etc.
                    if is_session_marker(marker_type.content()) {
                        return Err(format!(
                            "Sessions not allowed in Content containers. Found session marker: {}",
                            marker_type.content()
                        ));
                    }
                }
            }
        }
        ContainerType::Session => {
            // Session containers can contain any block type - no restrictions
        }
        ContainerType::Ignore => {
            // Ignore containers should only contain verbatim content and blank lines
            for token in content_tokens {
                match token {
                    Token::VerbatimContent { .. }
                    | Token::BlankLine { .. }
                    | Token::Newline { .. } => {
                        // These are allowed in ignore containers
                    }
                    _ => {
                        return Err(format!(
                            "Only verbatim content allowed in Ignore containers. Found: {:?}",
                            token
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Checks if a sequence marker indicates a session
///
/// Session markers are numeric hierarchical sequences like:
/// - "1.", "2.", "3." (top level)
/// - "1.1.", "1.2.", "2.1." (subsections)
/// - "1.1.1.", "2.3.4." (sub-subsections)
fn is_session_marker(marker: &str) -> bool {
    // Remove trailing period if present
    let marker = marker.strip_suffix('.').unwrap_or(marker);

    // Split by periods and check if all parts are numeric
    if marker.is_empty() {
        return false;
    }

    marker
        .split('.')
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::tokens::{Position, SourceSpan};

    fn create_test_span() -> SourceSpan {
        SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 1 },
        }
    }

    #[test]
    fn test_detect_container_start() {
        let indent_token = Token::Indent {
            span: create_test_span(),
        };
        assert!(detect_container_start(&indent_token));

        let text_token = Token::Text {
            content: "hello".to_string(),
            span: create_test_span(),
        };
        assert!(!detect_container_start(&text_token));
    }

    #[test]
    fn test_detect_container_end() {
        let dedent_token = Token::Dedent {
            span: create_test_span(),
        };
        assert!(detect_container_end(&dedent_token));

        let text_token = Token::Text {
            content: "hello".to_string(),
            span: create_test_span(),
        };
        assert!(!detect_container_end(&text_token));
    }

    #[test]
    fn test_determine_container_type() {
        assert_eq!(
            determine_container_type(ContainerContext::Document),
            ContainerType::Session
        );
        assert_eq!(
            determine_container_type(ContainerContext::SessionContent),
            ContainerType::Session
        );
        assert_eq!(
            determine_container_type(ContainerContext::ListItem),
            ContainerType::Content
        );
        assert_eq!(
            determine_container_type(ContainerContext::VerbatimBlock),
            ContainerType::Ignore
        );
    }

    #[test]
    fn test_is_session_marker() {
        // Valid session markers
        assert!(is_session_marker("1."));
        assert!(is_session_marker("2."));
        assert!(is_session_marker("1.1."));
        assert!(is_session_marker("1.2.3."));
        assert!(is_session_marker("10."));
        assert!(is_session_marker("1.10."));

        // Invalid session markers
        assert!(!is_session_marker("a."));
        assert!(!is_session_marker("-"));
        assert!(!is_session_marker("1)"));
        assert!(!is_session_marker(""));
        assert!(!is_session_marker("."));
        assert!(!is_session_marker("1.."));
        assert!(!is_session_marker("1.a."));
    }

    #[test]
    fn test_validate_container_content() {
        let span = create_test_span();

        // Content container with allowed content
        let text_token = Token::Text {
            content: "hello".to_string(),
            span: span.clone(),
        };
        let list_marker = Token::SequenceMarker {
            marker_type: crate::ast::tokens::SequenceMarkerType::Plain("-".to_string()),
            span: span.clone(),
        };
        let content_tokens = vec![text_token, list_marker];

        assert!(validate_container_content(&ContainerType::Content, &content_tokens).is_ok());

        // Content container with disallowed session marker
        let session_marker = Token::SequenceMarker {
            marker_type: crate::ast::tokens::SequenceMarkerType::Numerical(1, "1.".to_string()),
            span: span.clone(),
        };
        let invalid_content = vec![session_marker];

        assert!(validate_container_content(&ContainerType::Content, &invalid_content).is_err());

        // Session container allows everything
        let session_marker = Token::SequenceMarker {
            marker_type: crate::ast::tokens::SequenceMarkerType::Numerical(1, "1.".to_string()),
            span: span.clone(),
        };
        let session_content = vec![session_marker];

        assert!(validate_container_content(&ContainerType::Session, &session_content).is_ok());
    }
}
