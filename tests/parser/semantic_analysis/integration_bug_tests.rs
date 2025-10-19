//! Integration tests that capture the main semantic analysis bugs
//!
//! These tests demonstrate the current failures in the semantic analysis pipeline
//! and will be used to verify fixes for the three major defects:
//! 1. Incomplete main analysis loop
//! 2. Flawed heuristics in is_core_block_element  
//! 3. Incorrect fallback for syntactic tokens

use txxt::ast::scanner_tokens::{Position, ScannerToken, SourceSpan};
use txxt::ast::semantic_tokens::SemanticToken;
use txxt::parser::pipeline::semantic_analysis::SemanticAnalyzer;

/// Test that captures Defect 1: Main analysis loop doesn't compose complex tokens
///
/// This test demonstrates that the current analyze() function processes tokens
/// individually instead of recognizing patterns and calling the appropriate
/// transformation functions like transform_definition().
#[test]
fn test_defect_1_definition_not_composed() {
    let analyzer = SemanticAnalyzer::new();

    // Create a definition pattern: "Term ::"
    let scanner_tokens = vec![
        ScannerToken::Text {
            content: "Term".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 7 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();

    // BUG: Currently this produces individual tokens instead of a Definition
    // The current implementation processes tokens one by one, creating:
    // - TextSpan("Term")
    // - TextSpan(" ")
    // - TxxtMarker
    // Instead of recognizing the pattern and creating a Definition token

    // This assertion should FAIL with current implementation
    // After fix, it should PASS
    let has_definition = semantic_tokens
        .iter()
        .any(|token| matches!(token, SemanticToken::Definition { .. }));

    // This test should FAIL initially, demonstrating the bug
    assert!(
        has_definition,
        "Expected Definition token to be composed, but got individual tokens: {:?}",
        semantic_tokens.tokens
    );
}

/// Test that captures Defect 1: Annotation not composed
///
/// This test demonstrates that the current analyze() function doesn't recognize
/// annotation patterns and call transform_annotation().
#[test]
fn test_defect_1_annotation_not_composed() {
    let analyzer = SemanticAnalyzer::new();

    // Create an annotation pattern: ":: author :: Arthur Debert"
    let scanner_tokens = vec![
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 2 },
                end: Position { row: 1, column: 3 },
            },
        },
        ScannerToken::Identifier {
            content: "author".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 3 },
                end: Position { row: 1, column: 9 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 9 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 12 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 12 },
                end: Position { row: 1, column: 13 },
            },
        },
        ScannerToken::Text {
            content: "Arthur Debert".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 13 },
                end: Position { row: 1, column: 26 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();

    // BUG: Currently this produces individual tokens instead of an Annotation
    // The current implementation processes tokens one by one, creating:
    // - TxxtMarker
    // - TextSpan(" ")
    // - Label("author")
    // - TextSpan(" ")
    // - TxxtMarker
    // - TextSpan(" ")
    // - TextSpan("Arthur Debert")
    // Instead of recognizing the pattern and creating an Annotation token

    // This assertion should FAIL with current implementation
    // After fix, it should PASS
    let has_annotation = semantic_tokens
        .iter()
        .any(|token| matches!(token, SemanticToken::Annotation { .. }));

    // This test should FAIL initially, demonstrating the bug
    assert!(
        has_annotation,
        "Expected Annotation token to be composed, but got individual tokens: {:?}",
        semantic_tokens.tokens
    );
}

/// Test that captures Defect 2: Flawed heuristics in is_core_block_element
///
/// This test demonstrates that the current heuristic logic incorrectly
/// classifies document types based on global properties instead of local patterns.
#[test]
fn test_defect_2_flawed_heuristics() {
    let analyzer = SemanticAnalyzer::new();

    // Create a simple paragraph that should be processed as a line
    // but the flawed heuristics might misclassify it
    let scanner_tokens = vec![ScannerToken::Text {
        content: "This is a simple paragraph.".to_string(),
        span: SourceSpan {
            start: Position { row: 1, column: 0 },
            end: Position { row: 1, column: 28 },
        },
    }];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();

    // BUG: The current is_core_block_element() function uses flawed heuristics
    // that analyze the entire token stream to guess if it's a "test scenario"
    // or "real document". This is incorrect - a parser should make decisions
    // based on local token patterns, not global file properties.

    // The current implementation might incorrectly process this as individual tokens
    // instead of recognizing it as a line-level element

    // This assertion should PASS - we expect a PlainTextLine
    // But the flawed heuristics might cause it to be processed incorrectly
    let has_plain_text_line = semantic_tokens
        .iter()
        .any(|token| matches!(token, SemanticToken::PlainTextLine { .. }));

    // This test might FAIL initially due to flawed heuristics
    assert!(has_plain_text_line,
        "Expected PlainTextLine token, but flawed heuristics may have caused incorrect processing: {:?}",
        semantic_tokens.tokens
    );
}

