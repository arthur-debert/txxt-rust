//! Debug test for complex verbatim detection issue
//!
//! This test isolates the verbatim scanner to show why the complex document
//! structure test is failing.

use rstest::rstest;
use txxt::cst::ScannerToken;
use txxt::lexer::tokenize;

#[test]
fn test_complex_verbatim_isolation() {
    // This is the exact verbatim block from the failing test
    let input = r#"    Python example:
        def hello():
            print("Hello from txxt!")
    :: python ::"#;

    println!("Input text:");
    println!("{}", input);
    println!();

    println!("Input with visible characters:");
    for (i, line) in input.lines().enumerate() {
        println!("Line {}: '{}' (len: {})", i, line, line.len());
        // Show indentation
        let indent = line.len() - line.trim_start().len();
        println!("  Indent: {} spaces", indent);
    }
    println!();

    let tokens = tokenize(input);

    println!("All tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
    println!();

    // Check if we found a VerbatimTitle
    let verbatim_title = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimTitle { .. }));

    match verbatim_title {
        Some(ScannerToken::VerbatimTitle { content, span }) => {
            println!("✅ Found VerbatimTitle: '{}'", content);
            println!("   Span: {:?}", span);
        }
        None => {
            println!("❌ No VerbatimTitle found!");

            // Check what we got instead
            let text_tokens: Vec<_> = tokens
                .iter()
                .filter(|token| matches!(token, ScannerToken::Text { .. }))
                .collect();

            if !text_tokens.is_empty() {
                println!("   Found {} Text tokens instead:", text_tokens.len());
                for token in text_tokens {
                    if let ScannerToken::Text { content, span } = token {
                        println!("     Text: '{}' at {:?}", content, span);
                    }
                }
            }
        }
    }

    // Check if we found verbatim content tokens
    let verbatim_content = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::IgnoreTextSpan { .. }));

    match verbatim_content {
        Some(ScannerToken::IgnoreTextSpan { content, span }) => {
            println!("✅ Found IgnoreTextSpan: '{}'", content);
            println!("   Span: {:?}", span);
        }
        None => {
            println!("❌ No IgnoreTextSpan found!");
        }
    }

    // Check if we found an IndentationWall
    let indentation_wall = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::IndentationWall { .. }));

    match indentation_wall {
        Some(ScannerToken::IndentationWall {
            level,
            wall_type,
            span,
        }) => {
            println!(
                "✅ Found IndentationWall: level={}, wall_type={:?}",
                level, wall_type
            );
            println!("   Span: {:?}", span);
        }
        None => {
            println!("❌ No IndentationWall found!");
        }
    }

    // The test should fail if no verbatim block was detected
    assert!(
        verbatim_title.is_some(),
        "Expected to find a VerbatimTitle token"
    );
    assert!(
        verbatim_content.is_some(),
        "Expected to find an IgnoreTextSpan token"
    );
    assert!(
        indentation_wall.is_some(),
        "Expected to find an IndentationWall token"
    );
}

#[test]
fn test_simple_verbatim_for_comparison() {
    // This is a simple verbatim block that should work
    let input = r#"simple title:
    content line
:: label ::"#;

    println!("Simple input text:");
    println!("{}", input);
    println!();

    println!("Simple input with visible characters:");
    for (i, line) in input.lines().enumerate() {
        println!("Line {}: '{}' (len: {})", i, line, line.len());
        let indent = line.len() - line.trim_start().len();
        println!("  Indent: {} spaces", indent);
    }
    println!();

    let tokens = tokenize(input);

    println!("Simple tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("Token {}: {:?}", i, token);
    }
    println!();

    // This should work
    let verbatim_title = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::VerbatimTitle { .. }));

    assert!(verbatim_title.is_some(), "Simple verbatim should work");

    if let Some(ScannerToken::VerbatimTitle { content, .. }) = verbatim_title {
        println!("✅ Simple verbatim title: '{}'", content);
    }
}
