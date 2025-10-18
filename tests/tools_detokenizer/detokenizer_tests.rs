//! Tests for the Detokenizer - Round-trip Verification
//!
//! These tests verify the detokenizer's ability to reconstruct source text
//! from tokens, enabling round-trip verification of the tokenization process.

use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;
use txxt::tools::detokenizer::Detokenizer;

/// Helper function to verify detokenization produces identical tokens for round-trip verification
fn verify_round_trip(original: &str) {
    // Step 1: ScannerTokenize
    let tokens1 = tokenize(original);

    // Step 2: Simple detokenization for verification (not using BlockGroup as it's for parsing)
    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer
        .detokenize_for_verification(&tokens1)
        .expect("Detokenization should succeed");

    // Step 3: Re-tokenize
    let tokens2 = tokenize(&reconstructed);

    // Step 4: Compare tokens (not strings)
    assert_eq!(
        tokens1.len(),
        tokens2.len(),
        "Token count mismatch for input: {}\nReconstructed: {}",
        original,
        reconstructed
    );

    for (i, (t1, t2)) in tokens1.iter().zip(tokens2.iter()).enumerate() {
        assert!(
            tokens_equal(t1, t2),
            "Token mismatch at position {} for input: {}\nExpected: {:?}\nGot: {:?}",
            i,
            original,
            t1,
            t2
        );
    }
}

/// Compare tokens for equality (ignoring source spans)
fn tokens_equal(t1: &ScannerToken, t2: &ScannerToken) -> bool {
    use ScannerToken::*;
    match (t1, t2) {
        (Text { content: c1, .. }, Text { content: c2, .. }) => c1 == c2,
        (Newline { .. }, Newline { .. }) => true,
        (BlankLine { .. }, BlankLine { .. }) => true,
        (Indent { .. }, Indent { .. }) => true,
        (Dedent { .. }, Dedent { .. }) => true,
        // IndentationWall and Indent tokens are semantically equivalent
        (IndentationWall { .. }, Indent { .. }) => true,
        (Indent { .. }, IndentationWall { .. }) => true,
        (
            SequenceMarker {
                marker_type: m1, ..
            },
            SequenceMarker {
                marker_type: m2, ..
            },
        ) => m1 == m2,
        (TxxtMarker { .. }, TxxtMarker { .. }) => true,
        (Dash { .. }, Dash { .. }) => true,
        (Period { .. }, Period { .. }) => true,
        (LeftBracket { .. }, LeftBracket { .. }) => true,
        (RightBracket { .. }, RightBracket { .. }) => true,
        (AtSign { .. }, AtSign { .. }) => true,
        (LeftParen { .. }, LeftParen { .. }) => true,
        (RightParen { .. }, RightParen { .. }) => true,
        (Colon { .. }, Colon { .. }) => true,
        (Equals { .. }, Equals { .. }) => true,
        (Comma { .. }, Comma { .. }) => true,
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
        (
            IndentationWall {
                level: l1,
                wall_type: wt1,
                ..
            },
            IndentationWall {
                level: l2,
                wall_type: wt2,
                ..
            },
        ) => l1 == l2 && wt1 == wt2,
        (IgnoreTextSpan { content: c1, .. }, IgnoreTextSpan { content: c2, .. }) => c1 == c2,
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
        (Whitespace { content: c1, .. }, Whitespace { content: c2, .. }) => c1 == c2,
        (Eof { .. }, Eof { .. }) => true,
        _ => false,
    }
}

// ===== LEVEL 1: Basic Text =====

#[test]
fn test_single_word() {
    verify_round_trip("Hello");
}

#[test]
fn test_two_words() {
    verify_round_trip("Hello world");
}

#[test]
fn test_sentence_with_punctuation() {
    verify_round_trip("Hello, world!");
}

#[test]
fn test_empty_string() {
    verify_round_trip("");
}

// ===== LEVEL 2: Paragraphs =====

#[test]
fn test_single_line_paragraph() {
    verify_round_trip("This is a paragraph.");
}

#[test]
fn test_two_line_paragraph() {
    verify_round_trip("First line.\nSecond line.");
}

#[test]
fn test_multi_line_paragraph() {
    verify_round_trip("First line.\nSecond line.\nThird line.");
}

#[test]
fn test_two_paragraphs() {
    verify_round_trip("First paragraph.\n\nSecond paragraph.");
}

#[test]
fn test_three_paragraphs() {
    verify_round_trip("First paragraph.\n\nSecond paragraph.\n\nThird paragraph.");
}

// ===== LEVEL 3: Simple Lists =====

#[test]
fn test_single_dash_item() {
    verify_round_trip("- Item");
}

#[test]
fn test_two_dash_items() {
    verify_round_trip("- First\n- Second");
}

