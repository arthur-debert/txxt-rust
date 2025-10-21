//! Tests for AST Construction
//!
//! Integration tests for the AstConstructor that verify the full parsing flow
//! from high-level tokens to AST nodes.

use txxt::cst::high_level_tokens::{
    HighLevelNumberingForm, HighLevelNumberingStyle, HighLevelTokenBuilder, HighLevelTokenList,
};
use txxt::cst::{Position, SourceSpan};
use txxt::semantic::ast_construction::{AstConstructor, AstNode};

/// Test that the parser machinery initializes correctly
#[test]
fn test_parser_initialization() {
    let parser = AstConstructor::new();
    assert_eq!(parser.position, 0);
    assert_eq!(parser.indentation_level, 0);
}

/// Test that the parser handles empty semantic token list
#[test]
fn test_parse_empty_tokens() {
    let mut parser = AstConstructor::new();
    let empty_tokens = HighLevelTokenList::with_tokens(vec![]);

    let result = parser.parse(&empty_tokens);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

/// Test that the parser skips structural tokens correctly
#[test]
fn test_parse_structural_tokens_only() {
    let mut parser = AstConstructor::new();

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 4 },
    };

    let tokens = vec![
        HighLevelTokenBuilder::indent(span.clone()),
        HighLevelTokenBuilder::blank_line(span.clone()),
        HighLevelTokenBuilder::dedent(span.clone()),
    ];

    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

/// Test that the parser handles indentation level tracking
#[test]
fn test_indentation_level_tracking() {
    let mut parser = AstConstructor::new();

    let span = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 4 },
    };

    let tokens = vec![
        HighLevelTokenBuilder::indent(span.clone()),
        HighLevelTokenBuilder::indent(span.clone()),
        HighLevelTokenBuilder::dedent(span.clone()),
        HighLevelTokenBuilder::dedent(span.clone()),
    ];

    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
    // Parser should end with indentation level back to 0
    assert_eq!(parser.indentation_level, 0);
}

/// Test that the parser can parse annotation semantic tokens
#[test]
fn test_parse_annotation() {
    let mut parser = AstConstructor::new();

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

    let tokens = vec![annotation_token];
    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    let ast_nodes = result.unwrap();
    assert_eq!(ast_nodes.len(), 1);

    match &ast_nodes[0] {
        AstNode::Annotation(annotation_block) => {
            assert_eq!(annotation_block.label, "note");
            // TODO: Check annotation content when properly implemented
        }
        _ => panic!("Expected Annotation node, got {:?}", ast_nodes[0]),
    }
}

/// Test that the parser can parse definition semantic tokens
#[test]
fn test_parse_definition() {
    let mut parser = AstConstructor::new();

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

    let tokens = vec![definition_token];
    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    let ast_nodes = result.unwrap();
    assert_eq!(ast_nodes.len(), 1);

    match &ast_nodes[0] {
        AstNode::Definition(_definition_block) => {
            // TODO: Check definition term when properly implemented
            // TODO: Check definition parameters when properly implemented
        }
        _ => panic!("Expected Definition node, got {:?}", ast_nodes[0]),
    }
}

/// Test that the parser can parse paragraph semantic tokens
#[test]
fn test_parse_paragraph() {
    let mut parser = AstConstructor::new();

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
        HighLevelTokenBuilder::text_span("Hello world".to_string(), content_span),
        span,
    );

    let tokens = vec![paragraph_token];
    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    let ast_nodes = result.unwrap();
    assert_eq!(ast_nodes.len(), 1);

    match &ast_nodes[0] {
        AstNode::Paragraph(_paragraph_block) => {
            // TODO: Check paragraph content when properly implemented
        }
        _ => panic!("Expected Paragraph node, got {:?}", ast_nodes[0]),
    }
}

/// Test that the parser can parse session patterns
#[test]
fn test_parse_session() {
    let mut parser = AstConstructor::new();

    let span1 = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 0 },
    };

    let span2 = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 2, column: 10 },
    };

    let span3 = SourceSpan {
        start: Position { row: 3, column: 0 },
        end: Position { row: 3, column: 0 },
    };

    let span4 = SourceSpan {
        start: Position { row: 4, column: 0 },
        end: Position { row: 4, column: 4 },
    };

    let span5 = SourceSpan {
        start: Position { row: 5, column: 4 },
        end: Position { row: 5, column: 20 },
    };

    // Create a session pattern: blank line + title + blank line + indent + content
    let tokens = vec![
        HighLevelTokenBuilder::blank_line(span1),
        HighLevelTokenBuilder::plain_text_line(
            HighLevelTokenBuilder::text_span("Session Title".to_string(), span2.clone()),
            span2,
        ),
        HighLevelTokenBuilder::blank_line(span3),
        HighLevelTokenBuilder::indent(span4),
        HighLevelTokenBuilder::plain_text_line(
            HighLevelTokenBuilder::text_span("Session content".to_string(), span5.clone()),
            span5,
        ),
    ];

    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    let ast_nodes = result.unwrap();
    assert_eq!(ast_nodes.len(), 1);

    match &ast_nodes[0] {
        AstNode::Session(_session_block) => {
            // TODO: Check session title when properly implemented
            // TODO: Check session child count when properly implemented
        }
        _ => panic!("Expected Session node, got {:?}", ast_nodes[0]),
    }
}

