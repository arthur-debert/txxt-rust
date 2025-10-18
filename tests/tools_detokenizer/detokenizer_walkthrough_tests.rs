//! Detokenizer tests based on walkthrough.txxt examples
//!
//! These tests verify the detokenizer can handle complex, realistic txxt documents
//! using examples from the walkthrough documentation.

use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;
use txxt::tools::detokenizer::Detokenizer;

/// Compare tokens for equality (ignoring source spans)
fn tokens_equal(t1: &ScannerToken, t2: &ScannerToken) -> bool {
    use ScannerToken::*;
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
        (Whitespace { content: c1, .. }, Whitespace { content: c2, .. }) => c1 == c2,
        (Eof { .. }, Eof { .. }) => true,
        _ => false,
    }
}

/// Helper to verify round-trip tokenization for verification purposes
fn verify_detokenizer_round_trip(original: &str) {
    let tokens1 = tokenize(original);

    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer
        .detokenize_for_verification(&tokens1)
        .expect("Detokenization should succeed");
    let tokens2 = tokenize(&reconstructed);

    // Compare tokens one by one to find exact mismatch
    let max_len = tokens1.len().max(tokens2.len());
    for i in 0..max_len {
        let t1_opt = tokens1.get(i);
        let t2_opt = tokens2.get(i);

        match (t1_opt, t2_opt) {
            (Some(t1), Some(t2)) => {
                if !tokens_equal(t1, t2) {
                    panic!(
                        "Token mismatch at position {}:\n  Expected: {:?}\n  Got:      {:?}\n\nOriginal:\n{}\n\nReconstructed:\n{}",
                        i, t1, t2, original, reconstructed
                    );
                }
            }
            (Some(t1), None) => {
                panic!(
                    "Missing token at position {}:\n  Expected: {:?}\n  Got:      MISSING\n\nOriginal:\n{}\n\nReconstructed:\n{}",
                    i, t1, original, reconstructed
                );
            }
            (None, Some(t2)) => {
                panic!(
                    "Extra token at position {}:\n  Expected: MISSING\n  Got:      {:?}\n\nOriginal:\n{}\n\nReconstructed:\n{}",
                    i, t2, original, reconstructed
                );
            }
            (None, None) => break,
        }
    }
}

#[test]
fn test_nested_numbered_sessions() {
    let input = r#"1. Introduction

    This is the introduction content, indented one level from the session title.
    
    This paragraph continues the introduction section.
    
2. Core Concepts

    2.1. Sessions
    
        Sessions can be nested to arbitrary depth, each with their own content."#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_mixed_list_types_nested() {
    let input = r#"Nested example:
1. Groceries
    - Milk
    - Bread
        a. Whole wheat
        b. Sourdough
2. Hardware store
    - Screws
    - Paint"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_verbatim_block_with_label_and_params() {
    let input = r#"Advanced example with parameters:
    def fibonacci(n):
        return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)
:: python:version=3.11,style=functional"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_definition_with_complex_content() {
    let input = r#"Recursion ::
    A programming technique where a function calls itself to solve smaller instances of the same problem.

    Definitions can contain complex content:
    - Multiple paragraphs
    - Lists and examples
    - Even verbatim blocks"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_inline_formatting_combined() {
    let input = r#"Within any text content, you can apply formatting using paired markers:

- Use asterisks for *bold* text
- Underscores for _italic_ text  
- Backticks for `monospace code`
- Hash signs for mathematical expressions: #E = mc^2#

Inline formats can be nested: *bold with _italic_ inside* but cannot span multiple lines."#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_references_various_types() {
    let input = r#"5.1. Basic References

    - External URLs: [https://example.com]
    - Simple domains: [example.com]  
    - File paths: [./other-doc.txxt] or [/absolute/path.txxt]
    - Section references: [#2.1] (this section)
    - Footnote style: [1] (links to last section, item 1)"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_academic_citations() {
    let input = r#"Academic documents use @ prefix for bibliographic references:

- Single citation: [@smith2023]
- Multiple citations: [@smith2023; @jones2024]
- With page numbers: [@smith2023, p. 45]"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_annotations_with_parameters() {
    let input = r#":: title :: Document Title
:: author :: Author Name
:: note :: This is a helpful annotation

Annotations can have parameters.
:: warning:severity=high :: Critical security information"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_multi_level_session_nesting() {
    let input = r#"1. Top Level

    Content for top level.

    1.1. Second Level
    
        Content for second level.
        
        1.1.1. Third Level
        
            Deep nesting works correctly.
            
    1.2. Another Second Level
    
        More content here."#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_verbatim_blocks_various() {
    let input = r#"Basic syntax:
    console.log("Hello, txxt!");
    alert("This content is preserved exactly");
:: javascript

Empty title example:
    raw content here
    more raw content
:: format"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_escaping_special_chars() {
    let input = r#"Use backslash to display special characters literally:
- `\*not bold\*` displays as *not bold*
- `\- not a list` prevents list parsing
- `\\` displays as a single backslash"#;

    verify_detokenizer_round_trip(input);
}

#[test]
fn test_complex_document_structure() {
    // A more comprehensive test combining multiple elements
    let input = r#":: title :: Complex Document Test
:: author :: Test Suite

1. Introduction

    This document tests various txxt features in combination.

2. Lists and Formatting

    Here we combine different list types:
    
    1. First numbered item with *bold text*
    2. Second item with _italic text_
        - Nested dash item with `code`
        - Another nested item
            a. Deep nesting with #math#
            b. More deep nesting
    
3. Definitions and References

    Parser ::
        A program that analyzes text[1] according to formal grammar rules.
        
        See also [@smith2023] for more details.

4. Code Examples

    Python example:
        def hello():
            print("Hello from txxt!")
    :: python
    
    The above code demonstrates basic syntax."#;

    verify_detokenizer_round_trip(input);
}
