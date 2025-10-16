//! Tests for escape sequence handling in text tokens
//!
//! These tests verify that the tokenizer correctly handles backslash escape
//! sequences for special characters that would otherwise be tokenized as
//! formatting delimiters or structural elements.

use txxt::lexer::{tokenize, Token};

#[test]
fn test_escaped_asterisk() {
    let input = r"This is \*not bold\* text";
    let tokens = tokenize(input);

    // Should tokenize with escaped asterisks in text tokens
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // The escaped asterisks should be in the text tokens
    let all_text = text_tokens.join("");
    assert!(all_text.contains(r"\*not") || all_text.contains(r"bold\*"));

    // Should not have any BoldDelimiter tokens
    let bold_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::BoldDelimiter { .. }))
        .count();
    assert_eq!(bold_count, 0);
}

#[test]
fn test_escaped_underscore() {
    let input = r"This has \_no emphasis\_ here";
    let tokens = tokenize(input);

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // The escaped underscores should be in the text tokens
    let all_text = text_tokens.join("");
    assert!(all_text.contains(r"\_no") || all_text.contains(r"emphasis\_"));

    // Should not have any ItalicDelimiter tokens
    let italic_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::ItalicDelimiter { .. }))
        .count();
    assert_eq!(italic_count, 0);
}

#[test]
fn test_escaped_backtick() {
    let input = r"This has \`no code\` formatting";
    let tokens = tokenize(input);

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // The escaped backticks should be in the text tokens
    let all_text = text_tokens.join("");
    assert!(all_text.contains(r"\`no") || all_text.contains(r"code\`"));

    // Should not have any CodeDelimiter tokens
    let code_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::CodeDelimiter { .. }))
        .count();
    assert_eq!(code_count, 0);
}

#[test]
fn test_escaped_hash() {
    let input = r"This has \#no math\# expression";
    let tokens = tokenize(input);

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // The escaped hashes should be in the text tokens
    let all_text = text_tokens.join("");
    assert!(all_text.contains(r"\#no") || all_text.contains(r"math\#"));

    // Should not have any MathDelimiter tokens
    let math_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::MathDelimiter { .. }))
        .count();
    assert_eq!(math_count, 0);
}

#[test]
fn test_escaped_dash() {
    let input = r"\- This is not a list item";
    let tokens = tokenize(input);

    // Should not create a SequenceMarker token
    let has_sequence_marker = tokens
        .iter()
        .any(|t| matches!(t, Token::SequenceMarker { .. }));
    assert!(
        !has_sequence_marker,
        "Should not have sequence marker for escaped dash"
    );

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert!(text_tokens.iter().any(|&text| text.contains(r"\-")));
}

#[test]
fn test_escaped_backslash() {
    let input = r"This has a literal \\ backslash";
    let tokens = tokenize(input);

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // Should contain the escaped backslash
    assert!(text_tokens.iter().any(|&text| text.contains(r"\\")));
}

#[test]
fn test_escaped_brackets() {
    let input = r"This has \[no reference\] here";
    let tokens = tokenize(input);

    // Should not create bracket tokens
    let has_brackets = tokens
        .iter()
        .any(|t| matches!(t, Token::LeftBracket { .. } | Token::RightBracket { .. }));
    assert!(
        !has_brackets,
        "Should not have bracket tokens for escaped brackets"
    );

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // The escaped brackets should be in the text tokens
    let all_text = text_tokens.join("");
    assert!(all_text.contains(r"\[no") || all_text.contains(r"reference\]"));
}

#[test]
fn test_multiple_escapes_in_text() {
    let input = r"Mix of \*bold\*, \_italic\_, and \`code\` escapes";
    let tokens = tokenize(input);

    let full_text: String = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");

    assert!(full_text.contains(r"\*bold\*"));
    assert!(full_text.contains(r"\_italic\_"));
    assert!(full_text.contains(r"\`code\`"));
}

#[test]
fn test_escape_at_start_of_line() {
    let input = r"\*Starting with escaped asterisk";
    let tokens = tokenize(input);

    // First text token should contain the escaped asterisk
    let first_text = tokens
        .iter()
        .find_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .unwrap();

    assert!(first_text.starts_with(r"\*"));
}

#[test]
fn test_unescaped_special_chars_still_work() {
    let input = "This has *real bold* text";
    let tokens = tokenize(input);

    // Should have BoldDelimiter tokens
    let bold_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::BoldDelimiter { .. }))
        .count();

    assert_eq!(
        bold_count, 2,
        "Should have two bold delimiters for unescaped asterisks"
    );
}

#[test]
fn test_partial_escape_sequences() {
    let input = r"Only first \*asterisk is escaped* here";
    let tokens = tokenize(input);

    // Should have one text token with escaped asterisk and one BoldDelimiter
    let has_escaped_text = tokens.iter().any(|t| match t {
        Token::Text { content, .. } => content.contains(r"\*"),
        _ => false,
    });

    let bold_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::BoldDelimiter { .. }))
        .count();

    assert!(has_escaped_text, "Should have text with escaped asterisk");
    assert_eq!(
        bold_count, 1,
        "Should have one bold delimiter for unescaped asterisk"
    );
}

#[test]
fn test_backslash_followed_by_non_special() {
    let input = r"This has \a regular backslash before 'a'";
    let tokens = tokenize(input);

    let text_tokens: Vec<_> = tokens
        .iter()
        .filter_map(|t| match t {
            Token::Text { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    // Backslash before non-special character should be preserved as-is
    assert!(text_tokens.iter().any(|&text| text.contains(r"\a")));
}

#[test]
fn test_escape_sequences_in_inline_code() {
    // Within backtick code spans, escapes should not be processed
    let input = r"In code: `\*not escaped\*` see?";
    let tokens = tokenize(input);

    // Should have CodeDelimiter tokens
    let code_delim_count = tokens
        .iter()
        .filter(|t| matches!(t, Token::CodeDelimiter { .. }))
        .count();

    assert_eq!(code_delim_count, 2, "Should have code delimiters");

    // The text between code delimiters should contain the backslashes literally
    let mut in_code = false;
    let mut code_content = String::new();
    for token in &tokens {
        match token {
            Token::CodeDelimiter { .. } => {
                in_code = !in_code;
                if !in_code && !code_content.is_empty() {
                    // Check the accumulated code content
                    assert!(
                        code_content.contains(r"\*not") || code_content.contains(r"escaped\*"),
                        "Code content should contain escaped sequences: {}",
                        code_content
                    );
                }
            }
            Token::Text { content, .. } if in_code => {
                code_content.push_str(content);
            }
            _ => {}
        }
    }
}
