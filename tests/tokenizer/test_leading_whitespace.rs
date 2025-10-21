use txxt::cst::HighLevelToken;
/// Test for issue #116: Leading whitespace should not become standalone TextSpan tokens
///
/// This test verifies that leading whitespace after Indent tokens is properly handled
/// by storing it in the indentation_chars field of line tokens, not as standalone TextSpan tokens.
use txxt::syntax::tokenize;
use txxt::syntax::SemanticAnalyzer;

#[test]
fn test_indented_paragraph_no_standalone_whitespace() {
    let source = "    This is indented.\n";

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(scanner_tokens).unwrap();

    // Expected high-level tokens:
    // 0: Indent
    // 1: PlainTextLine with indentation_chars="    " and content="This is indented."
    // 2: Dedent

    assert_eq!(
        result.tokens.len(),
        3,
        "Should have exactly 3 tokens: Indent, PlainTextLine, Dedent"
    );

    // Token 0: Indent
    assert!(
        matches!(result.tokens[0], HighLevelToken::Indent { .. }),
        "First token should be Indent, got {:?}",
        result.tokens[0]
    );

    // Token 1: PlainTextLine with indentation_chars
    match &result.tokens[1] {
        HighLevelToken::PlainTextLine {
            indentation_chars,
            content,
            ..
        } => {
            assert_eq!(
                indentation_chars, "    ",
                "PlainTextLine should have indentation_chars='    '"
            );

            // Content should be the text without leading whitespace
            match content.as_ref() {
                HighLevelToken::TextSpan { content: text, .. } => {
                    assert!(
                        text.starts_with("This"),
                        "Content should start with 'This', got: '{}'",
                        text
                    );
                }
                _ => panic!(
                    "PlainTextLine content should be TextSpan, got {:?}",
                    content
                ),
            }
        }
        _ => panic!(
            "Second token should be PlainTextLine, got {:?}",
            result.tokens[1]
        ),
    }

    // Token 2: Dedent
    assert!(
        matches!(result.tokens[2], HighLevelToken::Dedent { .. }),
        "Third token should be Dedent, got {:?}",
        result.tokens[2]
    );
}

#[test]
fn test_indented_list_no_standalone_whitespace() {
    let source = "1. Session\n\n    - First item\n    - Second item\n";

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(scanner_tokens).unwrap();

    // Expected high-level tokens:
    // 0: SequenceTextLine ("1. Session")
    // 1: BlankLine
    // 2: Indent
    // 3: SequenceTextLine with indentation_chars="    " ("- First item")
    // 4: SequenceTextLine with indentation_chars="    " ("- Second item")
    // 5: Dedent

    assert_eq!(
        result.tokens.len(),
        6,
        "Should have exactly 6 tokens, got {} tokens",
        result.tokens.len()
    );

    // Verify no standalone TextSpan tokens exist
    for (i, token) in result.tokens.iter().enumerate() {
        if let HighLevelToken::TextSpan { .. } = token {
            panic!(
                "Token {} should not be standalone TextSpan, got {:?}",
                i, token
            );
        }
    }

    // Token 2: Indent
    assert!(
        matches!(result.tokens[2], HighLevelToken::Indent { .. }),
        "Token 2 should be Indent"
    );

    // Token 3: First list item with indentation_chars
    match &result.tokens[3] {
        HighLevelToken::SequenceTextLine {
            indentation_chars, ..
        } => {
            assert_eq!(
                indentation_chars, "    ",
                "First list item should have indentation_chars='    '"
            );
        }
        _ => panic!(
            "Token 3 should be SequenceTextLine, got {:?}",
            result.tokens[3]
        ),
    }

    // Token 4: Second list item with indentation_chars
    match &result.tokens[4] {
        HighLevelToken::SequenceTextLine {
            indentation_chars, ..
        } => {
            assert_eq!(
                indentation_chars, "    ",
                "Second list item should have indentation_chars='    '"
            );
        }
        _ => panic!(
            "Token 4 should be SequenceTextLine, got {:?}",
            result.tokens[4]
        ),
    }
}

#[test]
fn test_multiple_indented_paragraphs_no_standalone_whitespace() {
    let source = "1. Session\n\n    First para.\n\n    Second para.\n";

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(scanner_tokens).unwrap();

    // Expected high-level tokens:
    // 0: SequenceTextLine ("1. Session")
    // 1: BlankLine
    // 2: Indent
    // 3: PlainTextLine with indentation_chars="    " ("First para.")
    // 4: BlankLine
    // 5: PlainTextLine with indentation_chars="    " ("Second para.")
    // 6: Dedent

    assert_eq!(
        result.tokens.len(),
        7,
        "Should have exactly 7 tokens, got {} tokens",
        result.tokens.len()
    );

    // Verify no standalone TextSpan tokens
    for (i, token) in result.tokens.iter().enumerate() {
        if let HighLevelToken::TextSpan { .. } = token {
            panic!(
                "Token {} should not be standalone TextSpan, got {:?}",
                i, token
            );
        }
    }

    // Token 3: First paragraph with indentation_chars
    match &result.tokens[3] {
        HighLevelToken::PlainTextLine {
            indentation_chars, ..
        } => {
            assert_eq!(
                indentation_chars, "    ",
                "First paragraph should have indentation_chars='    '"
            );
        }
        _ => panic!(
            "Token 3 should be PlainTextLine, got {:?}",
            result.tokens[3]
        ),
    }

    // Token 5: Second paragraph with indentation_chars
    match &result.tokens[5] {
        HighLevelToken::PlainTextLine {
            indentation_chars, ..
        } => {
            assert_eq!(
                indentation_chars, "    ",
                "Second paragraph should have indentation_chars='    '"
            );
        }
        _ => panic!(
            "Token 5 should be PlainTextLine, got {:?}",
            result.tokens[5]
        ),
    }
}

#[test]
fn test_top_level_paragraph_no_indentation_chars() {
    let source = "This is top level.\n";

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(scanner_tokens).unwrap();

    // Expected: PlainTextLine with empty indentation_chars
    assert_eq!(result.tokens.len(), 1, "Should have exactly 1 token");

    match &result.tokens[0] {
        HighLevelToken::PlainTextLine {
            indentation_chars,
            content,
            ..
        } => {
            assert_eq!(
                indentation_chars, "",
                "Top-level paragraph should have empty indentation_chars"
            );

            match content.as_ref() {
                HighLevelToken::TextSpan { content: text, .. } => {
                    assert!(
                        text.starts_with("This"),
                        "Content should start with 'This', got: '{}'",
                        text
                    );
                }
                _ => panic!("PlainTextLine content should be TextSpan"),
            }
        }
        _ => panic!("Should be PlainTextLine, got {:?}", result.tokens[0]),
    }
}
