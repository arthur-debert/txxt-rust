//! Tests for session element construction
//!
//! Tests that session components are correctly converted to session AST nodes.

use txxt::cst::high_level_tokens::HighLevelTokenBuilder;
use txxt::cst::{Position, SourceSpan};
use txxt::semantic::ast_construction::AstNode;
use txxt::semantic::elements::session::create_session_element;

/// Test that session elements are created correctly from title token and child nodes
#[test]
fn test_create_session_element() {
    let span = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 2, column: 10 },
    };

    // Create a session title token
    let title_token = HighLevelTokenBuilder::plain_text_line(
        HighLevelTokenBuilder::text_span("Session Title".to_string(), span.clone()),
        span,
    );

    // Empty child nodes for now
    let child_nodes: Vec<AstNode> = vec![];

    // Test the element constructor directly
    let result = create_session_element(&title_token, &child_nodes);

    assert!(result.is_ok());
    let _session_block = result.unwrap();
    // TODO: Check session title when properly implemented
    // TODO: Check session child count when properly implemented
}

/// Test that session elements work with sequence text line titles
#[test]
fn test_create_session_element_with_sequence_title() {
    let span = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 2, column: 15 },
    };

    let marker_span = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 2, column: 2 },
    };

    // Create a session title token with sequence marker
    let title_token = HighLevelTokenBuilder::sequence_text_line(
        HighLevelTokenBuilder::sequence_marker(
            txxt::cst::high_level_tokens::HighLevelNumberingStyle::Plain,
            txxt::cst::high_level_tokens::HighLevelNumberingForm::Regular,
            "1.".to_string(),
            marker_span,
        ),
        HighLevelTokenBuilder::text_span("Numbered Title".to_string(), span.clone()),
        span,
    );

    let child_nodes: Vec<AstNode> = vec![];

    let result = create_session_element(&title_token, &child_nodes);

    assert!(result.is_ok());
}

/// Test that session element creation fails for invalid title tokens
#[test]
fn test_create_session_element_rejects_wrong_token() {
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

    // Create an annotation token (not valid for session title)
    let annotation_token = HighLevelTokenBuilder::annotation(
        HighLevelTokenBuilder::label("note".to_string(), label_span),
        None,
        Some(HighLevelTokenBuilder::text_span(
            "This is a note".to_string(),
            content_span,
        )),
        span,
    );

    let child_nodes: Vec<AstNode> = vec![];

    // Should fail because annotation is not a valid title token
    let result = create_session_element(&annotation_token, &child_nodes);
    assert!(result.is_err());
}
