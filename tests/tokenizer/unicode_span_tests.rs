//! Comprehensive Unicode span tests for all tokenizers
//!
//! These tests ensure that ALL tokenizers correctly handle multi-byte Unicode
//! characters when calculating spans. This is critical for language server
//! functionality and any position-based features.

use txxt::ast::tokens::Token;
use txxt::tokenizer::Lexer;

/// Test that all tokenizers handle emoji (4-byte characters) correctly
#[test]
fn test_all_tokenizers_with_emoji() {
    // This test verifies that inline markers and special delimiters are properly
    // recognized after Unicode characters, and that column positions are calculated
    // based on character count (not byte count).

    struct TestCase {
        input: &'static str,
        description: &'static str,
        expected_tokens: Vec<ExpectedToken>,
    }

    #[allow(dead_code)]
    enum ExpectedToken {
        Text(&'static str),
        Dash,
        AnnotationMarker,
        BoldDelimiter,
        ItalicDelimiter,
        CodeDelimiter,
        MathDelimiter,
        LeftBracket,
        RightBracket,
        RefMarker,
        LeftParen,
        RightParen,
        Whitespace,
    }

    let test_cases = vec![
        TestCase {
            input: "🎉text",
            description: "emoji+text forms single token",
            expected_tokens: vec![ExpectedToken::Text("🎉text")],
        },
        TestCase {
            input: "🎉- item",
            description: "dash after emoji is recognized",
            expected_tokens: vec![
                ExpectedToken::Text("🎉"),
                ExpectedToken::Dash,
                ExpectedToken::Whitespace,
                ExpectedToken::Text("item"),
            ],
        },
        TestCase {
            input: "🎉:: label ::",
            description: "annotation markers after emoji",
            expected_tokens: vec![ExpectedToken::Text("🎉"), ExpectedToken::AnnotationMarker],
        },
        TestCase {
            input: "🎉*bold*",
            description: "bold delimiters after emoji",
            expected_tokens: vec![
                ExpectedToken::Text("🎉"),
                ExpectedToken::BoldDelimiter,
                ExpectedToken::Text("bold"),
                ExpectedToken::BoldDelimiter,
            ],
        },
        TestCase {
            input: "🎉_italic_",
            description: "italic delimiters after emoji - underscore is special",
            expected_tokens: vec![
                ExpectedToken::Text("🎉_italic"),
                ExpectedToken::ItalicDelimiter,
            ],
        },
        TestCase {
            input: "🎉 _italic_",
            description: "italic delimiters with space work normally",
            expected_tokens: vec![
                ExpectedToken::Text("🎉"),
                ExpectedToken::Whitespace,
                ExpectedToken::ItalicDelimiter,
                ExpectedToken::Text("italic"),
                ExpectedToken::ItalicDelimiter,
            ],
        },
        TestCase {
            input: "🎉`code`",
            description: "code delimiters after emoji",
            expected_tokens: vec![
                ExpectedToken::Text("🎉"),
                ExpectedToken::CodeDelimiter,
                ExpectedToken::Text("code"),
                ExpectedToken::CodeDelimiter,
            ],
        },
        TestCase {
            input: "🎉#math#",
            description: "math delimiters after emoji",
            expected_tokens: vec![
                ExpectedToken::Text("🎉"),
                ExpectedToken::MathDelimiter,
                ExpectedToken::Text("math"),
                ExpectedToken::MathDelimiter,
            ],
        },
        TestCase {
            input: "🎉[ref]",
            description: "brackets after emoji form RefMarker",
            expected_tokens: vec![ExpectedToken::Text("🎉"), ExpectedToken::RefMarker],
        },
        TestCase {
            input: "🎉@citation",
            description: "at-sign after emoji forms single token",
            expected_tokens: vec![ExpectedToken::Text("🎉@citation")],
        },
        TestCase {
            input: "🎉 @citation",
            description: "at-sign with space forms single text token",
            expected_tokens: vec![
                ExpectedToken::Text("🎉"),
                ExpectedToken::Whitespace,
                ExpectedToken::Text("@citation"),
            ],
        },
    ];

    for test_case in test_cases {
        let mut lexer = Lexer::new(test_case.input);
        let tokens = lexer.tokenize();

        let mut expected_col = 0;

        for (token_idx, expected) in test_case.expected_tokens.into_iter().enumerate() {
            assert!(
                token_idx < tokens.len() - 1, // -1 for Eof
                "{}: Expected more tokens",
                test_case.description
            );

            let token = &tokens[token_idx];

            match (expected, token) {
                (ExpectedToken::Text(expected_content), Token::Text { content, span }) => {
                    assert_eq!(
                        content, expected_content,
                        "{}: Expected text '{}', got '{}'",
                        test_case.description, expected_content, content
                    );
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: Text should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::Dash, Token::Dash { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: Dash should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::AnnotationMarker, Token::AnnotationMarker { span, .. }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: AnnotationMarker should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::BoldDelimiter, Token::BoldDelimiter { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: BoldDelimiter should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::ItalicDelimiter, Token::ItalicDelimiter { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: ItalicDelimiter should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::CodeDelimiter, Token::CodeDelimiter { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: CodeDelimiter should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::MathDelimiter, Token::MathDelimiter { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: MathDelimiter should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::LeftBracket, Token::LeftBracket { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: LeftBracket should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::RightBracket, Token::RightBracket { span }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: RightBracket should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::RefMarker, Token::RefMarker { span, .. }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: RefMarker should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (ExpectedToken::Whitespace, Token::Whitespace { span, .. }) => {
                    assert_eq!(
                        span.start.column, expected_col,
                        "{}: Whitespace should start at column {}",
                        test_case.description, expected_col
                    );
                    expected_col = span.end.column;
                }
                (expected, actual) => {
                    panic!(
                        "{}: Expected {:?}, got {:?}",
                        test_case.description,
                        match expected {
                            ExpectedToken::Text(_) => "Text",
                            ExpectedToken::Dash => "Dash",
                            ExpectedToken::AnnotationMarker => "AnnotationMarker",
                            ExpectedToken::BoldDelimiter => "BoldDelimiter",
                            ExpectedToken::ItalicDelimiter => "ItalicDelimiter",
                            ExpectedToken::CodeDelimiter => "CodeDelimiter",
                            ExpectedToken::MathDelimiter => "MathDelimiter",
                            ExpectedToken::LeftBracket => "LeftBracket",
                            ExpectedToken::RightBracket => "RightBracket",
                            ExpectedToken::RefMarker => "RefMarker",
                            ExpectedToken::LeftParen => "LeftParen",
                            ExpectedToken::RightParen => "RightParen",
                            ExpectedToken::Whitespace => "Whitespace",
                        },
                        actual
                    );
                }
            }
        }
    }
}

/// Test sequence markers with various Unicode scenarios
#[test]
fn test_sequence_markers_unicode() {
    // According to txxt spec, sequence markers MUST be at line start (column 0)
    // The cases "café- item", "→- item", "🎉- item" are NOT sequence markers
    // because they don't start at column 0. They are text followed by dash tokens.

    let valid_sequence_cases = vec![
        ("- café", 0, 1, "dash marker with accented content"),
        ("42. café", 0, 3, "numerical marker with accented content"),
        // Note: "à." is not a valid sequence marker - only ASCII alphabetical chars allowed
    ];

    for (input, expected_start, expected_end, description) in valid_sequence_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let marker = tokens
            .iter()
            .find(|t| matches!(t, Token::SequenceMarker { .. }))
            .unwrap_or_else(|| panic!("Should find sequence marker in: {}", description));

        match marker {
            Token::SequenceMarker { span, .. } => {
                assert_eq!(
                    span.start.column, expected_start,
                    "{}: marker start should be {} but was {}",
                    description, expected_start, span.start.column
                );
                assert_eq!(
                    span.end.column, expected_end,
                    "{}: marker end should be {} but was {}",
                    description, expected_end, span.end.column
                );
            }
            _ => unreachable!(),
        }
    }

    // Test cases that should NOT produce sequence markers
    let invalid_cases = vec![
        ("café- item", "text before dash - not a sequence marker"),
        ("→- item", "arrow before dash - not a sequence marker"),
        ("🎉- item", "emoji before dash - not a sequence marker"),
        (
            "à. item",
            "non-ASCII alphabetical - not a valid sequence marker",
        ),
    ];

    for (input, description) in invalid_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let has_sequence_marker = tokens
            .iter()
            .any(|t| matches!(t, Token::SequenceMarker { .. }));

        assert!(
            !has_sequence_marker,
            "{}: Should NOT produce a sequence marker",
            description
        );

        // These should produce dash tokens instead
        if input.contains('-') {
            let has_dash = tokens.iter().any(|t| matches!(t, Token::Dash { .. }));
            assert!(has_dash, "{}: Should have a dash token", description);
        }
    }
}

/// Test annotation and definition markers with Unicode
#[test]
fn test_annotation_definition_unicode() {
    // Annotation markers
    let input = "café :: label :: content";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let first_annotation = tokens
        .iter()
        .find(|t| matches!(t, Token::AnnotationMarker { .. }))
        .expect("Should find annotation marker");

    match first_annotation {
        Token::AnnotationMarker { span, .. } => {
            assert_eq!(
                span.start.column, 5,
                "Annotation after 'café ' should start at column 5"
            );
        }
        _ => unreachable!(),
    }

    // Definition markers
    let input = "café ::";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    let definition = tokens
        .iter()
        .find(|t| matches!(t, Token::DefinitionMarker { .. }))
        .expect("Should find definition marker");

    match definition {
        Token::DefinitionMarker { span, .. } => {
            assert_eq!(
                span.start.column, 5,
                "Definition after 'café ' should start at column 5"
            );
        }
        _ => unreachable!(),
    }
}

/// Test inline delimiters with Unicode
#[test]
fn test_inline_delimiters_unicode() {
    let test_cases = vec![
        ("café *bold*", '*', 5, "bold after accented"),
        ("café _italic_", '_', 5, "italic after accented"),
        ("café `code`", '`', 5, "code after accented"),
        ("café #math#", '#', 5, "math after accented"),
        ("🎉 *bold*", '*', 2, "bold after emoji"),
        ("→ _italic_", '_', 2, "italic after arrow"),
    ];

    for (input, delimiter_char, expected_col, description) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        let delimiter = tokens
            .iter()
            .find(|t| is_delimiter_token(t, delimiter_char))
            .unwrap_or_else(|| {
                panic!(
                    "Should find {} delimiter in: {}",
                    delimiter_char, description
                )
            });

        let span = get_token_span(delimiter);
        assert_eq!(
            span.start.column, expected_col,
            "{}: delimiter should start at column {} but was {}",
            description, expected_col, span.start.column
        );
    }
}

/// Test reference markers with Unicode
#[test]
fn test_reference_markers_unicode() {
    // Test that reference markers and @ signs in text are properly handled with Unicode
    // Note: @ is no longer a special delimiter outside of brackets
    // [ref] patterns are tokenized as RefMarker tokens

    #[allow(dead_code)]
    struct TestCase {
        input: &'static str,
        description: &'static str,
        #[allow(clippy::type_complexity)]
        check: Box<dyn Fn(&[Token])>,
    }

    let test_cases = vec![
        TestCase {
            input: "café [ref]",
            description: "ref marker after accented",
            check: Box::new(|tokens| {
                let ref_marker = tokens
                    .iter()
                    .find(|t| matches!(t, Token::RefMarker { .. }))
                    .expect("Should find RefMarker token");
                match ref_marker {
                    Token::RefMarker { span, .. } => {
                        assert_eq!(
                            span.start.column, 5,
                            "RefMarker should start at column 5 after 'café '"
                        );
                    }
                    _ => unreachable!(),
                }
            }),
        },
        TestCase {
            input: "café @cite",
            description: "@ in text after accented",
            check: Box::new(|tokens| {
                // @ is no longer special, so "@cite" should be part of a text token
                let text_with_at = tokens
                    .iter()
                    .find(|t| matches!(t, Token::Text { content, .. } if content.contains("@cite")))
                    .expect("Should find text token containing @cite");
                match text_with_at {
                    Token::Text { content, span } => {
                        assert!(content.contains("@cite"), "Text should contain @cite");
                        assert_eq!(span.start.column, 5, "Text with @ should start at column 5");
                    }
                    _ => unreachable!(),
                }
            }),
        },
        TestCase {
            input: "🎉 [ref]",
            description: "ref marker after emoji",
            check: Box::new(|tokens| {
                let ref_marker = tokens
                    .iter()
                    .find(|t| matches!(t, Token::RefMarker { .. }))
                    .expect("Should find RefMarker token");
                match ref_marker {
                    Token::RefMarker { span, .. } => {
                        assert_eq!(
                            span.start.column, 2,
                            "RefMarker should start at column 2 after emoji and space"
                        );
                    }
                    _ => unreachable!(),
                }
            }),
        },
        TestCase {
            input: "→ @cite",
            description: "@ in text after arrow",
            check: Box::new(|tokens| {
                let text_with_at = tokens
                    .iter()
                    .find(|t| matches!(t, Token::Text { content, .. } if content.contains("@cite")))
                    .expect("Should find text token containing @cite");
                match text_with_at {
                    Token::Text { span, .. } => {
                        assert_eq!(span.start.column, 2, "Text with @ should start at column 2");
                    }
                    _ => unreachable!(),
                }
            }),
        },
    ];

    for test_case in test_cases {
        let mut lexer = Lexer::new(test_case.input);
        let tokens = lexer.tokenize();
        (test_case.check)(&tokens);
    }
}

/// Test parameter spans with Unicode
#[test]
fn test_parameter_spans_unicode() {
    // Test that parameters with Unicode characters have correct positions
    // based on character count, not byte count

    let test_cases = vec![
        // Label with accent (café is a Text token, not part of Parameter)
        (
            ":: café:key=value ::",
            "café",
            true, // is_text
            3,
            7,
            "text label with accented characters",
        ),
        // Parameter with accented key
        (
            ":: label:café=value ::",
            "café",
            false, // is_text
            9,
            19, // The whole parameter token span, not just the key
            "parameter with accented key",
        ),
        // Parameter with accented value
        (
            ":: label:key=café ::",
            "café",
            false, // is_text
            9,
            17, // The whole parameter token span
            "parameter with accented value",
        ),
    ];

    for (input, unicode_content, is_text, expected_start, expected_end, description) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Find the token containing the Unicode content
        let token = if is_text {
            tokens
                .iter()
                .find(|t| matches!(t, Token::Text { content, .. } if content == unicode_content))
                .unwrap_or_else(|| {
                    panic!(
                        "Should find Text token with '{}' in: {}",
                        unicode_content, description
                    )
                })
        } else {
            tokens
                .iter()
                .find(|t| match t {
                    Token::Parameter { key, value, .. } => {
                        key == unicode_content || value == unicode_content
                    }
                    _ => false,
                })
                .unwrap_or_else(|| {
                    panic!(
                        "Should find Parameter token with '{}' in: {}",
                        unicode_content, description
                    )
                })
        };

        let span = get_token_span(token);
        assert_eq!(
            span.start.column, expected_start,
            "{}: token should start at column {} but was {}",
            description, expected_start, span.start.column
        );
        assert_eq!(
            span.end.column, expected_end,
            "{}: token should end at column {} but was {}",
            description, expected_end, span.end.column
        );
    }

    // Test emoji label separately - emoji in labels becomes text
    let emoji_input = ":: 🎉:key=value ::";
    let mut lexer = Lexer::new(emoji_input);
    let tokens = lexer.tokenize();

    let emoji_token = tokens
        .iter()
        .find(|t| matches!(t, Token::Text { content, .. } if content == "🎉"))
        .expect("Should find emoji text token");

    match emoji_token {
        Token::Text { span, .. } => {
            assert_eq!(span.start.column, 3, "Emoji should start at column 3");
            assert_eq!(span.end.column, 4, "Emoji should end at column 4");
        }
        _ => unreachable!(),
    }
}

/// Test mixed Unicode scenarios
#[test]
fn test_mixed_unicode_content() {
    let input = "🎉 café → résumé";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();

    // Verify each text token has correct position
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t, Token::Text { .. }))
        .collect();

    assert!(text_tokens.len() >= 3, "Should have at least 3 text tokens");

    // Check positions of each word
    let expected_positions = vec![
        ("café", 2),   // After "🎉 "
        ("→", 7),      // After "café "
        ("résumé", 9), // After "→ "
    ];

    for (expected_text, expected_col) in expected_positions {
        let token = text_tokens
            .iter()
            .find(|t| token_contains_text(t, expected_text))
            .unwrap_or_else(|| panic!("Should find text token '{}'", expected_text));

        let span = get_token_span(token);
        assert_eq!(
            span.start.column, expected_col,
            "'{}' should start at column {} but was {}",
            expected_text, expected_col, span.start.column
        );
    }
}

// Helper functions

fn get_token_span(token: &Token) -> &txxt::ast::tokens::SourceSpan {
    match token {
        Token::Text { span, .. }
        | Token::Identifier { span, .. }
        | Token::AnnotationMarker { span, .. }
        | Token::DefinitionMarker { span, .. }
        | Token::SequenceMarker { span, .. }
        | Token::BoldDelimiter { span }
        | Token::ItalicDelimiter { span }
        | Token::CodeDelimiter { span }
        | Token::MathDelimiter { span }
        | Token::LeftBracket { span }
        | Token::RightBracket { span }
        | Token::AtSign { span }
        | Token::LeftParen { span }
        | Token::RightParen { span }
        | Token::Colon { span }
        | Token::Dash { span }
        | Token::Period { span }
        | Token::Newline { span }
        | Token::BlankLine { span, .. }
        | Token::Whitespace { span, .. }
        | Token::Parameter { span, .. }
        | Token::RefMarker { span, .. }
        | Token::CitationRef { span, .. }
        | Token::PageRef { span, .. }
        | Token::SessionRef { span, .. }
        | Token::FootnoteRef { span, .. }
        | Token::Indent { span, .. }
        | Token::Dedent { span, .. }
        | Token::VerbatimTitle { span, .. }
        | Token::VerbatimLabel { span, .. }
        | Token::VerbatimContent { span, .. }
        | Token::Eof { span } => span,
    }
}

fn is_delimiter_token(token: &Token, delimiter: char) -> bool {
    matches!(
        (token, delimiter),
        (Token::BoldDelimiter { .. }, '*')
            | (Token::ItalicDelimiter { .. }, '_')
            | (Token::CodeDelimiter { .. }, '`')
            | (Token::MathDelimiter { .. }, '#')
    )
}

fn token_contains_text(token: &Token, text: &str) -> bool {
    match token {
        Token::Text { content, .. } => content.contains(text),
        Token::Identifier { content, .. } => content.contains(text),
        Token::Parameter { key, value, .. } => key.contains(text) || value.contains(text),
        _ => false,
    }
}
