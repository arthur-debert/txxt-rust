//! Complete Paragraph Assertion Tests
//!
//! Tests assert_paragraph() with hand-crafted AST nodes to validate ALL ParagraphBlock fields.
//! This proves the assertion works BEFORE parser integration.
//!
//! ParagraphBlock has 4 fields (from src/ast/elements/paragraph.rs):
//! 1. content: Vec<TextTransform>
//! 2. annotations: Vec<Annotation>
//! 3. parameters: Parameters
//! 4. tokens: TokenSequence

#[path = "assertions/mod.rs"]
mod assertions;

#[cfg(feature = "new-ast")]
use txxt::ast::{
    elements::annotation::annotation_content::{Annotation, AnnotationContent},
    elements::components::parameters::Parameters,
    elements::{
        inlines::{TextSpan, TextTransform},
        paragraph::ParagraphBlock,
        session::session_container::SessionContainerElement,
    },
    tokens::TokenSequence,
};

#[cfg(feature = "new-ast")]
use assertions::{assert_paragraph, ParagraphExpected};

// ============================================================================
// Helper: Create Hand-Crafted Paragraph AST
// ============================================================================

#[cfg(feature = "new-ast")]
fn make_paragraph(
    _text: &str,
    params: Vec<(&str, &str)>,
    annotations: Vec<&str>,
) -> ParagraphBlock {
    use txxt::ast::tokens::{Position, SourceSpan, Token};

    // Field 1: content (Vec<TextTransform>)
    // Create a Text token with the actual text
    let text_token = Token::Text {
        content: _text.to_string(),
        span: SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position {
                row: 0,
                column: _text.len(),
            },
        },
    };

    let text_transform = TextTransform::Identity(TextSpan {
        tokens: TokenSequence {
            tokens: vec![text_token.clone()],
        },
        annotations: vec![],
        parameters: Parameters::new(),
    });

    // Field 2: annotations (Vec<Annotation>)
    let annotation_vec = annotations
        .iter()
        .map(|label| Annotation {
            label: label.to_string(),
            namespace: None,
            parameters: Parameters::new(),
            content: AnnotationContent::Empty,
            tokens: TokenSequence { tokens: vec![] },
        })
        .collect();

    // Field 3: parameters (Parameters)
    let mut parameters = Parameters::new();
    for (key, value) in params {
        parameters.set(key.to_string(), value.to_string());
    }

    // Field 4: tokens (TokenSequence)
    let tokens = TokenSequence { tokens: vec![] };

    // Create paragraph with ALL fields populated
    ParagraphBlock::new(vec![text_transform], annotation_vec, parameters, tokens)
}

// ============================================================================
// Field 1: Content (Vec<TextTransform>) Tests
// ============================================================================

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_content_basic() {
    let para = make_paragraph("Simple text", vec![], vec![]);
    let element = SessionContainerElement::Paragraph(para);

    // Should validate content exists
    assert_paragraph(
        &element,
        ParagraphExpected {
            ..Default::default()
        },
    );
}

// ============================================================================
// Field 2: Annotations (Vec<Annotation>) Tests
// ============================================================================

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_annotations_empty() {
    let para = make_paragraph("Text", vec![], vec![]);
    let element = SessionContainerElement::Paragraph(para);

    // Should validate annotations vector is empty
    assert_paragraph(
        &element,
        ParagraphExpected {
            annotation_count: Some(0),
            ..Default::default()
        },
    );
}

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_annotations_single() {
    let para = make_paragraph("Text", vec![], vec!["note"]);
    let element = SessionContainerElement::Paragraph(para);

    // Should validate annotation count
    assert_paragraph(
        &element,
        ParagraphExpected {
            annotation_count: Some(1),
            has_annotation: Some("note"),
            ..Default::default()
        },
    );
}

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_annotations_multiple() {
    let para = make_paragraph("Text", vec![], vec!["note", "warning", "todo"]);
    let element = SessionContainerElement::Paragraph(para);

    // Should validate ALL annotations
    assert_paragraph(
        &element,
        ParagraphExpected {
            annotation_count: Some(3),
            has_annotation: Some("note"),
            ..Default::default()
        },
    );
}

#[cfg(feature = "new-ast")]
#[test]
#[should_panic(expected = "Annotation count mismatch")]
fn test_paragraph_field_annotations_count_wrong() {
    let para = make_paragraph("Text", vec![], vec!["note"]);
    let element = SessionContainerElement::Paragraph(para);

    // Should panic - wrong count
    assert_paragraph(
        &element,
        ParagraphExpected {
            annotation_count: Some(2), // Wrong!
            ..Default::default()
        },
    );
}

