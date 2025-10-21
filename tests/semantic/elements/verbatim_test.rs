//! Tests for verbatim element construction
//!
//! Tests that verbatim block tokens are correctly converted to verbatim AST nodes.

use txxt::cst::high_level_tokens::HighLevelTokenBuilder;
use txxt::cst::{Position, SourceSpan};
use txxt::semantic::elements::verbatim::create_verbatim_element;

/// Test that verbatim elements are created correctly from verbatim block tokens
#[test]
fn test_create_verbatim_element() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 5, column: 3 },
    };

    let title_span = SourceSpan {
        start: Position { row: 1, column: 4 },
        end: Position { row: 1, column: 16 },
    };

    let label_span = SourceSpan {
        start: Position { row: 1, column: 17 },
        end: Position { row: 1, column: 27 },
    };

    let wall_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 3 },
    };

    let content_span = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 4, column: 10 },
    };

    // Create a verbatim block semantic token
    let verbatim_token = HighLevelTokenBuilder::verbatim_block(
        HighLevelTokenBuilder::text_span("Code Example".to_string(), title_span),
        HighLevelTokenBuilder::text_span("```".to_string(), wall_span.clone()),
        HighLevelTokenBuilder::text_span("print('hello')".to_string(), content_span),
        HighLevelTokenBuilder::label("python".to_string(), label_span),
        None,
        txxt::cst::WallType::InFlow(0),
        span,
    );

    // Test the element constructor directly
    let result = create_verbatim_element(&verbatim_token);

    assert!(result.is_ok());
    let verbatim_block = result.unwrap();
    assert_eq!(verbatim_block.label, "python");
    // TODO: Check verbatim title when properly implemented
    // TODO: Check verbatim content when properly implemented
}

/// Test that verbatim element creation fails for non-verbatim tokens
#[test]
fn test_create_verbatim_element_rejects_wrong_token() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    // Create a plain text token (not a verbatim block)
    let plain_token = HighLevelTokenBuilder::plain_text_line(
        String::new(),
        HighLevelTokenBuilder::text_span("Hello".to_string(), span.clone()),
        span,
    );

    // Should fail because it's not a verbatim token
    let result = create_verbatim_element(&plain_token);
    assert!(result.is_err());
}
