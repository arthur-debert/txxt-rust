#![allow(deprecated)]
//! Tests for paragraph element construction
//!
//! Tests that paragraph tokens are correctly converted to paragraph AST nodes.

use txxt::cst::high_level_tokens::HighLevelTokenBuilder;
use txxt::cst::{Position, SourceSpan};
use txxt::semantic::elements::paragraph::create_paragraph_element;

/// Test that paragraph elements are created correctly from plain text line tokens
#[test]
fn test_create_paragraph_element() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 12 },
    };

    let content_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 12 },
    };

    // Create a paragraph semantic token
    let paragraph_token = HighLevelTokenBuilder::plain_text_line(
        String::new(),
        HighLevelTokenBuilder::text_span("Hello world".to_string(), content_span),
        span,
    );

    // Test the element constructor directly
    let result = create_paragraph_element(&paragraph_token);

    assert!(result.is_ok());
    let _paragraph_block = result.unwrap();
    // TODO: Check paragraph content when properly implemented
}

/// Test that paragraph element creation fails for non-paragraph tokens
#[test]
fn test_create_paragraph_element_rejects_wrong_token() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 20 },
    };

    let label_span = SourceSpan {
        start: Position { row: 1, column: 3 },
        end: Position { row: 1, column: 7 },
    };

    let content_span = SourceSpan {
        start: Position { row: 1, column: 11 },
        end: Position { row: 1, column: 20 },
    };

    // Create an annotation token (not a paragraph)
    let annotation_token = HighLevelTokenBuilder::annotation(
        HighLevelTokenBuilder::label("note".to_string(), label_span),
        None,
        Some(HighLevelTokenBuilder::text_span(
            "This is a note".to_string(),
            content_span,
        )),
        span,
    );

    // Should fail because it's not a plain text line token
    let result = create_paragraph_element(&annotation_token);
    assert!(result.is_err());
}
