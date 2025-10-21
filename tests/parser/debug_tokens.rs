use txxt::syntax::tokenize;
use txxt::syntax::SemanticAnalyzer;

#[test]
fn debug_verbatim_01_tokens() {
    let source = "Code example:\n    def hello():\n        print(\"Hello, world!\")\n:: python\n";

    println!("=== Source ===\n{}", source);

    // Phase 1.b: Scanner tokens
    let scanner_tokens = tokenize(source);
    println!("\n=== Scanner Tokens ===");
    for (i, token) in scanner_tokens.iter().enumerate() {
        println!("{}: {:?}", i, scanner_token_summary(token));
    }

    // Phase 1.c: High-level tokens
    let analyzer = SemanticAnalyzer::new();
    let high_level_tokens = analyzer.analyze(scanner_tokens);

    println!("\n=== High Level Tokens ===");
    let high_level_tokens = high_level_tokens.unwrap();
    for (i, token) in high_level_tokens.tokens.iter().enumerate() {
        println!("{}: {}", i, token_summary(token));
    }

    // Check we got exactly 1 verbatim token
    let verbatim_count = high_level_tokens
        .tokens
        .iter()
        .filter(|t| matches!(t, txxt::cst::HighLevelToken::VerbatimBlock { .. }))
        .count();

    println!("\nVerbatim blocks found: {}", verbatim_count);
    assert_eq!(verbatim_count, 1, "Expected 1 verbatim block");
}

#[test]
fn debug_simple_list_tokens() {
    let source = "- First item\n- Second item\n- Third item\n";

    println!("=== Source ===\n{}", source);

    let scanner_tokens = tokenize(source);
    let analyzer = SemanticAnalyzer::new();
    let high_level_tokens = analyzer.analyze(scanner_tokens);

    println!("\n=== High Level Tokens ===");
    let high_level_tokens = high_level_tokens.unwrap();
    for (i, token) in high_level_tokens.tokens.iter().enumerate() {
        println!("{}: {}", i, token_summary(token));
    }
}

#[test]
fn debug_session_with_list_tokens() {
    let source = "1. Key Features\n\n    This is intro.\n    \n    - First feature\n    - Second feature\n    \n    This is outro.\n";

    println!("=== Source ===\n{}", source);

    // Phase 1.b: Scanner tokens
    let scanner_tokens = tokenize(source);
    println!("\n=== Scanner Tokens ===");
    for (i, token) in scanner_tokens.iter().enumerate() {
        println!("{}: {:?}", i, scanner_token_summary(token));
    }

    // Phase 1.c: High-level tokens
    let analyzer = SemanticAnalyzer::new();
    let high_level_tokens = analyzer.analyze(scanner_tokens);

    println!("\n=== High Level Tokens ===");
    let high_level_tokens = high_level_tokens.unwrap();
    for (i, token) in high_level_tokens.tokens.iter().enumerate() {
        println!("{}: {}", i, token_summary(token));
    }
}

fn scanner_token_summary(token: &txxt::cst::ScannerToken) -> String {
    use txxt::cst::ScannerToken;
    match token {
        ScannerToken::Text { content, .. } => {
            format!("Text(\"{}\")", content.chars().take(20).collect::<String>())
        }
        ScannerToken::Whitespace { content, .. } => format!(
            "Whitespace(\"{}\")",
            content
                .replace("\n", "\\n")
                .replace("\t", "\\t")
                .replace(" ", "·")
        ),
        ScannerToken::Newline { .. } => "Newline".to_string(),
        ScannerToken::Indent { .. } => "Indent".to_string(),
        ScannerToken::Dedent { .. } => "Dedent".to_string(),
        ScannerToken::BlankLine { .. } => "BlankLine".to_string(),
        ScannerToken::Dash { .. } => "Dash".to_string(),
        ScannerToken::Period { .. } => "Period".to_string(),
        ScannerToken::TxxtMarker { .. } => "TxxtMarker".to_string(),
        other => format!("{:?}", other).chars().take(40).collect(),
    }
}

fn token_summary(token: &txxt::cst::HighLevelToken) -> String {
    use txxt::cst::HighLevelToken;
    match token {
        HighLevelToken::PlainTextLine { .. } => "PlainTextLine".to_string(),
        HighLevelToken::SequenceTextLine { .. } => "SequenceTextLine".to_string(),
        HighLevelToken::BlankLine { .. } => "BlankLine".to_string(),
        HighLevelToken::Indent { .. } => "Indent".to_string(),
        HighLevelToken::Dedent { .. } => "Dedent".to_string(),
        HighLevelToken::Annotation { .. } => "Annotation".to_string(),
        HighLevelToken::Definition { .. } => "Definition".to_string(),
        HighLevelToken::VerbatimBlock { .. } => "Verbatim".to_string(),
        other => format!("{:?}", other).chars().take(60).collect(),
    }
}
