//! Integration tests for page reference tokenization with main lexer

use txxt::ast::scanner_tokens::ScannerToken;
use txxt::lexer::tokenize;

#[test]
fn test_page_ref_integration_simple() {
    let tokens = tokenize("[p.123]");

    // Should have: PageRef, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        ScannerToken::PageRef { content, span } => {
            assert_eq!(content, "123");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 7);
        }
        _ => panic!("Expected PageRef token, got {:?}", tokens[0]),
    }

    match &tokens[1] {
        ScannerToken::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[test]
fn test_page_ref_integration_range() {
    let tokens = tokenize("[p.123-125]");

    // Should have: PageRef, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        ScannerToken::PageRef { content, span } => {
            assert_eq!(content, "123-125");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 11);
        }
        _ => panic!("Expected PageRef token, got {:?}", tokens[0]),
    }
}

#[test]
fn test_page_ref_integration_with_text() {
    let tokens = tokenize("See page [p.42] for details.");

    // Should have: Text, PageRef, Text, Eof
    assert!(tokens.len() >= 4);

    // Find the page reference
    let page_ref = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::PageRef { .. }))
        .expect("Should find PageRef token");

    match page_ref {
        ScannerToken::PageRef { content, .. } => {
            assert_eq!(content, "42");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_page_ref_vs_ref_marker() {
    let tokens = tokenize("[p.123]");

    // Should produce PageRef, not general RefMarker
    let page_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::PageRef { .. }))
        .collect();

    let ref_markers: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::RefMarker { .. }))
        .collect();

    assert_eq!(page_refs.len(), 1, "Should have exactly one PageRef token");
    assert_eq!(ref_markers.len(), 0, "Should not have RefMarker tokens");
}

#[test]
fn test_page_ref_single_digit() {
    let tokens = tokenize("[p.5]");

    let page_ref = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::PageRef { .. }))
        .expect("Should find PageRef token");

    match page_ref {
        ScannerToken::PageRef { content, .. } => {
            assert_eq!(content, "5");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_page_ref_large_numbers() {
    let tokens = tokenize("[p.999]");

    let page_ref = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::PageRef { .. }))
        .expect("Should find PageRef token");

    match page_ref {
        ScannerToken::PageRef { content, .. } => {
            assert_eq!(content, "999");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_page_ref_adjacent() {
    let tokens = tokenize("[p.10][p.20]");

    // Should parse as: PageRef("10"), PageRef("20"), Eof
    let page_refs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::PageRef { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(page_refs, vec!["10", "20"]);
}

#[test]
fn test_page_ref_range_variations() {
    let test_cases = vec![
        ("[p.1-2]", "1-2"),
        ("[p.10-15]", "10-15"),
        ("[p.100-200]", "100-200"),
    ];

    for (input, expected_content) in test_cases {
        let tokens = tokenize(input);

        let page_ref = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::PageRef { .. }))
            .unwrap_or_else(|| panic!("Should find PageRef token in {}", input));

        match page_ref {
            ScannerToken::PageRef { content, .. } => {
                assert_eq!(content, expected_content, "Failed for input: {}", input);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_incomplete_page_ref_fallback() {
    let tokens = tokenize("[p.incomplete");

    // Should not produce PageRef due to missing closing bracket
    let has_page_ref = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::PageRef { .. }));

    assert!(
        !has_page_ref,
        "Incomplete page ref should not produce PageRef token"
    );

    // Since [p.incomplete is not a valid reference marker (missing ]), it should be treated as text
    let has_text = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::Text { .. } | ScannerToken::Identifier { .. }));

    assert!(
        has_text,
        "Incomplete page ref should produce text/identifier tokens"
    );
}

#[test]
fn test_page_ref_with_invalid_chars() {
    let tokens = tokenize("[p.12a3]");

    // Should not produce PageRef due to invalid characters
    let has_page_ref = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::PageRef { .. }));

    assert!(
        !has_page_ref,
        "Page ref with invalid chars should not produce PageRef token"
    );
}

#[test]
fn test_page_ref_mixed_content() {
    let tokens = tokenize("Check [p.15] and [p.20-25] for references.");

    // Should have multiple page references
    let page_refs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::PageRef { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(page_refs, vec!["15", "20-25"]);
}

#[test]
fn test_page_ref_missing_dot() {
    let tokens = tokenize("[p123]");

    // Should not produce PageRef without dot
    let has_page_ref = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::PageRef { .. }));

    assert!(
        !has_page_ref,
        "Page ref without dot should not produce PageRef token"
    );

    // Should produce RefMarker instead
    let has_ref_marker = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::RefMarker { .. }));

    assert!(has_ref_marker, "Should produce RefMarker for [p123]");
}
