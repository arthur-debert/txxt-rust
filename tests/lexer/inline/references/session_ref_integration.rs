//! Integration tests for session reference tokenization with main lexer

use txxt::cst::ScannerToken;
use txxt::lexer::tokenize;

#[test]
fn test_session_ref_integration_simple() {
    let tokens = tokenize("[#1]");

    // Should have: SessionRef, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        ScannerToken::SessionRef { content, span } => {
            assert_eq!(content, "1");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 4);
        }
        _ => panic!("Expected SessionRef token, got {:?}", tokens[0]),
    }

    match &tokens[1] {
        ScannerToken::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[test]
fn test_session_ref_integration_hierarchical() {
    let tokens = tokenize("[#1.2]");

    // Should have: SessionRef, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        ScannerToken::SessionRef { content, span } => {
            assert_eq!(content, "1.2");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 6);
        }
        _ => panic!("Expected SessionRef token, got {:?}", tokens[0]),
    }
}

#[test]
fn test_session_ref_integration_deep_hierarchical() {
    let tokens = tokenize("[#1.2.3]");

    // Should have: SessionRef, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        ScannerToken::SessionRef { content, span } => {
            assert_eq!(content, "1.2.3");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 8);
        }
        _ => panic!("Expected SessionRef token, got {:?}", tokens[0]),
    }
}

#[test]
fn test_session_ref_integration_with_text() {
    let tokens = tokenize("See section [#1.2] for details.");

    // Should have: Text, SessionRef, Text, Eof
    assert!(tokens.len() >= 4);

    // Find the session reference
    let session_ref = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::SessionRef { .. }))
        .expect("Should find SessionRef token");

    match session_ref {
        ScannerToken::SessionRef { content, .. } => {
            assert_eq!(content, "1.2");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_session_ref_vs_ref_marker() {
    let tokens = tokenize("[#1.2]");

    // Should produce SessionRef, not general RefMarker
    let session_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::SessionRef { .. }))
        .collect();

    let ref_markers: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, ScannerToken::RefMarker { .. }))
        .collect();

    assert_eq!(
        session_refs.len(),
        1,
        "Should have exactly one SessionRef token"
    );
    assert_eq!(ref_markers.len(), 0, "Should not have RefMarker tokens");
}

#[test]
fn test_session_ref_multi_digit() {
    let tokens = tokenize("[#10.20.30]");

    let session_ref = tokens
        .iter()
        .find(|token| matches!(token, ScannerToken::SessionRef { .. }))
        .expect("Should find SessionRef token");

    match session_ref {
        ScannerToken::SessionRef { content, .. } => {
            assert_eq!(content, "10.20.30");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_session_ref_adjacent() {
    let tokens = tokenize("[#1][#2.1]");

    // Should parse as: SessionRef("1"), SessionRef("2.1"), Eof
    let session_refs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::SessionRef { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(session_refs, vec!["1", "2.1"]);
}

#[test]
fn test_session_ref_variations() {
    let test_cases = vec![
        ("[#1]", "1"),
        ("[#2.1]", "2.1"),
        ("[#3.2.1]", "3.2.1"),
        ("[#10]", "10"),
        ("[#100.200]", "100.200"),
    ];

    for (input, expected_content) in test_cases {
        let tokens = tokenize(input);

        let session_ref = tokens
            .iter()
            .find(|token| matches!(token, ScannerToken::SessionRef { .. }))
            .unwrap_or_else(|| panic!("Should find SessionRef token in {}", input));

        match session_ref {
            ScannerToken::SessionRef { content, .. } => {
                assert_eq!(content, expected_content, "Failed for input: {}", input);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_incomplete_session_ref_fallback() {
    let tokens = tokenize("[#1.incomplete");

    // Should not produce SessionRef due to missing closing bracket
    let has_session_ref = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::SessionRef { .. }));

    assert!(
        !has_session_ref,
        "Incomplete session ref should not produce SessionRef token"
    );

    // Since [#1.incomplete is not a valid reference marker (missing ]), it should be treated as text
    let has_text = tokens.iter().any(|token| {
        matches!(
            token,
            ScannerToken::Text { .. } | ScannerToken::Identifier { .. }
        )
    });

    assert!(
        has_text,
        "Incomplete session ref should produce text/identifier tokens"
    );
}

#[test]
fn test_session_ref_with_invalid_chars() {
    let tokens = tokenize("[#1.a]");

    // Should not produce SessionRef due to invalid characters
    let has_session_ref = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::SessionRef { .. }));

    assert!(
        !has_session_ref,
        "Session ref with invalid chars should not produce SessionRef token"
    );
}

#[test]
fn test_session_ref_mixed_content() {
    let tokens = tokenize("Check [#1] and [#2.3] for references.");

    // Should have multiple session references
    let session_refs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            ScannerToken::SessionRef { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(session_refs, vec!["1", "2.3"]);
}

#[test]
fn test_session_ref_empty() {
    let tokens = tokenize("[#]");

    // Should not produce SessionRef without content
    let has_session_ref = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::SessionRef { .. }));

    assert!(
        !has_session_ref,
        "Empty session ref should not produce SessionRef token"
    );

    // Should produce RefMarker instead (empty session ref falls back to RefMarker)
    let has_ref_marker = tokens
        .iter()
        .any(|token| matches!(token, ScannerToken::RefMarker { .. }));

    if !has_ref_marker {
        // Check if it falls back to text/identifier tokens instead
        let has_text = tokens.iter().any(|token| {
            matches!(
                token,
                ScannerToken::Text { .. } | ScannerToken::Identifier { .. }
            )
        });

        // Also check if it produces a MathDelimiter (the # character)
        let has_math_delimiter = tokens
            .iter()
            .any(|token| matches!(token, ScannerToken::MathDelimiter { .. }));

        assert!(
            has_text || has_math_delimiter,
            "Should produce either RefMarker, text tokens, or MathDelimiter for [#]"
        );
    }
}

#[test]
fn test_session_ref_invalid_patterns() {
    let invalid_patterns = vec![
        "[#.1]",    // starts with dot
        "[#1.]",    // ends with dot
        "[#1..2]",  // double dot
        "[#1.a.2]", // contains letter
        "[#a]",     // starts with letter
    ];

    for pattern in invalid_patterns {
        let tokens = tokenize(pattern);

        let has_session_ref = tokens
            .iter()
            .any(|token| matches!(token, ScannerToken::SessionRef { .. }));

        assert!(
            !has_session_ref,
            "Invalid pattern {} should not produce SessionRef token",
            pattern
        );
    }
}
