//! Tests for annotation element construction
//!
//! Tests that annotation tokens are correctly converted to annotation AST nodes.

use txxt::cst::high_level_tokens::HighLevelTokenBuilder;
use txxt::cst::{Position, SourceSpan};
use txxt::semantic::elements::annotation::create_annotation_element;

/// Test that annotation elements are created correctly from annotation tokens
#[test]
fn test_create_annotation_element() {
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

    // Create an annotation semantic token
    let annotation_token = HighLevelTokenBuilder::annotation(
        HighLevelTokenBuilder::label("note".to_string(), label_span),
        None, // No parameters
        Some(HighLevelTokenBuilder::text_span(
            "This is a note".to_string(),
            content_span,
        )),
        span,
    );

    // Test the element constructor directly
    let result = create_annotation_element(&annotation_token, &[]);

    assert!(result.is_ok());
    let annotation_block = result.unwrap();
    assert_eq!(annotation_block.label, "note");
    // TODO: Check annotation content when properly implemented
}

/// Test that annotation element creation fails for non-annotation tokens
#[test]
fn test_create_annotation_element_rejects_wrong_token() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    // Create a plain text token (not an annotation)
    let plain_token = HighLevelTokenBuilder::plain_text_line(
        String::new(),
        HighLevelTokenBuilder::text_span("Hello".to_string(), span.clone()),
        span,
    );

    // Should fail because it's not an annotation token
    let result = create_annotation_element(&plain_token, &[]);
    assert!(result.is_err());
}
