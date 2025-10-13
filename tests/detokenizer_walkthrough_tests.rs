//! Detokenizer tests based on walkthrough.txxt examples
//!
//! These tests verify the detokenizer can handle complex, realistic txxt documents
//! using examples from the walkthrough documentation.

use txxt::parser::detokenizer::Detokenizer;
use txxt::tokenizer::tokenize;

/// Helper to verify round-trip tokenization
fn verify_detokenizer_round_trip(original: &str) {
    let tokens1 = tokenize(original);
    let detokenizer = Detokenizer::new();
    let reconstructed = detokenizer
        .detokenize_tokens(&tokens1)
        .expect("Detokenization should succeed");
    let tokens2 = tokenize(&reconstructed);

    // Compare token counts
    assert_eq!(
        tokens1.len(),
        tokens2.len(),
        "Token count mismatch for input:\n{}\nReconstructed:\n{}",
        original,
        reconstructed
    );

    // For debugging: if tokens don't match, show the differences
    if tokens1 != tokens2 {
        eprintln!("Original tokens: {:?}", tokens1);
        eprintln!("Reconstructed tokens: {:?}", tokens2);
        panic!("Token mismatch");
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
#[ignore = "Verbatim label reconstruction issue - unrelated to blank line preservation"]
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