#[test]
fn test_single_numbered_item() {
    verify_round_trip("1. Item");
}

#[test]
fn test_two_numbered_items() {
    verify_round_trip("1. First\n2. Second");
}

#[test]
fn test_single_alpha_item() {
    verify_round_trip("a. Item");
}

#[test]
fn test_mixed_list_types() {
    verify_round_trip("1. Numbered\na. Alpha\n- Dash");
}

// ===== LEVEL 4: Inline Formatting =====

#[test]
fn test_bold_single_word() {
    verify_round_trip("*bold*");
}

#[test]
fn test_bold_in_sentence() {
    verify_round_trip("This is *bold* text.");
}

#[test]
fn test_italic_single_word() {
    verify_round_trip("_italic_");
}

#[test]
fn test_italic_in_sentence() {
    verify_round_trip("This is _italic_ text.");
}

#[test]
fn test_code_single_word() {
    verify_round_trip("`code`");
}

#[test]
fn test_code_in_sentence() {
    verify_round_trip("This is `code` text.");
}

#[test]
fn test_math_single_word() {
    verify_round_trip("#math#");
}

#[test]
fn test_math_in_sentence() {
    verify_round_trip("This is #math# text.");
}

// ===== LEVEL 5: References =====

#[test]
fn test_footnote_naked() {
    verify_round_trip("[1]");
}

#[test]
fn test_footnote_in_text() {
    verify_round_trip("Text[1] here.");
}

#[test]
fn test_footnote_labeled() {
    verify_round_trip("[^note]");
}

#[test]
fn test_page_ref() {
    verify_round_trip("[p.42]");
}

#[test]
fn test_page_ref_in_text() {
    verify_round_trip("See [p.42] for details.");
}

#[test]
fn test_citation_ref() {
    verify_round_trip("[@smith2020]");
}

#[test]
fn test_citation_ref_in_text() {
    verify_round_trip("As noted [@smith2020] in the study.");
}

#[test]
fn test_session_ref() {
    verify_round_trip("[#1.2]");
}

// ===== LEVEL 6: Annotations and Definitions =====

#[test]
fn test_simple_annotation() {
    verify_round_trip(":: note ::");
}

#[test]
fn test_annotation_with_content() {
    verify_round_trip(":: author :: John Doe");
}

#[test]
fn test_simple_definition() {
    verify_round_trip("Term ::");
}

#[test]
fn test_definition_with_content() {
    verify_round_trip("Term :: definition text");
}

// ===== LEVEL 7: Indented Content =====

#[test]
fn test_simple_indented_line() {
    verify_round_trip("    Indented content");
}

#[test]
fn test_list_with_indented_content() {
    verify_round_trip("- Item\n    Indented under item");
}

#[test]
fn test_numbered_list_with_indent() {
    verify_round_trip("1. First\n    Details");
}

// ===== LEVEL 8: Sessions =====

#[test]
fn test_simple_session() {
    verify_round_trip("Title");
}

#[test]
fn test_numbered_session() {
    verify_round_trip("1. Session Title");
}

#[test]
fn test_session_with_content() {
    verify_round_trip("Title\n\n    Content");
}

// ===== LEVEL 9: Verbatim Blocks =====

#[test]
fn test_simple_verbatim() {
    verify_round_trip("code:\n    print(\"hello\")");
}

#[test]
fn test_verbatim_with_label() {
    verify_round_trip("code:\n    print(\"hello\")\n:: python");
}

// ===== LEVEL 10: Parameters =====

#[test]
fn test_annotation_with_param() {
    verify_round_trip(":: note:id=123 ::");
}

#[test]
fn test_annotation_with_two_params() {
    verify_round_trip(":: note:id=123,type=info ::");
}

#[test]
fn test_param_with_quoted_value() {
    verify_round_trip(":: note:title=\"Hello, World\" ::");
}

// ===== LEVEL 11: Complex Nesting =====

#[test]
fn test_nested_lists() {
    verify_round_trip("1. First\n    - Sub item\n    - Another sub");
}

#[test]
fn test_deeply_nested() {
    verify_round_trip("1. Level 1\n    1.1. Level 2\n        - Level 3");
}

#[test]
fn test_annotation_block() {
    verify_round_trip(":: note ::\n\n    This is a note.");
}

#[test]
fn test_definition_block() {
    verify_round_trip("Term ::\n\n    Definition content.");
}

// ===== LEVEL 12: Mixed Content =====

#[test]
fn test_paragraph_with_formatting() {
    verify_round_trip("This has *bold* and _italic_ and `code`.");
}

#[test]
fn test_list_with_references() {
    verify_round_trip("- See [p.42]\n- Check [@smith2020]\n- Note[1]");
}
