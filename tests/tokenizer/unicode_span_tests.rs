//! Comprehensive Unicode span tests for all tokenizers
//!
//! These tests ensure that ALL tokenizers correctly handle multi-byte Unicode
//! characters when calculating spans. This is critical for language server
//! functionality and any position-based features.

use txxt::ast::tokens::Token;
use txxt::tokenizer::Lexer;

/// Test that all tokenizers handle emoji (4-byte characters) correctly
#[test]
#[ignore = "Emoji are now recognized as text, but column position tracking needs Unicode grapheme cluster support"]
fn test_all_tokenizers_with_emoji() {
    let test_cases = vec![
        // Format: (input, description, expected_tokens_after_emoji)
        (
            "🎉text",
            "text after emoji",
            vec!["text should start at column 1"],
        ),
        (
            "🎉- item",
            "sequence marker after emoji",
            vec!["dash should start at column 1"],
        ),
        (
            "🎉:: label ::",
            "annotation after emoji",
            vec![":: should start at column 1"],
        ),
        (
            "🎉term ::",
            "definition after emoji",
            vec![": should start at column 1"],
        ),
        (
            "🎉*bold*",
            "bold delimiter after emoji",
            vec!["* should start at column 1"],
        ),
        (
            "🎉_italic_",
            "italic delimiter after emoji",
            vec!["_ should start at column 1"],
        ),
        (
            "🎉`code`",
            "code delimiter after emoji",
            vec!["` should start at column 1"],
        ),
        (
            "🎉#math#",
            "math delimiter after emoji",
            vec!["# should start at column 1"],
        ),
        (
            "🎉[ref]",
            "reference after emoji",
            vec!["[ should start at column 1"],
        ),
        (
            "🎉@citation",
            "citation after emoji",
            vec!["@ should start at column 1"],
        ),
        (
            "🎉(@p.42)",
            "page ref after emoji",
            vec!["( should start at column 1"],
        ),
        (
            "🎉[^footnote]",
            "footnote after emoji",
            vec!["[ should start at column 1"],
        ),
    ];

    for (input, description, _expectations) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Find first non-whitespace, non-emoji token
        let first_token_after_emoji = tokens
            .iter()
            .find(|t| !matches!(t, Token::Eof { .. }) && token_starts_after_position(t, 0))
            .unwrap_or_else(|| panic!("Should find token after emoji in: {}", description));

        assert_eq!(
            get_token_start_column(first_token_after_emoji),
            1,
            "{}: First token after emoji should start at column 1, but starts at column {}",
            description,
            get_token_start_column(first_token_after_emoji)
        );
    }
}

/// Test sequence markers with various Unicode scenarios
#[test]
#[ignore = "Unicode character position calculation incorrect"]
fn test_sequence_markers_unicode() {
    let test_cases = vec![
        ("café- item", 4, 5, "accented letter before dash"),
        ("→- item", 1, 2, "arrow before dash"),
        ("🎉- item", 1, 2, "emoji before dash"),
        ("- café", 0, 1, "accented letter in item"),
        ("42. café", 0, 3, "numerical marker with accented item"),
        ("à. item", 0, 2, "accented letter as alphabetical marker"),
    ];

    for (input, expected_start, expected_end, description) in test_cases {
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
#[ignore = "Unicode position calculation affects reference marker detection"]
fn test_reference_markers_unicode() {
    let test_cases = vec![
        ("café [ref]", '[', 5, "ref after accented"),
        ("café @cite", '@', 5, "citation after accented"),
        ("🎉 [ref]", '[', 2, "ref after emoji"),
        ("→ @cite", '@', 2, "citation after arrow"),
    ];

    for (input, expected_char, expected_col, description) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Find the bracket or at-sign
        let token = tokens
            .iter()
            .find(|t| {
                matches!(t, Token::LeftBracket { .. }) && expected_char == '['
                    || matches!(t, Token::AtSign { .. }) && expected_char == '@'
            })
            .unwrap_or_else(|| panic!("Should find {} in: {}", expected_char, description));

        let span = get_token_span(token);
        assert_eq!(
            span.start.column, expected_col,
            "{}: {} should start at column {} but was {}",
            description, expected_char, expected_col, span.start.column
        );
    }
}

/// Test parameter spans with Unicode
#[test]
#[ignore = "Unicode in labels not properly handled"]
fn test_parameter_spans_unicode() {
    let test_cases = vec![
        (
            ":: café:key=value ::",
            "café",
            3,
            7,
            "parameter with accented label",
        ),
        (
            ":: 🎉:key=value ::",
            "🎉",
            3,
            4,
            "parameter with emoji label",
        ),
        (
            ":: label:café=value ::",
            "café",
            9,
            13,
            "parameter with accented key",
        ),
        (
            ":: label:key=café ::",
            "café",
            13,
            17,
            "parameter with accented value",
        ),
    ];

    for (input, unicode_part, expected_start, expected_end, description) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();

        // Find token containing the Unicode part
        let token = tokens
            .iter()
            .find(|t| token_contains_text(t, unicode_part))
            .unwrap_or_else(|| {
                panic!(
                    "Should find token with '{}' in: {}",
                    unicode_part, description
                )
            });

        let span = get_token_span(token);
        assert_eq!(
            span.start.column, expected_start,
            "{}: '{}' should start at column {} but was {}",
            description, unicode_part, expected_start, span.start.column
        );
        assert_eq!(
            span.end.column, expected_end,
            "{}: '{}' should end at column {} but was {}",
            description, unicode_part, expected_end, span.end.column
        );
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

fn get_token_start_column(token: &Token) -> usize {
    get_token_span(token).start.column
}

fn token_starts_after_position(token: &Token, position: usize) -> bool {
    get_token_start_column(token) > position
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