/// Test that captures Defect 3: Syntactic tokens converted to text spans
///
/// This test demonstrates that the current fallback logic incorrectly converts
/// syntactic markers like Colon to plain text spans, losing their syntactic meaning.
#[test]
fn test_defect_3_syntactic_tokens_lost() {
    let analyzer = SemanticAnalyzer::new();

    // Create a definition with parameters: "Term: param ::"
    let scanner_tokens = vec![
        ScannerToken::Text {
            content: "Term".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 4 },
            },
        },
        ScannerToken::Colon {
            span: SourceSpan {
                start: Position { row: 1, column: 4 },
                end: Position { row: 1, column: 5 },
            },
        },
        ScannerToken::Text {
            content: "param".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 5 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 11 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 11 },
                end: Position { row: 1, column: 13 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();

    // BUG: The current fallback logic converts the Colon token to a TextSpan
    // using token_to_text_content(). This erases the syntactic meaning of the
    // colon, making it impossible for subsequent parsing phases to correctly
    // identify parameter structures.

    // This assertion should FAIL with current implementation
    // After fix, it should PASS (the colon should be preserved as a syntactic marker)
    let has_colon_preserved = semantic_tokens.iter().any(|token| match token {
        SemanticToken::TextSpan { content, .. } => content == ":",
        _ => false,
    });

    // This test should FAIL initially, demonstrating the bug
    // The colon should be preserved as a syntactic marker, not converted to text
    assert!(!has_colon_preserved,
        "CRITICAL BUG: Colon syntactic marker was converted to TextSpan, losing syntactic meaning. Tokens: {:?}",
        semantic_tokens.tokens
    );
}

/// Test that demonstrates the correct expected behavior after fixes
///
/// This test shows what the semantic analysis should produce after all
/// three defects are fixed.
#[test]
fn test_expected_behavior_after_fixes() {
    let analyzer = SemanticAnalyzer::new();

    // Create a complex document with multiple element types
    let scanner_tokens = vec![
        // Definition: "Algorithm ::"
        ScannerToken::Text {
            content: "Algorithm".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 0 },
                end: Position { row: 1, column: 9 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 1, column: 9 },
                end: Position { row: 1, column: 10 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 1, column: 10 },
                end: Position { row: 1, column: 12 },
            },
        },
        ScannerToken::Newline {
            span: SourceSpan {
                start: Position { row: 1, column: 12 },
                end: Position { row: 1, column: 12 },
            },
        },
        // Annotation: ":: author :: Arthur Debert"
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 2, column: 0 },
                end: Position { row: 2, column: 2 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 2 },
                end: Position { row: 2, column: 3 },
            },
        },
        ScannerToken::Identifier {
            content: "author".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 3 },
                end: Position { row: 2, column: 9 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 9 },
                end: Position { row: 2, column: 10 },
            },
        },
        ScannerToken::TxxtMarker {
            span: SourceSpan {
                start: Position { row: 2, column: 10 },
                end: Position { row: 2, column: 12 },
            },
        },
        ScannerToken::Whitespace {
            content: " ".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 12 },
                end: Position { row: 2, column: 13 },
            },
        },
        ScannerToken::Text {
            content: "Arthur Debert".to_string(),
            span: SourceSpan {
                start: Position { row: 2, column: 13 },
                end: Position { row: 2, column: 26 },
            },
        },
    ];

    let result = analyzer.analyze(scanner_tokens);
    assert!(result.is_ok());

    let semantic_tokens = result.unwrap();

    // After fixes, we should have:
    // 1. A Definition token for "Algorithm ::"
    // 2. An Annotation token for ":: author :: Arthur Debert"
    // 3. Proper structural token handling

    let tokens = &semantic_tokens.tokens;

    // Check for Definition token
    let has_definition = tokens
        .iter()
        .any(|token| matches!(token, SemanticToken::Definition { .. }));
    assert!(has_definition, "Expected Definition token after fixes");

    // Check for Annotation token
    let has_annotation = tokens
        .iter()
        .any(|token| matches!(token, SemanticToken::Annotation { .. }));
    assert!(has_annotation, "Expected Annotation token after fixes");

    // Verify that we don't have individual TxxtMarker tokens floating around
    // (they should be consumed by the complex token transformations)
    let individual_txxt_markers = tokens
        .iter()
        .filter(|token| matches!(token, SemanticToken::TxxtMarker { .. }))
        .count();

    // There should be no individual TxxtMarker tokens left - they should all
    // be consumed by Definition and Annotation transformations
    assert_eq!(individual_txxt_markers, 0,
        "Expected no individual TxxtMarker tokens - they should be consumed by complex transformations. Got: {:?}",
        tokens
    );
}
