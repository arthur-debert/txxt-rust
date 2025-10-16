//! Integration tests for citation reference tokenization with main lexer

use txxt::ast::tokens::Token;
use txxt::lexer::tokenize;

#[test]
fn test_citation_ref_integration_simple() {
    let tokens = tokenize("[@smith2020]");

    // Should have: CitationRef, Eof
    assert_eq!(tokens.len(), 2);

    match &tokens[0] {
        Token::CitationRef { content, span } => {
            assert_eq!(content, "smith2020");
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, 12);
        }
        _ => panic!("Expected CitationRef token, got {:?}", tokens[0]),
    }

    match &tokens[1] {
        Token::Eof { .. } => {}
        _ => panic!("Expected Eof token, got {:?}", tokens[1]),
    }
}

#[test]
fn test_citation_ref_integration_with_text() {
    let tokens = tokenize("According to [@smith2020], the theory is correct.");

    // Should have: Text, CitationRef, Text, Eof
    assert!(tokens.len() >= 4);

    // Find the citation reference
    let citation_ref = tokens
        .iter()
        .find(|token| matches!(token, Token::CitationRef { .. }))
        .expect("Should find CitationRef token");

    match citation_ref {
        Token::CitationRef { content, .. } => {
            assert_eq!(content, "smith2020");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_citation_ref_vs_ref_marker() {
    let tokens = tokenize("[@citation]");

    // Should produce CitationRef, not general RefMarker
    let citation_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::CitationRef { .. }))
        .collect();

    let ref_markers: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::RefMarker { .. }))
        .collect();

    assert_eq!(
        citation_refs.len(),
        1,
        "Should have exactly one CitationRef token"
    );
    assert_eq!(ref_markers.len(), 0, "Should not have RefMarker tokens");
}

#[test]
fn test_citation_ref_complex_keys() {
    let tokens = tokenize("[@author:smith-jones.2020]");

    // Should parse complex citation key with namespace, hyphens, and dots
    let citation_ref = tokens
        .iter()
        .find(|token| matches!(token, Token::CitationRef { .. }))
        .expect("Should find CitationRef token");

    match citation_ref {
        Token::CitationRef { content, .. } => {
            assert_eq!(content, "author:smith-jones.2020");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_citation_ref_adjacent() {
    let tokens = tokenize("[@ref1][@ref2]");

    // Should parse as: CitationRef("ref1"), CitationRef("ref2"), Eof
    let citation_refs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::CitationRef { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(citation_refs, vec!["ref1", "ref2"]);
}

#[test]
fn test_incomplete_citation_ref_fallback() {
    let tokens = tokenize("[@incomplete");

    // Should not produce CitationRef due to missing closing bracket
    let has_citation_ref = tokens
        .iter()
        .any(|token| matches!(token, Token::CitationRef { .. }));

    assert!(
        !has_citation_ref,
        "Incomplete citation ref should not produce CitationRef token"
    );

    // Since [@incomplete is not a valid reference marker (missing ]), it should be treated as text
    // The tokenizer should parse this as individual text tokens or identifiers
    let has_text = tokens
        .iter()
        .any(|token| matches!(token, Token::Text { .. } | Token::Identifier { .. }));

    assert!(
        has_text,
        "Incomplete citation ref should produce text/identifier tokens"
    );
}

#[test]
fn test_citation_ref_with_invalid_chars() {
    let tokens = tokenize("[@invalid chars]");

    // Should not produce CitationRef due to spaces
    let has_citation_ref = tokens
        .iter()
        .any(|token| matches!(token, Token::CitationRef { .. }));

    assert!(
        !has_citation_ref,
        "Citation ref with invalid chars should not produce CitationRef token"
    );
}

#[test]
fn test_citation_ref_mixed_content() {
    let tokens = tokenize("See [@paper1] and [@paper2] for details.");

    // Should have multiple citations
    let citation_refs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::CitationRef { content, .. } => Some(content.as_str()),
            _ => None,
        })
        .collect();

    assert_eq!(citation_refs, vec!["paper1", "paper2"]);
}