/// Test that the parser can parse list patterns
#[test]
fn test_parse_list() {
    let mut parser = AstConstructor::new();

    let span1 = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 10 },
    };

    let span2 = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 2, column: 12 },
    };

    // Create a list pattern: two sequence text lines
    let tokens = vec![
        HighLevelTokenBuilder::sequence_text_line(
            HighLevelTokenBuilder::sequence_marker(
                HighLevelNumberingStyle::Plain,
                HighLevelNumberingForm::Regular,
                "-".to_string(),
                span1.clone(),
            ),
            HighLevelTokenBuilder::text_span("First item".to_string(), span1.clone()),
            span1,
        ),
        HighLevelTokenBuilder::sequence_text_line(
            HighLevelTokenBuilder::sequence_marker(
                HighLevelNumberingStyle::Plain,
                HighLevelNumberingForm::Regular,
                "-".to_string(),
                span2.clone(),
            ),
            HighLevelTokenBuilder::text_span("Second item".to_string(), span2.clone()),
            span2,
        ),
    ];

    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    let ast_nodes = result.unwrap();
    assert_eq!(ast_nodes.len(), 1);

    match &ast_nodes[0] {
        AstNode::List(_list_block) => {
            // TODO: Check list item count when properly implemented
        }
        _ => panic!("Expected List node, got {:?}", ast_nodes[0]),
    }
}

/// Test that the parser can parse nested session patterns
#[test]
fn test_parse_nested_sessions() {
    let mut parser = AstConstructor::new();

    let span1 = SourceSpan {
        start: Position { row: 1, column: 0 },
        end: Position { row: 1, column: 0 },
    };

    let span2 = SourceSpan {
        start: Position { row: 2, column: 0 },
        end: Position { row: 2, column: 15 },
    };

    let span3 = SourceSpan {
        start: Position { row: 3, column: 0 },
        end: Position { row: 3, column: 0 },
    };

    let span4 = SourceSpan {
        start: Position { row: 4, column: 0 },
        end: Position { row: 4, column: 4 },
    };

    let span5 = SourceSpan {
        start: Position { row: 5, column: 4 },
        end: Position { row: 5, column: 20 },
    };

    let span6 = SourceSpan {
        start: Position { row: 6, column: 4 },
        end: Position { row: 6, column: 0 },
    };

    let span7 = SourceSpan {
        start: Position { row: 7, column: 4 },
        end: Position { row: 7, column: 4 },
    };

    let _span8 = SourceSpan {
        start: Position { row: 8, column: 4 },
        end: Position { row: 8, column: 8 },
    };

    let span9 = SourceSpan {
        start: Position { row: 9, column: 8 },
        end: Position { row: 9, column: 25 },
    };

    let span10 = SourceSpan {
        start: Position { row: 10, column: 4 },
        end: Position { row: 10, column: 4 },
    };

    // Create a nested session pattern:
    // Blank line + "Outer Session" + blank line + indent +
    //   blank line + "Inner Session" + blank line + indent + "Content" + dedent + dedent
    let tokens = vec![
        HighLevelTokenBuilder::blank_line(span1),
        HighLevelTokenBuilder::plain_text_line(
            HighLevelTokenBuilder::text_span("Outer Session".to_string(), span2.clone()),
            span2,
        ),
        HighLevelTokenBuilder::blank_line(span3),
        HighLevelTokenBuilder::indent(span4),
        HighLevelTokenBuilder::blank_line(span6.clone()), // Add blank line before inner session
        HighLevelTokenBuilder::plain_text_line(
            HighLevelTokenBuilder::text_span("Inner Session".to_string(), span5.clone()),
            span5,
        ),
        HighLevelTokenBuilder::blank_line(span6),
        HighLevelTokenBuilder::indent(span7),
        HighLevelTokenBuilder::plain_text_line(
            HighLevelTokenBuilder::text_span("Nested content".to_string(), span9.clone()),
            span9,
        ),
        HighLevelTokenBuilder::dedent(span10.clone()),
        HighLevelTokenBuilder::dedent(span10),
    ];

    let semantic_tokens = HighLevelTokenList::with_tokens(tokens);
    let result = parser.parse(&semantic_tokens);

    assert!(result.is_ok());
    let ast_nodes = result.unwrap();
    assert_eq!(ast_nodes.len(), 1);

    match &ast_nodes[0] {
        AstNode::Session(_session_block) => {
            // TODO: Check session title when properly implemented
            // TODO: Check session child count when properly implemented
        }
        _ => panic!("Expected Session node, got {:?}", ast_nodes[0]),
    }
}
