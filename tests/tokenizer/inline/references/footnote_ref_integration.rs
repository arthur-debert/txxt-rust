//! Integration tests for footnote reference tokenization

use rstest::rstest;
use txxt::ast::tokens::Token;
use txxt::tokenizer::inline::references::footnote_ref::FootnoteType;
use txxt::tokenizer::tokenize;

// =============================================================================
// FootnoteRef Token - Naked Numerical Format Tests
// =============================================================================

#[rstest]
#[case("[1]", 1)]
#[case("[2]", 2)]
#[case("[42]", 42)]
#[case("[123]", 123)]
#[case("[999]", 999)]
fn test_footnote_ref_naked_passing(#[case] input: &str, #[case] expected_number: u32) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // FootnoteRef + EOF

    match &tokens[0] {
        Token::FootnoteRef { footnote_type, span } => {
            assert_eq!(footnote_type, &FootnoteType::Naked(expected_number));
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected FootnoteRef token, got {:?}", tokens[0]),
    }
}

#[rstest]
#[case("[0]")] // Zero not allowed for footnotes
#[case("[01]")] // Leading zeros not supported  
#[case("[1a]")] // Mixed alphanumeric
#[case("[a1]")] // Mixed alphanumeric
#[case("[-1]")] // Negative numbers
fn test_footnote_ref_naked_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not produce FootnoteRef token
    let footnote_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::FootnoteRef { .. }))
        .collect();
    
    assert_eq!(footnote_refs.len(), 0, "Should not produce FootnoteRef for: {}", input);
}

// =============================================================================
// FootnoteRef Token - Labeled Format Tests  
// =============================================================================

#[rstest]
#[case("[^note1]", "note1")]
#[case("[^detailed-explanation]", "detailed-explanation")]
#[case("[^methodology_note]", "methodology_note")]
#[case("[^_private]", "_private")]
#[case("[^Note123]", "Note123")]
#[case("[^a]", "a")]
#[case("[^Z]", "Z")]
fn test_footnote_ref_labeled_passing(#[case] input: &str, #[case] expected_label: &str) {
    let tokens = tokenize(input);

    assert_eq!(tokens.len(), 2); // FootnoteRef + EOF

    match &tokens[0] {
        Token::FootnoteRef { footnote_type, span } => {
            assert_eq!(footnote_type, &FootnoteType::Labeled(expected_label.to_string()));
            assert_eq!(span.start.row, 0);
            assert_eq!(span.start.column, 0);
            assert_eq!(span.end.row, 0);
            assert_eq!(span.end.column, input.len());
        }
        _ => panic!("Expected FootnoteRef token, got {:?}", tokens[0]),
    }
}

#[rstest]
#[case("[^]")] // Empty label
#[case("[^123]")] // Label can't start with digit
#[case("[^-invalid]")] // Label can't start with dash
#[case("[^invalid.label]")] // Dots not allowed
#[case("[^invalid label]")] // Spaces not allowed
#[case("[^invalid@label]")] // Special chars not allowed
fn test_footnote_ref_labeled_failing(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not produce FootnoteRef token
    let footnote_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::FootnoteRef { .. }))
        .collect();
    
    assert_eq!(footnote_refs.len(), 0, "Should not produce FootnoteRef for: {}", input);
}

// =============================================================================
// FootnoteRef Token - Edge Cases and Error Handling
// =============================================================================

#[rstest]
#[case("[]")] // Empty content
#[case("[1")] // Unclosed bracket
#[case("1]")] // Missing opening bracket
#[case("[ 1]")] // Space before content
#[case("[1 ]")] // Space after content
#[case("[^ note]")] // Space in labeled footnote
fn test_footnote_ref_malformed(#[case] input: &str) {
    let tokens = tokenize(input);

    // Should not produce FootnoteRef token
    let footnote_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::FootnoteRef { .. }))
        .collect();
    
    assert_eq!(footnote_refs.len(), 0, "Should not produce FootnoteRef for malformed: {}", input);
}

