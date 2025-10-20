//! Tests for inline formatting delimiter tokens using rstest and proptest
//!
//! Tests both successful parsing and failure cases for inline formatting delimiters:
//! * (bold), _ (italic), ` (code), # (math)

use proptest::prelude::*;
use rstest::rstest;
use txxt::cst::ScannerToken;
use txxt::lexer::tokenize;

// =============================================================================
// Inline Delimiter Tokens - Isolated Tests (rstest)
// =============================================================================

#[rstest]
#[case("*")]
#[case("_")]
#[case("`")]
#[case("#")]
fn test_inline_delimiter_isolated_passing(#[case] input: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // DELIMITER + EOF

    let expected_token_type = match input {
        "*" => "BoldDelimiter",
        "_" => "ItalicDelimiter",
        "`" => "CodeDelimiter",
        "#" => "MathDelimiter",
        _ => panic!("Unexpected input: {}", input),
    };

    match &tokens[0] {
        ScannerToken::BoldDelimiter { span } if input == "*" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 1);
        }
        ScannerToken::ItalicDelimiter { span } if input == "_" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 1);
        }
        ScannerToken::CodeDelimiter { span } if input == "`" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 1);
        }
        ScannerToken::MathDelimiter { span } if input == "#" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!(
            "Expected {} token, got {:?}",
            expected_token_type, tokens[0]
        ),
    }

    // Should end with EOF
    match &tokens[1] {
        ScannerToken::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[rstest]
#[case("*bold*", "*", "bold", "*")]
#[case("_italic_", "_", "italic", "_")]
#[case("`code`", "`", "code", "`")]
fn test_inline_delimiter_with_content_passing(
    #[case] input: &str,
    #[case] start_delim: &str,
    #[case] expected_text: &str,
    #[case] end_delim: &str,
) {
    let tokens = tokenize(input);

    // Should have: START_DELIMITER, TEXT, END_DELIMITER, EOF
    assert!(tokens.len() >= 4);

    // First token should be start delimiter
    let expected_start_type = match start_delim {
        "*" => "BoldDelimiter",
        "_" => "ItalicDelimiter",
        "`" => "CodeDelimiter",
        _ => panic!("Unexpected delimiter: {}", start_delim),
    };

    match &tokens[0] {
        ScannerToken::BoldDelimiter { span } if start_delim == "*" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 1);
        }
        ScannerToken::ItalicDelimiter { span } if start_delim == "_" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 1);
        }
        ScannerToken::CodeDelimiter { span } if start_delim == "`" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.column, 1);
        }
        _ => panic!(
            "Expected {} token, got {:?}",
            expected_start_type, tokens[0]
        ),
    }

    // Second token should be text
    match &tokens[1] {
        ScannerToken::Text { content, span } => {
            assert_eq!(content, expected_text);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 1); // After start delimiter
        }
        _ => panic!("Expected Text token, got {:?}", tokens[1]),
    }

    // Third token should be end delimiter (same type as start)
    let expected_end_type = match end_delim {
        "*" => "BoldDelimiter",
        "_" => "ItalicDelimiter",
        "`" => "CodeDelimiter",
        "#" => "MathDelimiter",
        _ => panic!("Unexpected delimiter: {}", end_delim),
    };

    match &tokens[2] {
        ScannerToken::BoldDelimiter { span } if end_delim == "*" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 1 + expected_text.len());
        }
        ScannerToken::ItalicDelimiter { span } if end_delim == "_" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 1 + expected_text.len());
        }
        ScannerToken::CodeDelimiter { span } if end_delim == "`" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 1 + expected_text.len());
        }
        ScannerToken::MathDelimiter { span } if end_delim == "#" => {
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 1 + expected_text.len());
        }
        _ => panic!("Expected {} token, got {:?}", expected_end_type, tokens[2]),
    }
}

