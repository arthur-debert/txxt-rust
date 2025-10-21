use txxt::cst::{HighLevelToken, ScannerToken};
use txxt::syntax::tokenize;
use txxt::syntax::SemanticAnalyzer;

fn scanner_summary(token: &ScannerToken) -> String {
    match token {
        ScannerToken::Text { content, .. } => format!("Text(\"{}\")", content),
        ScannerToken::Whitespace { content, .. } => {
            format!("Whitespace(\"{}\")", content.replace(" ", "·"))
        }
        ScannerToken::Newline { .. } => "Newline".to_string(),
        ScannerToken::Indent { .. } => "Indent".to_string(),
        ScannerToken::Dedent { .. } => "Dedent".to_string(),
        ScannerToken::BlankLine { .. } => "BlankLine".to_string(),
        ScannerToken::SequenceMarker { marker_type, .. } => {
            format!("SequenceMarker({})", marker_type.content())
        }
        _ => format!("{:?}", token).chars().take(30).collect(),
    }
}

fn hl_summary(token: &HighLevelToken) -> String {
    match token {
        HighLevelToken::PlainTextLine { .. } => "PlainTextLine".to_string(),
        HighLevelToken::SequenceTextLine { .. } => "SequenceTextLine".to_string(),
        HighLevelToken::BlankLine { .. } => "BlankLine".to_string(),
        HighLevelToken::Indent { .. } => "Indent".to_string(),
        HighLevelToken::Dedent { .. } => "Dedent".to_string(),
        HighLevelToken::TextSpan { content, .. } => {
            format!("TextSpan(\"{}\")", content.replace(" ", "·"))
        }
        _ => format!("{:?}", token).chars().take(40).collect(),
    }
}

fn test_scenario(name: &str, source: &str) {
    println!("\n{}", "=".repeat(60));
    println!("SCENARIO: {}", name);
    println!("{}", "=".repeat(60));
    println!("Source:\n{}", source.replace(" ", "·").replace("\n", "↵\n"));

    let scanner_tokens = tokenize(source);
    println!("\nScanner tokens:");
    for (i, token) in scanner_tokens.iter().enumerate() {
        println!("  {}: {}", i, scanner_summary(token));
    }

    let analyzer = SemanticAnalyzer::new();
    let hl_tokens = analyzer.analyze(scanner_tokens).unwrap();
    println!("\nHigh-level tokens:");
    for (i, token) in hl_tokens.tokens.iter().enumerate() {
        println!("  {}: {}", i, hl_summary(token));
    }
}

fn main() {
    // Scenario 1: Top-level paragraph
    test_scenario("Top-level paragraph", "This is a paragraph.\n");

    // Scenario 2: Indented paragraph (in a session)
    test_scenario("Indented paragraph", "    This is indented.\n");

    // Scenario 3: Session with paragraph
    test_scenario(
        "Session with paragraph",
        "1. Session title\n\n    This is content.\n",
    );

    // Scenario 4: Top-level list
    test_scenario("Top-level list", "- First item\n- Second item\n");

    // Scenario 5: Indented list (in session)
    test_scenario(
        "Indented list in session",
        "1. Session\n\n    - First item\n    - Second item\n",
    );

    // Scenario 6: Multiple indented paragraphs
    test_scenario(
        "Multiple indented paragraphs",
        "1. Session\n\n    First para.\n\n    Second para.\n",
    );
}
