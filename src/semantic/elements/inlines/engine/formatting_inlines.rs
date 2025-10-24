//! Formatting Inline Type Implementations
//!
//! This module implements the four formatting inline types for the generic engine:
//! - Bold (Strong): `*content*` - Can nest other types except bold
//! - Italic (Emphasis): `_content_` - Can nest other types except italic
//! - Code: `` `content` `` - No nesting, literal content
//! - Math: `#content#` - No nesting, literal content

use super::pipeline_data::{MatchedSpan, StageData, StageError};
use super::{Pipeline, PipelineBuilder};
use crate::ast::elements::formatting::inlines::{Inline, Text, TextTransform};
use crate::cst::{ScannerToken, ScannerTokenSequence};

/// Extract text content from tokens
fn extract_content(tokens: &[ScannerToken]) -> String {
    tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::Text { content, .. } => Some(content.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

// ============================================================================
// Bold (Strong) Processing
// ============================================================================

/// Process bold/strong formatting
///
/// Bold can nest other formatting types except bold itself.
/// The nesting prevention happens at the engine level - bold delimiters
/// inside bold content become plain text because the engine won't match
/// overlapping delimiters.
fn process_bold(data: StageData) -> Result<StageData, StageError> {
    let span = data.downcast::<MatchedSpan>()?;

    // TODO: Recursively process inner content for nested formatting
    // For now, treat as plain text (will be fixed when all formatting types are registered)
    let nested_transforms = tokens_to_identity_transforms(&span.inner_tokens);

    let inline = Inline::TextLine(TextTransform::Strong(nested_transforms));
    Ok(StageData::new(inline))
}

/// Build bold inline pipeline
pub fn build_bold_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .then("process_bold", process_bold)
        .build()
}

// ============================================================================
// Italic (Emphasis) Processing
// ============================================================================

/// Process italic/emphasis formatting
///
/// Italic can nest other formatting types except italic itself.
fn process_italic(data: StageData) -> Result<StageData, StageError> {
    let span = data.downcast::<MatchedSpan>()?;

    // TODO: Recursively process inner content for nested formatting
    let nested_transforms = tokens_to_identity_transforms(&span.inner_tokens);

    let inline = Inline::TextLine(TextTransform::Emphasis(nested_transforms));
    Ok(StageData::new(inline))
}

/// Build italic inline pipeline
pub fn build_italic_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .then("process_italic", process_italic)
        .build()
}

// ============================================================================
// Code Processing
// ============================================================================

/// Process code formatting
///
/// Code is literal - no nesting or further parsing.
fn process_code(data: StageData) -> Result<StageData, StageError> {
    let span = data.downcast::<MatchedSpan>()?;

    let text_content = extract_content(&span.inner_tokens);
    let token_sequence = ScannerTokenSequence {
        tokens: span.inner_tokens.clone(),
    };

    let inline = Inline::TextLine(TextTransform::Code(Text::simple_with_tokens(
        &text_content,
        token_sequence,
    )));

    Ok(StageData::new(inline))
}

/// Build code inline pipeline
pub fn build_code_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .then("process_code", process_code)
        .build()
}

// ============================================================================
// Math Processing
// ============================================================================

/// Process math formatting
///
/// Math is literal - no nesting or further parsing.
fn process_math(data: StageData) -> Result<StageData, StageError> {
    let span = data.downcast::<MatchedSpan>()?;

    let text_content = extract_content(&span.inner_tokens);
    let token_sequence = ScannerTokenSequence {
        tokens: span.inner_tokens.clone(),
    };

    let inline = Inline::TextLine(TextTransform::Math(Text::simple_with_tokens(
        &text_content,
        token_sequence,
    )));

    Ok(StageData::new(inline))
}

/// Build math inline pipeline
pub fn build_math_pipeline() -> Pipeline {
    PipelineBuilder::new()
        .then("process_math", process_math)
        .build()
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert tokens to plain text transforms (for nested content processing)
///
/// TODO: This is temporary - will be replaced with recursive engine call
/// once all formatting types are registered.
fn tokens_to_identity_transforms(tokens: &[ScannerToken]) -> Vec<TextTransform> {
    tokens
        .iter()
        .map(|token| {
            let token_sequence = ScannerTokenSequence {
                tokens: vec![token.clone()],
            };
            TextTransform::Identity(Text::simple_with_tokens(token.content(), token_sequence))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    fn create_text_token(content: &str) -> ScannerToken {
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

    #[test]
    fn test_process_bold() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("hello")],
            full_tokens: vec![
                create_text_token("*"),
                create_text_token("hello"),
                create_text_token("*"),
            ],
            start: 0,
            end: 3,
            inline_name: "bold".to_string(),
        };

        let result = process_bold(StageData::new(span));
        assert!(result.is_ok());

        let inline = result.unwrap().downcast::<Inline>().unwrap().clone();
        match inline {
            Inline::TextLine(TextTransform::Strong(_)) => {}
            _ => panic!("Expected Strong transform"),
        }
    }

    #[test]
    fn test_process_italic() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("hello")],
            full_tokens: vec![
                create_text_token("_"),
                create_text_token("hello"),
                create_text_token("_"),
            ],
            start: 0,
            end: 3,
            inline_name: "italic".to_string(),
        };

        let result = process_italic(StageData::new(span));
        assert!(result.is_ok());

        let inline = result.unwrap().downcast::<Inline>().unwrap().clone();
        match inline {
            Inline::TextLine(TextTransform::Emphasis(_)) => {}
            _ => panic!("Expected Emphasis transform"),
        }
    }

    #[test]
    fn test_process_code() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("code")],
            full_tokens: vec![
                create_text_token("`"),
                create_text_token("code"),
                create_text_token("`"),
            ],
            start: 0,
            end: 3,
            inline_name: "code".to_string(),
        };

        let result = process_code(StageData::new(span));
        assert!(result.is_ok());

        let inline = result.unwrap().downcast::<Inline>().unwrap().clone();
        match inline {
            Inline::TextLine(TextTransform::Code(text)) => {
                assert_eq!(text.content(), "code");
            }
            _ => panic!("Expected Code transform"),
        }
    }

    #[test]
    fn test_process_math() {
        let span = MatchedSpan {
            inner_tokens: vec![create_text_token("x^2")],
            full_tokens: vec![
                create_text_token("#"),
                create_text_token("x^2"),
                create_text_token("#"),
            ],
            start: 0,
            end: 3,
            inline_name: "math".to_string(),
        };

        let result = process_math(StageData::new(span));
        assert!(result.is_ok());

        let inline = result.unwrap().downcast::<Inline>().unwrap().clone();
        match inline {
            Inline::TextLine(TextTransform::Math(text)) => {
                assert_eq!(text.content(), "x^2");
            }
            _ => panic!("Expected Math transform"),
        }
    }
}
