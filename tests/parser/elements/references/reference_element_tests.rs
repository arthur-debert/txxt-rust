//! Reference Elements Parser Tests
//!
//! These tests validate the parsing of reference inline elements
//! (citations, footnotes, sessions, files, URLs) using the TxxtCorpora framework.

use txxt::parser::elements::inlines::references::*;
use txxt::ast::elements::references::reference_types::*;
use txxt::ast::elements::tokens::{Token, SourceSpan, Position};

/// Helper function to create a test source span
fn test_span() -> SourceSpan {
    SourceSpan {
        start: Position { row: 0, column: 0 },
        end: Position { row: 0, column: 1 },
    }
}

/// Helper function to create bracket tokens around content
fn create_bracketed_tokens(content: &str) -> Vec<Token> {
    let mut tokens = vec![
        Token::Text {
            content: "[".to_string(),
            span: test_span(),
        },
    ];
    
    tokens.push(Token::Text {
        content: content.to_string(),
        span: test_span(),
    });
    
    tokens.push(Token::Text {
        content: "]".to_string(),
        span: test_span(),
    });
    
    tokens
}

/// Test citation reference parsing
#[test]
fn test_parse_citation_simple() {
    let tokens = create_bracketed_tokens("@smith2023");
    
    let result = parse_citation(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Citation { citations, raw, .. } => {
                assert_eq!(citations.len(), 1);
                assert_eq!(citations[0].key, "smith2023");
                assert_eq!(citations[0].locator, None);
                assert_eq!(raw, "[@smith2023]");
            }
            _ => panic!("Expected Citation reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test citation with locator
#[test]
fn test_parse_citation_with_locator() {
    let tokens = create_bracketed_tokens("@smith2023, p. 123");
    
    let result = parse_citation(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Citation { citations, .. } => {
                assert_eq!(citations.len(), 1);
                assert_eq!(citations[0].key, "smith2023");
                assert_eq!(citations[0].locator, Some("p. 123".to_string()));
            }
            _ => panic!("Expected Citation reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test multiple citations
#[test]
fn test_parse_multiple_citations() {
    let tokens = create_bracketed_tokens("@smith2023; @jones2025");
    
    let result = parse_citation(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Citation { citations, .. } => {
                assert_eq!(citations.len(), 2);
                assert_eq!(citations[0].key, "smith2023");
                assert_eq!(citations[1].key, "jones2025");
            }
            _ => panic!("Expected Citation reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test naked footnote reference
#[test]
fn test_parse_footnote_naked() {
    let tokens = create_bracketed_tokens("1");
    
    let result = parse_footnote_ref(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::NakedNumerical { number, raw, .. } => {
                assert_eq!(*number, 1);
                assert_eq!(raw, "[1]");
            }
            _ => panic!("Expected NakedNumerical reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test labeled footnote reference
#[test]
fn test_parse_footnote_labeled() {
    let tokens = create_bracketed_tokens("^note-label");
    
    let result = parse_footnote_ref(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::NamedAnchor { anchor, raw, .. } => {
                assert_eq!(anchor, "note-label");
                assert_eq!(raw, "[^note-label]");
            }
            _ => panic!("Expected NamedAnchor reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test session reference numeric
#[test]
fn test_parse_session_ref_numeric() {
    let tokens = create_bracketed_tokens("#3");
    
    let result = parse_session_ref(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Section { identifier, raw, .. } => {
                match identifier {
                    SectionIdentifier::Numeric { levels, negative_index } => {
                        assert_eq!(levels, &vec![3]);
                        assert_eq!(*negative_index, false);
                    }
                    _ => panic!("Expected Numeric section identifier"),
                }
                assert_eq!(raw, "[#3]");
            }
            _ => panic!("Expected Section reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test session reference hierarchical
#[test]
fn test_parse_session_ref_hierarchical() {
    let tokens = create_bracketed_tokens("#2.1.3");
    
    let result = parse_session_ref(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Section { identifier, .. } => {
                match identifier {
                    SectionIdentifier::Numeric { levels, negative_index } => {
                        assert_eq!(levels, &vec![2, 1, 3]);
                        assert_eq!(*negative_index, false);
                    }
                    _ => panic!("Expected Numeric section identifier"),
                }
            }
            _ => panic!("Expected Section reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test session reference negative indexing
#[test]
fn test_parse_session_ref_negative() {
    let tokens = create_bracketed_tokens("#-1.2");
    
    let result = parse_session_ref(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Section { identifier, .. } => {
                match identifier {
                    SectionIdentifier::Numeric { levels, negative_index } => {
                        assert_eq!(levels, &vec![1, 2]);
                        assert_eq!(*negative_index, true);
                    }
                    _ => panic!("Expected Numeric section identifier"),
                }
            }
            _ => panic!("Expected Section reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test session reference named
#[test]
fn test_parse_session_ref_named() {
    let tokens = create_bracketed_tokens("introduction");
    
    let result = parse_session_ref(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Section { identifier, .. } => {
                match identifier {
                    SectionIdentifier::Named { name } => {
                        assert_eq!(name, "introduction");
                    }
                    _ => panic!("Expected Named section identifier"),
                }
            }
            _ => panic!("Expected Section reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test reference classifier
#[test]
fn test_reference_classification() {
    let classifier = ReferenceClassifier::new();
    
    // Test URL classification
    assert_eq!(classifier.classify("https://example.com"), SimpleReferenceType::Url);
    assert_eq!(classifier.classify("example.com"), SimpleReferenceType::Url);
    assert_eq!(classifier.classify("user@domain.com"), SimpleReferenceType::Url);
    
    // Test citation classification
    assert_eq!(classifier.classify("@smith2023"), SimpleReferenceType::Citation);
    assert_eq!(classifier.classify("@smith2023, p. 45"), SimpleReferenceType::Citation);
    
    // Test section classification
    assert_eq!(classifier.classify("#3"), SimpleReferenceType::Section);
    assert_eq!(classifier.classify("#2.1"), SimpleReferenceType::Section);
    assert_eq!(classifier.classify("#-1.2"), SimpleReferenceType::Section);
    
    // Test footnote classification
    assert_eq!(classifier.classify("1"), SimpleReferenceType::Footnote);
    assert_eq!(classifier.classify("42"), SimpleReferenceType::Footnote);
    
    // Test TK classification
    assert_eq!(classifier.classify("TK"), SimpleReferenceType::ToComeTK);
    assert_eq!(classifier.classify("tk"), SimpleReferenceType::ToComeTK);
    assert_eq!(classifier.classify("TK-1"), SimpleReferenceType::ToComeTK);
    assert_eq!(classifier.classify("TK-someword"), SimpleReferenceType::ToComeTK);
    
    // Test file classification
    assert_eq!(classifier.classify("./file.txt"), SimpleReferenceType::File);
    assert_eq!(classifier.classify("../dir/file.txt"), SimpleReferenceType::File);
    assert_eq!(classifier.classify("/absolute/path"), SimpleReferenceType::File);
    
    // Test not sure fallback
    assert_eq!(classifier.classify("some-content"), SimpleReferenceType::NotSure);
}

/// Test general reference parser
#[test]
fn test_parse_reference_url() {
    let tokens = create_bracketed_tokens("https://example.com");
    
    let result = parse_reference(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::Url { url, fragment, raw, .. } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(fragment, &None);
                assert_eq!(raw, "[https://example.com]");
            }
            _ => panic!("Expected Url reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test file reference parsing
#[test]
fn test_parse_reference_file() {
    let tokens = create_bracketed_tokens("./docs/guide.txxt");
    
    let result = parse_reference(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::File { path, section, raw, .. } => {
                assert_eq!(path, "./docs/guide.txxt");
                assert_eq!(section, &None);
                assert_eq!(raw, "[./docs/guide.txxt]");
            }
            _ => panic!("Expected File reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test file reference with section
#[test]
fn test_parse_reference_file_with_section() {
    let tokens = create_bracketed_tokens("./docs/guide.txxt#introduction");
    
    let result = parse_reference(&tokens);
    assert!(result.is_ok());
    
    if let Ok(txxt::ast::elements::formatting::inlines::Inline::Reference(reference)) = result {
        match &reference.target {
            ReferenceTarget::File { path, section, .. } => {
                assert_eq!(path, "./docs/guide.txxt");
                assert_eq!(section, &Some("introduction".to_string()));
            }
            _ => panic!("Expected File reference target"),
        }
    } else {
        panic!("Expected Reference inline");
    }
}

/// Test error handling for empty content
#[test]
fn test_reference_empty_content_error() {
    let empty_tokens: Vec<Token> = vec![];
    
    assert!(parse_citation(&empty_tokens).is_err());
    assert!(parse_footnote_ref(&empty_tokens).is_err());
    assert!(parse_session_ref(&empty_tokens).is_err());
    assert!(parse_reference(&empty_tokens).is_err());
}

/// Test error handling for invalid citation format
#[test]
fn test_citation_invalid_format_error() {
    let tokens = create_bracketed_tokens("invalid-citation");
    
    let result = parse_citation(&tokens);
    assert!(result.is_err());
}

/// Test error handling for invalid footnote format
#[test]
fn test_footnote_invalid_format_error() {
    let tokens = create_bracketed_tokens("not-a-number");
    
    let result = parse_footnote_ref(&tokens);
    assert!(result.is_err());
}

// TODO: Add tests using TxxtCorpora when test cases are defined in specs
// These would load test cases from docs/specs/elements/references/

/// Placeholder test for future TxxtCorpora integration
#[test]
#[ignore] // Ignore until test corpus is defined
fn test_references_with_corpora() {
    // This test would use TxxtCorpora to load test cases from the specification
    // Example:
    // let corpus = TxxtCorpora::load("txxt.core.spec.references.citation.simple").unwrap();
    // let result = parse_citation_from_text(&corpus.source_text);
    // assert!(result.is_ok());
}