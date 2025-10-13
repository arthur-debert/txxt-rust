//! Tests for the Detokenizer - Round-trip Verification
//!
//! These tests verify the detokenizer's ability to reconstruct source text
//! from tokens, enabling round-trip verification of the tokenization process.

use txxt::ast::tokens::Token;
use txxt::parser::detokenizer::Detokenizer;
use txxt::parser::pipeline::BlockGrouper;
use txxt::tokenizer::tokenize;

/// Helper function to perform round-trip test
fn round_trip_test(original: &str) -> Result<(), String> {
    // Step 1: Tokenize
    let tokens1 = tokenize(original);

    // Step 2: Detokenize
    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer
        .detokenize_tokens(&tokens1)
        .map_err(|e| format!("Detokenization failed: {:?}", e))?;

    // Step 3: Re-tokenize
    let tokens2 = tokenize(&reconstructed);

    // Step 4: Compare tokens (not strings)
    if tokens1.len() != tokens2.len() {
        return Err(format!(
            "Token count mismatch: {} vs {}",
            tokens1.len(),
            tokens2.len()
        ));
    }

    for (i, (t1, t2)) in tokens1.iter().zip(tokens2.iter()).enumerate() {
        if !tokens_equal(t1, t2) {
            return Err(format!(
                "Token mismatch at position {}: {:?} vs {:?}",
                i, t1, t2
            ));
        }
    }

    Ok(())
}

/// Compare tokens for equality (ignoring source spans)
fn tokens_equal(t1: &Token, t2: &Token) -> bool {
    use Token::*;
    match (t1, t2) {
        (Text { content: c1, .. }, Text { content: c2, .. }) => c1 == c2,
        (Newline { .. }, Newline { .. }) => true,
        (BlankLine { .. }, BlankLine { .. }) => true,
        (Indent { .. }, Indent { .. }) => true,
        (Dedent { .. }, Dedent { .. }) => true,
        (
            SequenceMarker {
                marker_type: m1, ..
            },
            SequenceMarker {
                marker_type: m2, ..
            },
        ) => m1 == m2,
        (AnnotationMarker { content: c1, .. }, AnnotationMarker { content: c2, .. }) => c1 == c2,
        (DefinitionMarker { content: c1, .. }, DefinitionMarker { content: c2, .. }) => c1 == c2,
        (Dash { .. }, Dash { .. }) => true,
        (Period { .. }, Period { .. }) => true,
        (LeftBracket { .. }, LeftBracket { .. }) => true,
        (RightBracket { .. }, RightBracket { .. }) => true,
        (AtSign { .. }, AtSign { .. }) => true,
        (LeftParen { .. }, LeftParen { .. }) => true,
        (RightParen { .. }, RightParen { .. }) => true,
        (Colon { .. }, Colon { .. }) => true,
        (Identifier { content: c1, .. }, Identifier { content: c2, .. }) => c1 == c2,
        (RefMarker { content: c1, .. }, RefMarker { content: c2, .. }) => c1 == c2,
        (
            FootnoteRef {
                footnote_type: f1, ..
            },
            FootnoteRef {
                footnote_type: f2, ..
            },
        ) => f1 == f2,
        (VerbatimTitle { content: c1, .. }, VerbatimTitle { content: c2, .. }) => c1 == c2,
        (VerbatimContent { content: c1, .. }, VerbatimContent { content: c2, .. }) => c1 == c2,
        (VerbatimLabel { content: c1, .. }, VerbatimLabel { content: c2, .. }) => c1 == c2,
        (
            Parameter {
                key: k1, value: v1, ..
            },
            Parameter {
                key: k2, value: v2, ..
            },
        ) => k1 == k2 && v1 == v2,
        (BoldDelimiter { .. }, BoldDelimiter { .. }) => true,
        (ItalicDelimiter { .. }, ItalicDelimiter { .. }) => true,
        (CodeDelimiter { .. }, CodeDelimiter { .. }) => true,
        (MathDelimiter { .. }, MathDelimiter { .. }) => true,
        (CitationRef { content: c1, .. }, CitationRef { content: c2, .. }) => c1 == c2,
        (PageRef { content: c1, .. }, PageRef { content: c2, .. }) => c1 == c2,
        (SessionRef { content: c1, .. }, SessionRef { content: c2, .. }) => c1 == c2,
        (Eof { .. }, Eof { .. }) => true,
        _ => false,
    }
}

// Basic text tests

