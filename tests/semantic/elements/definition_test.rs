//! Tests for definition element construction
//!
//! Tests that definition tokens are correctly converted to definition AST nodes.

use txxt::cst::high_level_tokens::HighLevelTokenBuilder;
use txxt::cst::{Position, SourceSpan};
use txxt::semantic::elements::definition::create_definition_element;

/// Test that definition elements are created correctly from definition tokens
#[test]
fn test_create_definition_element() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 15 },
    };

    let term_span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 12 },
    };

    // Create a definition semantic token
    let definition_token = HighLevelTokenBuilder::definition(
        HighLevelTokenBuilder::text_span("Term".to_string(), term_span),
        None, // No parameters
        span,
    );

    // Test the element constructor directly
    let result = create_definition_element(&definition_token);

    assert!(result.is_ok());
    let _definition_block = result.unwrap();
    // TODO: Check definition term when properly implemented
    // TODO: Check definition parameters when properly implemented
}

/// Test that definition element creation fails for non-definition tokens
#[test]
fn test_create_definition_element_rejects_wrong_token() {
    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    // Create a plain text token (not a definition)
    let plain_token = HighLevelTokenBuilder::plain_text_line(
        String::new(),
        HighLevelTokenBuilder::text_span("Hello".to_string(), span.clone()),
        span,
    );

    // Should fail because it's not a definition token
    let result = create_definition_element(&plain_token);
    assert!(result.is_err());
}