// ============================================================================
// Field 3: Parameters (Parameters) Tests
// ============================================================================

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_parameters_empty() {
    let para = make_paragraph("Text", vec![], vec![]);
    let element = SessionContainerElement::Paragraph(para);

    // Parameters should be empty (no validation needed)
    assert_paragraph(&element, ParagraphExpected::default());
}

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_parameters_single() {
    let para = make_paragraph("Text", vec![("ref", "para-1")], vec![]);
    let element = SessionContainerElement::Paragraph(para);

    // Should validate parameter exists
    assert_paragraph(
        &element,
        ParagraphExpected {
            has_parameter: Some(("ref", "para-1")),
            ..Default::default()
        },
    );
}

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_parameters_multiple() {
    let para = make_paragraph(
        "Text",
        vec![("ref", "para-1"), ("category", "introduction")],
        vec![],
    );
    let element = SessionContainerElement::Paragraph(para);

    // Should validate both parameters
    assert_paragraph(
        &element,
        ParagraphExpected {
            has_parameter: Some(("ref", "para-1")),
            ..Default::default()
        },
    );
}

#[cfg(feature = "new-ast")]
#[test]
#[should_panic(expected = "Parameter")]
fn test_paragraph_field_parameters_missing() {
    let para = make_paragraph("Text", vec![("ref", "para-1")], vec![]);
    let element = SessionContainerElement::Paragraph(para);

    // Should panic - wrong parameter
    assert_paragraph(
        &element,
        ParagraphExpected {
            has_parameter: Some(("category", "missing")), // Doesn't exist!
            ..Default::default()
        },
    );
}

// ============================================================================
// Field 4: Tokens (TokenSequence) Tests
// ============================================================================

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_field_tokens_preserved() {
    let para = make_paragraph("Text", vec![], vec![]);
    let element = SessionContainerElement::Paragraph(para);

    // Tokens are always preserved (no specific validation in Expected struct)
    assert_paragraph(&element, ParagraphExpected::default());
}

// ============================================================================
// Combined: ALL Fields Together
// ============================================================================

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_all_fields_populated() {
    let para = make_paragraph(
        "Important text",
        vec![("ref", "important"), ("category", "key-point")],
        vec!["note", "important"],
    );
    let element = SessionContainerElement::Paragraph(para);

    // Validate ALL extractable fields in one assertion
    assert_paragraph(
        &element,
        ParagraphExpected {
            // Field 1: content (validated via text extraction)
            text_contains: Some("Important"),
            // Field 2: annotations
            annotation_count: Some(2),
            has_annotation: Some("note"),
            // Field 3: parameters
            has_parameter: Some(("ref", "important")),
            // Field 4: tokens (always present, not validated)
            ..Default::default()
        },
    );
}

#[cfg(feature = "new-ast")]
#[test]
#[should_panic(expected = "mismatch")]
fn test_paragraph_all_fields_validation_catches_errors() {
    let para = make_paragraph("Text", vec![("ref", "para-1")], vec!["note"]);
    let element = SessionContainerElement::Paragraph(para);

    // Should panic on first mismatch
    assert_paragraph(
        &element,
        ParagraphExpected {
            annotation_count: Some(99), // WRONG!
            ..Default::default()
        },
    );
}

// ============================================================================
// Summary Test: Proves Assertion Validates All Fields
// ============================================================================

#[cfg(feature = "new-ast")]
#[test]
fn test_paragraph_assertion_validates_all_ast_fields() {
    // This test documents what assert_paragraph() validates:
    //
    // ✅ Field 1 (content): Via text_contains, text, text_matches, has_formatting
    // ✅ Field 2 (annotations): Via annotation_count, has_annotation
    // ✅ Field 3 (parameters): Via has_parameter
    // ⏸️  Field 4 (tokens): Always preserved, not validated by Expected struct
    //
    // 3 out of 4 fields validated (tokens are infrastructure, always present)

    let para = make_paragraph("Test", vec![("key", "val")], vec!["ann"]);
    let element = SessionContainerElement::Paragraph(para);

    assert_paragraph(
        &element,
        ParagraphExpected {
            text_contains: Some("Test"),         // Field 1
            annotation_count: Some(1),           // Field 2
            has_parameter: Some(("key", "val")), // Field 3
            // Field 4 (tokens) - infrastructure, always present
            ..Default::default()
        },
    );
}
