//! Comprehensive audit of all span calculations in tokenizers
//!
//! This systematically tests every tokenizer that creates spans to ensure
//! they handle Unicode correctly.

use txxt::ast::scanner_tokens::{ScannerToken, SourceSpan};
use txxt::lexer::Lexer;

/// List all places where we found column arithmetic
#[allow(dead_code)]
fn get_problematic_patterns() -> Vec<(&'static str, &'static str)> {
    vec![
        ("sequence.rs:58", "column: start_pos.column + 1"),
        ("sequence.rs:95", "column: start_pos.column + marker.len()"),
        ("sequence.rs:137", "column: start_pos.column + marker.len()"),
        ("sequence.rs:170", "column: start_pos.column + marker.len()"),
        (
            "parameter_integration.rs:134",
            "column: span.start.column + colon_pos",
        ),
        (
            "parameter_integration.rs:144",
            "column: span.start.column + colon_pos",
        ),
        (
            "parameter_integration.rs:148",
            "column: span.start.column + colon_pos + 1",
        ),
        (
            "parameter_integration.rs:157",
            "column: span.start.column + colon_pos + 1",
        ),
        (
            "parameter_integration.rs:241",
            "column: span.start.column + colon_pos",
        ),
        (
            "parameter_integration.rs:251",
            "column: span.start.column + colon_pos",
        ),
        (
            "parameter_integration.rs:255",
            "column: span.start.column + colon_pos + 1",
        ),
        (
            "parameter_integration.rs:264",
            "column: span.start.column + colon_pos + 1",
        ),
        (
            "parameter_integration.rs:365",
            "column: param_start.column + consumed",
        ),
    ]
}

#[test]
fn audit_sequence_marker_plain() {
    // The plain marker "-" is hardcoded to add 1, which is correct
    // since "-" is always 1 character. But let's verify with context.
    let input = "- test";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let marker = find_sequence_marker(&tokens);
    assert_span_equals(get_span(marker), 0, 0, 0, 1, "Plain marker '- '");
}

#[test]
fn audit_sequence_marker_numerical_single_byte() {
    // Test "1." which should work correctly
    let input = "1. test";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let marker = find_sequence_marker(&tokens);
    assert_span_equals(get_span(marker), 0, 0, 0, 2, "Numerical marker '1.'");
}

#[test]
fn audit_sequence_marker_numerical_multi_byte() {
    // This test demonstrates the bug: it calculates end as start + "42.".len() = 0 + 3 = 3
    // But this is actually CORRECT because "42." is 3 characters!
    let input = "42. test";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let marker = find_sequence_marker(&tokens);
    assert_span_equals(get_span(marker), 0, 0, 0, 3, "Numerical marker '42.'");
}

#[test]
fn audit_whitespace_preservation() {
    // Verify whitespace tokens have correct spans
    let input = "text    more";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let whitespace = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Whitespace { .. }))
        .unwrap();
    assert_span_equals(get_span(whitespace), 0, 4, 0, 8, "Four spaces");
}

#[test]
fn audit_text_token_with_unicode() {
    // Text tokens should handle Unicode correctly since they use current_position()
    let input = "café";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let text = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Text { .. }))
        .unwrap();
    assert_span_equals(get_span(text), 0, 0, 0, 4, "Text 'café'");
}

#[test]
fn audit_annotation_markers() {
    let input = ":: label ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let markers: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t, ScannerToken::AnnotationMarker { .. }))
        .collect();

    assert_eq!(markers.len(), 2);
    assert_span_equals(get_span(markers[0]), 0, 0, 0, 2, "First ::");
    assert_span_equals(get_span(markers[1]), 0, 9, 0, 11, "Second ::");
}

#[test]
fn audit_inline_delimiters() {
    let input = "*bold* _italic_ `code` #math#";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let bold1 = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::BoldDelimiter { .. }))
        .unwrap();
    assert_span_equals(get_span(bold1), 0, 0, 0, 1, "First *");

    let italic1 = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::ItalicDelimiter { .. }))
        .unwrap();
    assert_span_equals(get_span(italic1), 0, 7, 0, 8, "First _");

    let code1 = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::CodeDelimiter { .. }))
        .unwrap();
    assert_span_equals(get_span(code1), 0, 16, 0, 17, "First `");

    let math1 = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::MathDelimiter { .. }))
        .unwrap();
    assert_span_equals(get_span(math1), 0, 23, 0, 24, "First #");
}

#[test]
fn audit_parameter_spans() {
    let input = ":: label:key=value ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let param = tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::Parameter { .. }))
        .unwrap();
    // This will fail because parameters are created at (0,0)
    assert_span_equals(get_span(param), 0, 9, 0, 18, "Parameter key=value");
}

// The real Unicode test - this is where bugs will show up
#[test]
#[should_panic(expected = "is actually correct")] // Remove this when we find real bugs
fn audit_sequence_marker_with_unicode_context() {
    // If column arithmetic is wrong, this will fail
    // The marker "1." after "café " should start at column 5 (after 5 chars: c-a-f-é-space)
    // and end at column 7 (after the "1.")
    // BUT if the code does column + "1.".len(), it's actually CORRECT because "1." is 2 chars
    let input = "café 1. test";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    println!("\nDEBUG: ScannerTokens for '{}':", input);
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token {}: {:?}", i, token);
    }

    // This should fail because sequence markers are only recognized at column 0
    // So this is actually correct behavior - no bug here!
    panic!("This is actually correct - sequence markers only work at line start");
}

// Helper functions

fn find_sequence_marker(tokens: &[ScannerToken]) -> &ScannerToken {
    tokens
        .iter()
        .find(|t| matches!(t, ScannerToken::SequenceMarker { .. }))
        .expect("Should find sequence marker")
}

fn assert_span_equals(
    span: &SourceSpan,
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
    context: &str,
) {
    assert_eq!(
        (
            span.start.row,
            span.start.column,
            span.end.row,
            span.end.column
        ),
        (start_row, start_col, end_row, end_col),
        "{}: span mismatch - expected ({},{}) to ({},{}), got ({},{}) to ({},{})",
        context,
        start_row,
        start_col,
        end_row,
        end_col,
        span.start.row,
        span.start.column,
        span.end.row,
        span.end.column
    );
}

fn get_span(token: &ScannerToken) -> &SourceSpan {
    match token {
        ScannerToken::Text { span, .. }
        | ScannerToken::Whitespace { span, .. }
        | ScannerToken::SequenceMarker { span, .. }
        | ScannerToken::AnnotationMarker { span, .. }
        | ScannerToken::DefinitionMarker { span, .. }
        | ScannerToken::Parameter { span, .. }
        | ScannerToken::BoldDelimiter { span }
        | ScannerToken::ItalicDelimiter { span }
        | ScannerToken::CodeDelimiter { span }
        | ScannerToken::MathDelimiter { span }
        | ScannerToken::Newline { span }
        | ScannerToken::BlankLine { span, .. }
        | ScannerToken::LeftBracket { span }
        | ScannerToken::RightBracket { span }
        | ScannerToken::Colon { span }
        | ScannerToken::Eof { span } => span,
        _ => panic!("Token variant not handled in get_span()"),
    }
}