#[test]
fn test_footnote_ref_multiline_rejection() {
    let input = "[1\n]";
    let tokens = tokenize(input);

    // Should not produce FootnoteRef token (cannot span lines)
    let footnote_refs: Vec<_> = tokens
        .iter()
        .filter(|token| matches!(token, Token::FootnoteRef { .. }))
        .collect();
    
    assert_eq!(footnote_refs.len(), 0);
}

// =============================================================================
// FootnoteRef Token - Integration with Text
// =============================================================================

#[test]
fn test_footnote_ref_with_surrounding_text() {
    let input = "See footnote [1] for details.";
    let tokens = tokenize(input);

    // Should have: Text + FootnoteRef + Text + EOF
    assert!(tokens.len() >= 4);

    // Find the FootnoteRef token
    let footnote_ref = tokens
        .iter()
        .find(|token| matches!(token, Token::FootnoteRef { .. }))
        .expect("Should find FootnoteRef token");

    match footnote_ref {
        Token::FootnoteRef { footnote_type, .. } => {
            assert_eq!(footnote_type, &FootnoteType::Naked(1));
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_footnote_ref_labeled_with_text() {
    let input = "Important point[^methodology-note] explained here.";
    let tokens = tokenize(input);

    // Find the FootnoteRef token
    let footnote_ref = tokens
        .iter()
        .find(|token| matches!(token, Token::FootnoteRef { .. }))
        .expect("Should find FootnoteRef token");

    match footnote_ref {
        Token::FootnoteRef { footnote_type, .. } => {
            assert_eq!(footnote_type, &FootnoteType::Labeled("methodology-note".to_string()));
        }
        _ => unreachable!(),
    }
}

// =============================================================================
// FootnoteRef Token - Precedence Tests
// =============================================================================

#[test]
fn test_footnote_ref_vs_other_references() {
    // Test that footnote refs take precedence over general references for naked numbers
    let inputs = vec![
        ("[1]", "naked footnote"),
        ("[^note]", "labeled footnote"),
    ];

    for (input, description) in inputs {
        let tokens = tokenize(input);
        
        let footnote_refs: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::FootnoteRef { .. }))
            .collect();
        
        let ref_markers: Vec<_> = tokens
            .iter()
            .filter(|token| matches!(token, Token::RefMarker { .. }))
            .collect();

        assert_eq!(footnote_refs.len(), 1, "Should find FootnoteRef for {}", description);
        assert_eq!(ref_markers.len(), 0, "Should not find RefMarker for {}", description);
    }
}

// =============================================================================
// FootnoteRef Token - Properties and Span Consistency
// =============================================================================

#[test]
fn test_footnote_ref_span_consistency() {
    let test_cases = vec![
        ("[1]", 3),
        ("[42]", 4),
        ("[^note]", 7),
        ("[^detailed-explanation]", 22),
    ];

    for (input, expected_length) in test_cases {
        let tokens = tokenize(input);
        
        let footnote_ref = tokens
            .iter()
            .find(|token| matches!(token, Token::FootnoteRef { .. }))
            .expect("Should find FootnoteRef token");

        match footnote_ref {
            Token::FootnoteRef { span, .. } => {
                assert_eq!(span.start.row, 0);
                assert_eq!(span.start.column, 0);
                assert_eq!(span.end.row, 0);
                assert_eq!(span.end.column, expected_length);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn test_footnote_ref_token_access_methods() {
    let input = "[^test-note]";
    let tokens = tokenize(input);
    
    let footnote_ref = tokens
        .iter()
        .find(|token| matches!(token, Token::FootnoteRef { .. }))
        .expect("Should find FootnoteRef token");

    // Test footnote_type() accessor method
    let footnote_type = footnote_ref.footnote_type().expect("Should return footnote type");
    assert_eq!(footnote_type, &FootnoteType::Labeled("test-note".to_string()));

    // Test that other tokens return None for footnote_type()
    let text_token = tokens
        .iter()
        .find(|token| matches!(token, Token::Eof { .. }))
        .expect("Should find EOF token");
    
    assert!(text_token.footnote_type().is_none());
}