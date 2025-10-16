/// # Paragraph Parser Tests
///
/// Tests for the paragraph parser implementation using corpora examples
/// and ensemble documents.
#[path = "corpora.rs"]
mod corpora;

use corpora::TxxtCorpora;
use txxt::parser::elements::paragraph::paragraph::parse_paragraph;

/// Test simple paragraph from corpora
#[test]
fn test_corpora_simple_paragraph() {
    let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")
        .expect("Corpus should exist in specs");

    // Verify we got the right content
    assert!(!corpus.source_text.is_empty(), "Corpus should have content");
    assert!(
        corpus.source_text.contains("paragraph"),
        "Should be about paragraphs"
    );

    // Tokenize the content
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Parse the paragraph
    let result = parse_paragraph(&tokens);
    assert!(result.is_ok(), "Simple paragraph should parse successfully");

    let paragraph = result.unwrap();
    assert_eq!(paragraph.content.len(), 1);
    assert_eq!(paragraph.annotations.len(), 0);
    assert_eq!(paragraph.parameters.map.len(), 0);

    // Check text content
    if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) = &paragraph.content[0]
    {
        assert!(text_span.tokens.text().contains("paragraph"));
    } else {
        panic!("Expected identity text transform");
    }
}

/// Test multiline paragraph from corpora
#[test]
fn test_corpora_multiline_paragraph() {
    let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.multiline")
        .expect("Corpus should exist in specs");

    // Verify we got the right content
    assert!(!corpus.source_text.is_empty(), "Corpus should have content");
    assert!(
        corpus.source_text.contains("line"),
        "Should be about multiline content"
    );

    // Tokenize the content
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Parse the paragraph
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Multiline paragraph should parse successfully"
    );

    let paragraph = result.unwrap();
    assert_eq!(paragraph.content.len(), 1);

    // Check text content contains multiline text
    if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) = &paragraph.content[0]
    {
        let text = text_span.tokens.text();
        assert!(text.contains("line"));
        assert!(text.contains("paragraph"));
    } else {
        panic!("Expected identity text transform");
    }
}

/// Test consistent indent paragraph from corpora
#[test]
fn test_corpora_consistent_indent_paragraph() {
    let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.consistent-indent")
        .expect("Corpus should exist in specs");

    // Verify we got the right content
    assert!(!corpus.source_text.is_empty(), "Corpus should have content");

    // Tokenize the content
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Parse the paragraph
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Consistent indent paragraph should parse successfully"
    );

    let paragraph = result.unwrap();
    assert_eq!(paragraph.content.len(), 1);

    // Check text content
    if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) = &paragraph.content[0]
    {
        let text = text_span.tokens.text();
        assert!(text.contains("paragraph"));
        assert!(text.contains("indentation"));
    } else {
        panic!("Expected identity text transform");
    }
}

// ============================================================================
// Ensemble Document Tests
// ============================================================================

/// Test single paragraph ensemble document
#[test]
fn test_ensemble_single_paragraph() {
    let corpus =
        TxxtCorpora::load_document("01-two-paragraphs").expect("Ensemble document should exist");

    // Verify we got the right content
    assert!(
        !corpus.source_text.is_empty(),
        "Document should have content"
    );
    assert!(
        corpus.source_text.contains("paragraph"),
        "Should contain paragraph content"
    );

    // Tokenize the content
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Parse the paragraph
    let result = parse_paragraph(&tokens);
    assert!(
        result.is_ok(),
        "Single paragraph document should parse successfully"
    );

    let paragraph = result.unwrap();
    assert_eq!(paragraph.content.len(), 1);

    // Check text content
    if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) = &paragraph.content[0]
    {
        let text = text_span.tokens.text();
        assert!(text.contains("paragraph"));
        assert!(text.contains("simple"));
    } else {
        panic!("Expected identity text transform");
    }
}

/// Test multiple paragraphs ensemble document
#[test]
fn test_ensemble_multiple_paragraphs() {
    let corpus =
        TxxtCorpora::load_document("01-two-paragraphs").expect("Ensemble document should exist");

    // Verify we got the right content
    assert!(
        !corpus.source_text.is_empty(),
        "Document should have content"
    );
    assert!(
        corpus.source_text.contains("first paragraph"),
        "Should contain first paragraph"
    );
    assert!(
        corpus.source_text.contains("second paragraph"),
        "Should contain second paragraph"
    );

    // Tokenize the content
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // For multiple paragraphs, we need to handle them separately
    // This is a simplified test - in a real implementation, we'd parse the full document
    // and extract individual paragraphs

    // Parse the first paragraph (up to first blank line)
    let first_paragraph_tokens: Vec<_> = tokens
        .iter()
        .take_while(|token| !matches!(token, txxt::ast::tokens::Token::BlankLine { .. }))
        .cloned()
        .collect();

    if !first_paragraph_tokens.is_empty() {
        let result = parse_paragraph(&first_paragraph_tokens);
        assert!(result.is_ok(), "First paragraph should parse successfully");

        let paragraph = result.unwrap();
        if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) =
            &paragraph.content[0]
        {
            let text = text_span.tokens.text();
            assert!(text.contains("first paragraph"));
        } else {
            panic!("Expected identity text transform");
        }
    }
}

/// Test paragraph parsing with BlockParser integration
#[test]
fn test_block_parser_integration() {
    use txxt::lexer::pipeline::TokenTreeBuilder;
    use txxt::parser::pipeline::BlockParser;

    let corpus = TxxtCorpora::load("txxt.core.spec.paragraph.valid.simple")
        .expect("Corpus should exist in specs");

    // Tokenize the content
    let tokens = txxt::lexer::tokenize(&corpus.source_text);
    assert!(!tokens.is_empty(), "Should have tokens");

    // Build token tree
    let token_tree_builder = TokenTreeBuilder::new();
    let token_tree = token_tree_builder
        .build_tree(tokens)
        .expect("Should build token tree successfully");

    // Parse with BlockParser
    let block_parser = BlockParser::new();
    let result = block_parser.parse_blocks(token_tree);
    assert!(result.is_ok(), "BlockParser should parse successfully");

    let elements = result.unwrap();
    assert!(!elements.is_empty(), "Should have parsed elements");

    // Check that we got a paragraph element
    if let txxt::ast::ElementNode::ParagraphBlock(paragraph) = &elements[0] {
        assert_eq!(paragraph.content.len(), 1);
        if let txxt::ast::elements::inlines::TextTransform::Identity(text_span) =
            &paragraph.content[0]
        {
            let text = text_span.tokens.text();
            // Debug: print the actual text to see what we got
            println!("Parsed text: '{}'", text);
            assert!(
                text.contains("paragraph") || text.contains("basic"),
                "Text should contain 'paragraph' or 'basic', but got: '{}'",
                text
            );
        } else {
            panic!("Expected identity text transform");
        }
    } else {
        panic!("Expected ParagraphBlock element");
    }
}