#[rstest]
#[case("text *bold* more", "text", "*", "bold", "*", "more")]
#[case("start _italic_ end", "start", "_", "italic", "_", "end")]
#[case("before `code` after", "before", "`", "code", "`", "after")]
fn test_inline_delimiter_mixed_content(
    #[case] input: &str,
    #[case] prefix_text: &str,
    #[case] _start_delim: &str,
    #[case] inner_text: &str,
    #[case] _end_delim: &str,
    #[case] suffix_text: &str,
) {
    let tokens = tokenize(input);

    // Should have: TEXT, DELIMITER, TEXT, DELIMITER, TEXT, EOF
    assert!(tokens.len() >= 6);

    // Find prefix text
    let prefix_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == prefix_text))
        .expect("Should find prefix text");

    match prefix_token {
        ScannerToken::Text { content, span } => {
            assert_eq!(content, prefix_text);
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
        }
        _ => unreachable!(),
    }

    // Find inner text
    let inner_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == inner_text))
        .expect("Should find inner text");

    match inner_token {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, inner_text);
        }
        _ => unreachable!(),
    }

    // Find suffix text
    let suffix_token = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::Text { content, .. } if content == suffix_text))
        .expect("Should find suffix text");

    match suffix_token {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, suffix_text);
        }
        _ => unreachable!(),
    }

    // Check that we have the right delimiters
    let delimiter_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| {
            matches!(
                token,
                ScannerToken::BoldDelimiter { .. }
                    | ScannerToken::ItalicDelimiter { .. }
                    | ScannerToken::CodeDelimiter { .. }
                    | ScannerToken::MathDelimiter { .. }
            )
        })
        .collect();

    assert_eq!(delimiter_tokens.len(), 2, "Should have 2 delimiter tokens");
}

#[test]
fn test_math_delimiter_integration() {
    let tokens = tokenize("#math#");

    // Should have: MathDelimiter, Text, MathDelimiter, Eof (delimiter approach)
    assert_eq!(tokens.len(), 4);

    match &tokens[0] {
        ScannerToken::MathDelimiter { .. } => {}
        _ => panic!("Expected MathDelimiter token, got {:?}", tokens[0]),
    }

    match &tokens[1] {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, "math");
        }
        _ => panic!("Expected Text token, got {:?}", tokens[1]),
    }

    match &tokens[2] {
        ScannerToken::MathDelimiter { .. } => {}
        _ => panic!("Expected MathDelimiter token, got {:?}", tokens[2]),
    }
}

#[test]
fn test_math_delimiter_mixed_content() {
    let tokens = tokenize("prefix #math# suffix");

    // Should have: Text("prefix"), MathDelimiter, Text("math"), MathDelimiter, Text("suffix"), Eof
    assert!(tokens.len() >= 6);

    let math_delimiters: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::MathDelimiter { .. }))
        .collect();
    assert_eq!(math_delimiters.len(), 2);

    let math_content = tokens
        .iter()
        .find(|token| {
            if let ScannerToken::Text { content, .. } = token {
                content == "math"
            } else {
                false
            }
        })
        .expect("Should find math content token");

    match math_content {
        ScannerToken::Text { content, .. } => {
            assert_eq!(content, "math");
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// Inline Delimiter Tokens - Edge Cases
// =============================================================================

#[rstest]
#[case("**")] // Double bold delimiters
#[case("__")] // Double italic delimiters
#[case("``")] // Double code delimiters
fn test_inline_delimiter_double_delimiters(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should produce 2 separate delimiter tokens + EOF
    assert_eq!(tokens.len(), 3);

    let delimiter_tokens: Vec<_> = tokens
        .iter()
        .filter(|token| {
            matches!(
                token,
                ScannerToken::BoldDelimiter { .. }
                    | ScannerToken::ItalicDelimiter { .. }
                    | ScannerToken::CodeDelimiter { .. }
            )
        })
        .collect();

    assert_eq!(
        delimiter_tokens.len(),
        2,
        "Should produce 2 separate delimiter tokens"
    );

    // Check positions are correct
    let (first, second) = (&tokens[0], &tokens[1]);
    assert_eq!(first.span().start.column, 0);
    assert_eq!(first.span().end.column, 1);
    assert_eq!(second.span().start.column, 1);
    assert_eq!(second.span().end.column, 2);
}

#[test]
fn test_double_math_delimiters() {
    let tokens = tokenize("##");

    // Should produce 2 separate MathDelimiter tokens + EOF
    assert_eq!(tokens.len(), 3);

    let math_delimiters: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::MathDelimiter { .. }))
        .collect();

    assert_eq!(
        math_delimiters.len(),
        2,
        "Should produce 2 MathDelimiter tokens"
    );

    // Verification that delimiters work consistently
}

