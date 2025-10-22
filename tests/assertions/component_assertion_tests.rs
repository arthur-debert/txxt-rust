//! Component Assertion Tests
//!
//! Tests for low-level component assertions (Parameters, Labels, Annotations).
//! Uses hand-crafted data structures to prove assertions work independently of parsers.

use std::collections::HashMap;

use txxt::ast::{
    elements::annotation::annotation_content::Annotation,
    elements::components::parameters::Parameters,
};

use super::elements::components::component_assertions::*;

// ============================================================================
// Parameters Assertion Tests
// ============================================================================

fn make_test_parameters(pairs: &[(&str, &str)]) -> Parameters {
    let mut params = Parameters::new();
    for (key, value) in pairs {
        params.set(key.to_string(), value.to_string());
    }
    params
}

#[test]
fn test_assert_parameters_exact_success() {
    let params = make_test_parameters(&[("version", "3.11"), ("style", "functional")]);

    let mut expected = HashMap::new();
    expected.insert("version", "3.11");
    expected.insert("style", "functional");

    // Should not panic
    assert_parameters_exact(&params, &expected);
}

#[test]
#[should_panic(expected = "Parameter count mismatch")]
fn test_assert_parameters_exact_count_mismatch() {
    let params = make_test_parameters(&[("version", "3.11")]);

    let mut expected = HashMap::new();
    expected.insert("version", "3.11");
    expected.insert("style", "functional"); // Extra parameter

    assert_parameters_exact(&params, &expected);
}

#[test]
#[should_panic(expected = "Parameter 'version' validation failed")]
fn test_assert_parameters_exact_value_mismatch() {
    let params = make_test_parameters(&[("version", "3.10")]);

    let mut expected = HashMap::new();
    expected.insert("version", "3.11"); // Wrong value

    assert_parameters_exact(&params, &expected);
}

#[test]
fn test_assert_parameter_single_success() {
    let params = make_test_parameters(&[("version", "3.11"), ("style", "functional")]);

    // Should not panic
    assert_parameter(&params, "version", "3.11");
    assert_parameter(&params, "style", "functional");
}

#[test]
#[should_panic(expected = "Parameter 'missing' not found")]
fn test_assert_parameter_missing() {
    let params = make_test_parameters(&[("version", "3.11")]);
    assert_parameter(&params, "missing", "value");
}

#[test]
fn test_assert_parameters_not_empty_success() {
    let params = make_test_parameters(&[("key", "value")]);
    assert_parameters_not_empty(&params);
}

#[test]
#[should_panic(expected = "Parameters is empty")]
fn test_assert_parameters_not_empty_fails() {
    let params = Parameters::new();
    assert_parameters_not_empty(&params);
}

#[test]
fn test_assert_parameters_empty_success() {
    let params = Parameters::new();
    assert_parameters_empty(&params);
}

#[test]
#[should_panic(expected = "Parameters should be empty")]
fn test_assert_parameters_empty_fails() {
    let params = make_test_parameters(&[("key", "value")]);
    assert_parameters_empty(&params);
}

// ============================================================================
// Label Assertion Tests
// ============================================================================

#[test]
fn test_assert_label_exact_success() {
    assert_label_exact("python", "python");
    assert_label_exact("  python  ", "python"); // Trimming works
}

#[test]
#[should_panic(expected = "Label validation failed")]
fn test_assert_label_exact_fails() {
    assert_label_exact("javascript", "python");
}

#[test]
fn test_assert_label_starts_with_success() {
    assert_label_starts_with("python.advanced", "python");
    assert_label_starts_with("lang.python", "lang.");
}

#[test]
#[should_panic(expected = "Label prefix validation failed")]
fn test_assert_label_starts_with_fails() {
    assert_label_starts_with("javascript", "python");
}

#[test]
fn test_assert_label_not_empty_success() {
    assert_label_not_empty("python");
    assert_label_not_empty("  x  ");
}

#[test]
#[should_panic(expected = "Label should not be empty")]
fn test_assert_label_not_empty_fails() {
    assert_label_not_empty("");
}

// ============================================================================
// Annotations Assertion Tests
// ============================================================================

fn make_test_annotation(label: &str) -> Annotation {
    use txxt::ast::elements::annotation::annotation_content::AnnotationContent;

    Annotation::new(label.to_string(), AnnotationContent::Empty)
}

#[test]
fn test_assert_has_annotations_success() {
    let annotations = vec![make_test_annotation("note")];
    assert_has_annotations(&annotations);
}

#[test]
#[should_panic(expected = "Annotations vector is empty")]
fn test_assert_has_annotations_fails() {
    let annotations: Vec<Annotation> = vec![];
    assert_has_annotations(&annotations);
}

#[test]
fn test_assert_annotation_count_exact_success() {
    let annotations = vec![
        make_test_annotation("note"),
        make_test_annotation("warning"),
    ];
    assert_annotation_count_exact(&annotations, 2);
}

#[test]
#[should_panic(expected = "Annotation count mismatch")]
fn test_assert_annotation_count_exact_fails() {
    let annotations = vec![make_test_annotation("note")];
    assert_annotation_count_exact(&annotations, 2);
}

#[test]
fn test_assert_has_annotation_with_label_success() {
    let annotations = vec![
        make_test_annotation("note"),
        make_test_annotation("warning"),
    ];
    assert_has_annotation_with_label(&annotations, "note");
    assert_has_annotation_with_label(&annotations, "warning");
}

#[test]
#[should_panic(expected = "Annotation with label 'missing' not found")]
fn test_assert_has_annotation_with_label_fails() {
    let annotations = vec![make_test_annotation("note")];
    assert_has_annotation_with_label(&annotations, "missing");
}