#[test]
fn test_detokenize_simple_text() {
    let original = "Hello, world!";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_paragraph() {
    let original = "This is a paragraph.\nIt has multiple lines.\nAnd ends here.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_with_blank_lines() {
    let original = "First paragraph.\n\nSecond paragraph.";
    round_trip_test(original).expect("Round trip failed");
}

// List tests

#[test]
fn test_detokenize_simple_list() {
    let original = "- First item\n- Second item\n- Third item";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_numbered_list() {
    let original = "1. First item\n2. Second item\n3. Third item";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_alphabetical_list() {
    let original = "a. First item\nb. Second item\nc. Third item";
    round_trip_test(original).expect("Round trip failed");
}

// Annotation tests

#[test]
fn test_detokenize_annotation() {
    let original = ":: author :: John Doe";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_annotation_block() {
    let original = ":: note ::\n\n    This is a note.";
    round_trip_test(original).expect("Round trip failed");
}

// Definition tests

#[test]
fn test_detokenize_definition() {
    let original = "Term ::\n\n    Definition content here.";
    round_trip_test(original).expect("Round trip failed");
}

// Session tests

#[test]
fn test_detokenize_simple_session() {
    let original = "1. Session Title\n\n    Content in session.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_unnumbered_session() {
    let original = "Session Title\n\n    Content in session.";
    round_trip_test(original).expect("Round trip failed");
}

// Verbatim tests

#[test]
fn test_detokenize_verbatim() {
    let original = "code:\n    fn main() {\n        println!(\"Hello\");\n    }\n:: rust";
    round_trip_test(original).expect("Round trip failed");
}

// Inline formatting tests

#[test]
fn test_detokenize_bold() {
    let original = "This is *bold* text.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_italic() {
    let original = "This is _italic_ text.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_code() {
    let original = "This is `code` text.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_math() {
    let original = "This is #math# text.";
    round_trip_test(original).expect("Round trip failed");
}

// Reference tests

#[test]
fn test_detokenize_citation() {
    let original = "See [@smith2020] for details.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_page_ref() {
    let original = "See [p.42] for details.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_session_ref() {
    let original = "See [#1.2] for details.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_footnote_ref() {
    let original = "Some text[1] with footnote.";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_labeled_footnote() {
    let original = "Some text[^note] with footnote.";
    round_trip_test(original).expect("Round trip failed");
}

// Complex nested structure tests

#[test]
fn test_detokenize_nested_structure() {
    let original = r#"1. Main Section

    First paragraph in section.
    
    - List item one
    - List item two
    
    Another paragraph."#;
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_deeply_nested() {
    let original = r#"1. Level 1

    Content at level 1.
    
    1.1. Level 2
    
        Content at level 2.
        
        - Nested list
            With indented content"#;
    round_trip_test(original).expect("Round trip failed");
}

// Block group tests

#[test]
fn test_detokenize_from_block_groups() {
    let original = "Parent\n    Child 1\n    Child 2\nBack to parent";

    // Tokenize and group
    let tokens = tokenize(original);
    let grouper = BlockGrouper::new();
    let blocks = grouper.group_blocks(tokens).unwrap();

    // Detokenize from blocks
    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer.detokenize(&blocks).unwrap();

    // Re-tokenize and compare
    let tokens2 = tokenize(&reconstructed);
    let tokens1 = tokenize(original);

    assert_eq!(tokens1.len(), tokens2.len(), "Token count mismatch");
}

// Edge cases

#[test]
fn test_detokenize_empty_string() {
    let original = "";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_only_whitespace() {
    // Note: whitespace normalization may occur
    let original = "   \n   \n   ";
    let tokens = tokenize(original);
    let detokenizer = Detokenizer::new();
    let result = detokenizer.detokenize_tokens(&tokens).unwrap();
    // Just verify it doesn't crash - exact whitespace may differ
    assert!(result.trim().is_empty());
}

// Walkthrough document test

#[test]
fn test_detokenize_walkthrough_snippet() {
    let original = r#"TXXT :: A Radical Take on Minimal Structured Text

TXXT is a plain text format designed for simplicity and expressiveness.

1. What is TXXT?

    TXXT (pronounced "text") is a structured plain text format.
    
    It features:
    - Human readability
    - Machine parsability"#;

    round_trip_test(original).expect("Round trip failed");
}

// Parameter tests

#[test]
fn test_detokenize_parameters() {
    let original = ":: note:id=123,type=info ::";
    round_trip_test(original).expect("Round trip failed");
}

#[test]
fn test_detokenize_parameters_with_quotes() {
    let original = r#":: note:title="Hello, World",author=John ::"#;
    round_trip_test(original).expect("Round trip failed");
}

// Multi-line verbatim test

#[test]
fn test_detokenize_multiline_verbatim() {
    let original = r#"Example code:
    def hello():
        print("Hello, World!")
        return 42
:: python"#;
    round_trip_test(original).expect("Round trip failed");
}
