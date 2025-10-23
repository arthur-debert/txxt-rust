//! Tests for verbatim scanner false positive issue #31
use txxt::syntax::ScannerToken;

use txxt::syntax::tokenize;

#[test]
fn test_colon_followed_by_annotation_not_verbatim() {
    let input = r#"Annotations can have parameters:
:: warning:severity=high :: Critical security information"#;

    let tokens = tokenize(input);

    // Should not have any verbatim tokens
    let has_verbatim = tokens.iter().any(|t| {
        matches!(
            t,
            ScannerToken::VerbatimBlockStart { .. }
                | ScannerToken::VerbatimContentLine { .. }
                | ScannerToken::VerbatimBlockEnd { .. }
        )
    });

    assert!(!has_verbatim, "Should not tokenize as verbatim block");

    // Should have annotation marker tokens
    let has_annotation = tokens
        .iter()
        .any(|t| matches!(t, ScannerToken::TxxtMarker { .. }));

    assert!(has_annotation, "Should have annotation marker");
}

#[test]
fn test_multiple_colons_followed_by_annotation() {
    let input = r#"Some text ending with colon:
:: label :: annotation content"#;

    let tokens = tokenize(input);

    // Should not be verbatim
    let has_verbatim = tokens.iter().any(|t| {
        matches!(
            t,
            ScannerToken::VerbatimBlockStart { .. }
                | ScannerToken::VerbatimContentLine { .. }
                | ScannerToken::VerbatimBlockEnd { .. }
        )
    });

    assert!(!has_verbatim, "Should not tokenize as verbatim block");
}

#[test]
fn test_definition_followed_by_annotation() {
    // New syntax: single colon for definitions
    let input = r#"Some definition:
:: note :: This is an annotation, not verbatim content"#;

    let tokens = tokenize(input);

    // Should have definition marker (Colon)
    let has_colon = tokens
        .iter()
        .any(|t| matches!(t, ScannerToken::Colon { .. }));

    assert!(has_colon, "Should have colon marker for definition");

    // Should not be verbatim
    let has_verbatim = tokens.iter().any(|t| {
        matches!(
            t,
            ScannerToken::VerbatimBlockStart { .. }
                | ScannerToken::VerbatimContentLine { .. }
                | ScannerToken::VerbatimBlockEnd { .. }
        )
    });

    assert!(!has_verbatim, "Should not tokenize as verbatim block");
}

#[test]
fn test_real_verbatim_still_works() {
    let input = r#"Code example:
    def hello():
        print("Hello!")
:: python ::"#;

    let tokens = tokenize(input);

    // Should have verbatim tokens
    let has_verbatim = tokens.iter().any(|t| {
        matches!(
            t,
            ScannerToken::VerbatimBlockStart { .. }
                | ScannerToken::VerbatimContentLine { .. }
                | ScannerToken::VerbatimBlockEnd { .. }
        )
    });

    assert!(has_verbatim, "Should tokenize as verbatim block");
}

#[test]
fn test_colon_with_indented_annotation() {
    let input = r#"    Indented text with colon:
    :: warning :: Indented annotation"#;

    let tokens = tokenize(input);

    // Should not be verbatim
    let has_verbatim = tokens.iter().any(|t| {
        matches!(
            t,
            ScannerToken::VerbatimBlockStart { .. }
                | ScannerToken::VerbatimContentLine { .. }
                | ScannerToken::VerbatimBlockEnd { .. }
        )
    });

    assert!(!has_verbatim, "Should not tokenize as verbatim block");
}