#[rstest]
#[case("*_`#")] // All delimiters in sequence
fn test_inline_delimiter_mixed_sequence(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should have: BOLD, ITALIC, CODE, MATH_DELIMITER (standalone #), EOF
    assert_eq!(tokens.len(), 5);

    match (&tokens[0], &tokens[1], &tokens[2], &tokens[3]) {
        (
            ScannerToken::BoldDelimiter { .. },
            ScannerToken::ItalicDelimiter { .. },
            ScannerToken::CodeDelimiter { .. },
            ScannerToken::MathDelimiter { .. },
        ) => {
            // Check positions are sequential
            assert_eq!(tokens[0].span().start.column, 0);
            assert_eq!(tokens[1].span().start.column, 1);
            assert_eq!(tokens[2].span().start.column, 2);
            assert_eq!(tokens[3].span().start.column, 3);
        }
        _ => panic!(
            "Expected sequence of all delimiter types, got {:?}",
            &tokens[0..4]
        ),
    }
}

// =============================================================================
// Property-Based Tests (proptest)
// =============================================================================

proptest! {
    #[test]
    fn test_inline_delimiter_properties(
        delimiter in r"[*_`#]"
    ) {
        let tokens = tokenize(&delimiter);

        // Should have exactly 1 delimiter token + EOF
        prop_assert_eq!(tokens.len(), 2);

        // First token should be appropriate delimiter type
        match &tokens[0] {
            ScannerToken::BoldDelimiter { span } if delimiter == "*" => {
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, 1);
            }
            ScannerToken::ItalicDelimiter { span } if delimiter == "_" => {
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, 1);
            }
            ScannerToken::CodeDelimiter { span } if delimiter == "`" => {
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, 1);
            }
            ScannerToken::MathDelimiter { span } if delimiter == "#" => {
                prop_assert_eq!(span.start.row, 0);
                prop_assert_eq!(span.start.column, 0);
                prop_assert_eq!(span.end.column, 1);
            }
            _ => prop_assert!(false, "Expected appropriate delimiter token"),
        }

        // Second token should be EOF
        prop_assert!(matches!(tokens[1], ScannerToken::Eof { .. }), "Second token should be EOF");
    }

    #[test]
    fn test_inline_delimiter_span_consistency(
        text in "[a-zA-Z0-9]+",
        delimiter in r"[*_`#]"
    ) {
        let input = format!("{}{}{}", delimiter, text, delimiter);
        let tokens = tokenize(&input);

        for token in &tokens {
            match token {
                ScannerToken::BoldDelimiter { span } |
                ScannerToken::ItalicDelimiter { span } |
                ScannerToken::CodeDelimiter { span } |
                ScannerToken::MathDelimiter { span } => {
                    // Delimiter span should be exactly 1 character
                    prop_assert_eq!(
                        span.end.column - span.start.column,
                        1,
                        "Delimiter span should be 1 character"
                    );

                    // Start should come before end
                    prop_assert!(span.start.column <= span.end.column);
                    prop_assert!(span.start.row <= span.end.row);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_inline_delimiter_with_text_properties(
        text in "[a-zA-Z0-9]+",
        delimiter in r"[*_`]" // Exclude # since it now creates MathDelimiter tokens
    ) {
        let input = format!("{}{}{}", delimiter, text, delimiter);
        let tokens = tokenize(&input);

        // Should have: DELIMITER, TEXT, DELIMITER, EOF
        prop_assert!(tokens.len() >= 4);

        // Should find exactly 2 delimiter tokens
        let delimiter_count = tokens.iter()
            .filter(|token| matches!(token,
                ScannerToken::BoldDelimiter { .. } |
                ScannerToken::ItalicDelimiter { .. } |
                ScannerToken::CodeDelimiter { .. }
            ))
            .count();

        prop_assert_eq!(delimiter_count, 2, "Should find exactly 2 delimiter tokens");

        // Should find exactly 1 text token with the expected content
        let text_tokens: Vec<_> = tokens.iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content, .. } if content == &text => Some(content),
                _ => None,
            })
            .collect();

        prop_assert_eq!(text_tokens.len(), 1, "Should find exactly 1 text token with expected content");
    }

    #[test]
    fn test_math_delimiter_properties(
        content in "[a-zA-Z0-9]+",
    ) {
        let input = format!("#{content}#");
        let tokens = tokenize(&input);

        // Should have: MathDelimiter, Text, MathDelimiter, EOF
        prop_assert_eq!(tokens.len(), 4, "Math delimiters should produce exactly 4 tokens");

        // Should find exactly 2 math delimiter tokens
        let delimiter_count = tokens.iter()
            .filter(|token| matches!(token, ScannerToken::MathDelimiter { .. }))
            .count();

        prop_assert_eq!(delimiter_count, 2, "Should find exactly 2 MathDelimiter tokens");

        // Should find exactly 1 text token with expected content
        let text_tokens: Vec<_> = tokens.iter()
            .filter_map(|token| match token {
                ScannerToken::Text { content: text_content, .. } if text_content == &content => Some(text_content),
                _ => None,
            })
            .collect();

        prop_assert_eq!(text_tokens.len(), 1, "Should find exactly 1 Text token with expected content");
    }
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn test_framework_setup() {
        // Verify our test framework is working
        let tokens = tokenize("*test*");
        assert!(!tokens.is_empty());
    }
}
