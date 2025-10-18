//! Tests for semantic tokens infrastructure
//!
//! This module tests the semantic tokens implementation including:
//! - SemanticToken enum variants
//! - SemanticTokenList functionality
//! - Conversion traits between scanner and semantic tokens
//! - Builder pattern usage
//! - Serialization/deserialization

use txxt::ast::{
    scanner_tokens::{Position, ScannerToken, SourceSpan},
    semantic_tokens::{
        FromScannerToken, SemanticNumberingForm, SemanticNumberingStyle, SemanticToken,
        SemanticTokenBuilder, SemanticTokenList, ToScannerToken,
    },
};

#[test]
fn test_semantic_token_list_creation() {
    let list = SemanticTokenList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn test_semantic_token_list_with_tokens() {
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };
    let token = SemanticToken::Label {
        text: "test".to_string(),
        span: span.clone(),
    };
    let list = SemanticTokenList::with_tokens(vec![token.clone()]);

    assert!(!list.is_empty());
    assert_eq!(list.len(), 1);
    assert_eq!(list.source_span, span);
}

#[test]
fn test_semantic_token_list_push() {
    let mut list = SemanticTokenList::new();
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };
    let token = SemanticToken::Label {
        text: "test".to_string(),
        span,
    };

    list.push(token);
    assert!(!list.is_empty());
    assert_eq!(list.len(), 1);
}

#[test]
fn test_from_scanner_token_conversion() {
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };
    let scanner_token = ScannerToken::BlankLine {
        whitespace: "".to_string(),
        span: span.clone(),
    };

    let semantic_token = SemanticToken::from_scanner_token(&scanner_token);
    assert!(semantic_token.is_some());

    let semantic_token = semantic_token.unwrap();
    match semantic_token {
        SemanticToken::BlankLine { span: token_span } => {
            assert_eq!(token_span, span);
        }
        _ => panic!("Expected BlankLine"),
    }
}

#[test]
fn test_to_scanner_token_conversion() {
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };
    let semantic_token = SemanticToken::BlankLine { span: span.clone() };

    let scanner_tokens = semantic_token.to_scanner_tokens();
    assert_eq!(scanner_tokens.len(), 1);

    match &scanner_tokens[0] {
        ScannerToken::BlankLine {
            span: token_span, ..
        } => {
            assert_eq!(token_span, &span);
        }
        _ => panic!("Expected BlankLine"),
    }
}

#[test]
fn test_semantic_token_builder() {
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };

    let label = SemanticTokenBuilder::label("test".to_string(), span.clone());
    match label {
        SemanticToken::Label {
            text,
            span: token_span,
        } => {
            assert_eq!(text, "test");
            assert_eq!(token_span, span);
        }
        _ => panic!("Expected Label"),
    }
}

#[test]
fn test_semantic_numbering_styles() {
    assert_eq!(
        SemanticNumberingStyle::Numeric,
        SemanticNumberingStyle::Numeric
    );
    assert_eq!(
        SemanticNumberingStyle::Alphabetic,
        SemanticNumberingStyle::Alphabetic
    );
    assert_eq!(SemanticNumberingStyle::Roman, SemanticNumberingStyle::Roman);
    assert_eq!(SemanticNumberingStyle::Plain, SemanticNumberingStyle::Plain);
}

#[test]
fn test_semantic_numbering_forms() {
    assert_eq!(
        SemanticNumberingForm::Regular,
        SemanticNumberingForm::Regular
    );
    assert_eq!(
        SemanticNumberingForm::Extended,
        SemanticNumberingForm::Extended
    );
}

#[test]
fn test_serialization() {
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };
    let token = SemanticToken::Label {
        text: "test".to_string(),
        span,
    };

    let serialized = serde_json::to_string(&token).unwrap();
    let deserialized: SemanticToken = serde_json::from_str(&serialized).unwrap();

    assert_eq!(token, deserialized);
}

#[test]
fn test_all_semantic_token_variants() {
    let span = SourceSpan {
        start: Position { row: 1, column: 1 },
        end: Position { row: 1, column: 2 },
    };

    // Test all semantic token variants can be created
    let _label = SemanticToken::Label {
        text: "test".to_string(),
        span: span.clone(),
    };
    let _parameters = SemanticToken::Parameters {
        params: std::collections::HashMap::new(),
        span: span.clone(),
    };
    let _sequence_marker = SemanticToken::SequenceMarker {
        style: SemanticNumberingStyle::Numeric,
        form: SemanticNumberingForm::Regular,
        marker: "1.".to_string(),
        span: span.clone(),
    };
    let _text_span = SemanticToken::TextSpan {
        content: "hello".to_string(),
        span: span.clone(),
    };
    let _plain_text_line = SemanticToken::PlainTextLine {
        content: Box::new(SemanticToken::TextSpan {
            content: "hello".to_string(),
            span: span.clone(),
        }),
        span: span.clone(),
    };
    let _ignore_line = SemanticToken::IgnoreLine {
        content: "raw content".to_string(),
        span: span.clone(),
    };
    let _blank_line = SemanticToken::BlankLine { span: span.clone() };
    let _indent = SemanticToken::Indent { span: span.clone() };
    let _dedent = SemanticToken::Dedent { span };

    // If we get here, all variants can be created successfully
    // This test verifies that all semantic token variants can be instantiated
    // without compilation errors, which is the main purpose of this test
}
