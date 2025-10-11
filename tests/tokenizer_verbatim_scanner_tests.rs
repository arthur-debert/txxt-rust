use txxt::tokenizer::{tokenize, TokenType, VerbatimScanner};

/// Test that verbatim blocks are correctly identified by the pre-scanner
#[test]
fn test_verbatim_scanner_identifies_blocks() {
    let text = r#"Regular paragraph.

Simple verbatim:
    content line 1
    content line 2
(python)

Another paragraph."#;

    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(text);

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].start_line, 3); // "Simple verbatim:"
    assert_eq!(blocks[0].end_line, 6); // "(python)"
    assert_eq!(blocks[0].start_indent, 0);
}

/// Test that verbatim content lines are correctly identified
#[test]
fn test_verbatim_scanner_identifies_content_lines() {
    let text = r#"Simple verbatim:
    content line 1
    content line 2
(python)"#;

    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(text);

    // Line 1 = title (not content)
    assert!(!scanner.is_verbatim_content(1, &blocks));

    // Lines 2-3 = content
    assert!(scanner.is_verbatim_content(2, &blocks));
    assert!(scanner.is_verbatim_content(3, &blocks));

    // Line 4 = label (not content)
    assert!(!scanner.is_verbatim_content(4, &blocks));
}

/// Test that definitions (ending with ::) are NOT treated as verbatim
#[test]
fn test_definitions_not_treated_as_verbatim() {
    let text = r#"Definition ::
    This is a definition content
    Not verbatim content
"#;

    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(text);

    // Should find no verbatim blocks
    assert_eq!(blocks.len(), 0);
}

/// CRITICAL TEST: Verbatim content lines should NOT have Indent/Dedent tokens
#[test]
fn test_verbatim_content_no_indent_tokens() {
    let text = r#"Simple verbatim:
    print("Hello World")
    return 42
(python)"#;

    let tokens = tokenize(text);

    // Find all Indent/Dedent tokens
    let indent_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.token_type, TokenType::Indent | TokenType::Dedent))
        .collect();

    // CRITICAL: Should have NO Indent/Dedent tokens because verbatim content
    // should not be processed for indentation structure
    assert_eq!(
        indent_tokens.len(),
        0,
        "Verbatim blocks should not contain Indent/Dedent tokens. Found: {:?}",
        indent_tokens
    );
}

/// Test that verbatim content is preserved exactly as written
#[test]
fn test_verbatim_content_preserved_exactly() {
    let text = r#"Code example:
    def hello():
        print("World")  # comment
    return 42
(python)"#;

    let tokens = tokenize(text);

    // Find verbatim content tokens
    let content_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::VerbatimContent)
        .collect();

    assert_eq!(content_tokens.len(), 3);
    assert_eq!(
        content_tokens[0].value,
        Some("    def hello():".to_string())
    );
    assert_eq!(
        content_tokens[1].value,
        Some("        print(\"World\")  # comment".to_string())
    );
    assert_eq!(content_tokens[2].value, Some("    return 42".to_string()));
}

/// Test that title and label are still processed as normal TXXT
#[test]
fn test_verbatim_title_and_label_processed_normally() {
    let text = r#"Simple verbatim:
    content here
(python)"#;

    let tokens = tokenize(text);

    // Should have normal TXXT tokens for title and label
    let text_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Text)
        .collect();

    let verbatim_start_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::VerbatimStart)
        .collect();

    let verbatim_end_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::VerbatimEnd)
        .collect();

    let identifier_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Identifier)
        .collect();

    // Should have title processed as Text + VerbatimStart
    assert_eq!(text_tokens.len(), 1);
    assert_eq!(text_tokens[0].value, Some("Simple verbatim".to_string()));
    assert_eq!(verbatim_start_tokens.len(), 1);

    // Should have label processed as VerbatimEnd + Identifier + VerbatimEnd
    assert_eq!(verbatim_end_tokens.len(), 2); // "(" and ")"
    assert_eq!(identifier_tokens.len(), 1);
    assert_eq!(identifier_tokens[0].value, Some("python".to_string()));
}

/// Test multiple verbatim blocks in same document
#[test]
fn test_multiple_verbatim_blocks() {
    let text = r#"First verbatim:
    content 1
(label1)

Second verbatim:
    content 2
(label2)"#;

    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(text);

    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].start_line, 1);
    assert_eq!(blocks[0].end_line, 3);
    assert_eq!(blocks[1].start_line, 5);
    assert_eq!(blocks[1].end_line, 7);
}

/// Test indented verbatim block (inside other structure)
#[test]
fn test_indented_verbatim_block() {
    let text = r#"Some content ::

    Indented verbatim:
        verbatim content here
        more content
    (label)"#;

    let scanner = VerbatimScanner::new();
    let blocks = scanner.scan(text);

    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].start_line, 3); // "    Indented verbatim:"
    assert_eq!(blocks[0].end_line, 6); // "    (label)"
    assert_eq!(blocks[0].start_indent, 4); // Indented at level 4
}
