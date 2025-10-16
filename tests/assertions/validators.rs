//! Shared validation helpers used by all assertion functions.
//!
//! These utilities provide consistent validation logic across all element types,
//! reducing duplication and ensuring uniform error messages.

use std::collections::HashMap;

use txxt::ast::{
    elements::annotation::annotation_content::Annotation,
    elements::components::parameters::Parameters, elements::inlines::TextTransform,
};

// ============================================================================
// Parameter Validation
// ============================================================================

/// Validate that all expected parameters exist with correct values.
///
/// # Arguments
///
/// * `actual` - The Parameters from the parsed element
/// * `expected` - HashMap of expected key-value pairs
///
/// # Panics
///
/// Panics if any expected parameter is missing or has wrong value.
#[allow(dead_code)]
pub fn validate_parameters(actual: &Parameters, expected: &HashMap<&str, &str>) {
    for (key, expected_value) in expected {
        match actual.get(key) {
            Some(actual_value) if actual_value == expected_value => {
                // Match - continue
            }
            Some(actual_value) => {
                panic!(
                    "Parameter validation failed for key '{}'\n\
                     Expected value: '{}'\n\
                     Actual value: '{}'\n\
                     All parameters: {:?}",
                    key, expected_value, actual_value, actual.map
                );
            }
            None => {
                panic!(
                    "Parameter validation failed: missing key '{}'\n\
                     Expected: '{}' = '{}'\n\
                     Available parameters: {:?}",
                    key,
                    key,
                    expected_value,
                    actual.map.keys()
                );
            }
        }
    }
}

/// Validate a specific parameter exists with expected value.
pub fn validate_parameter(actual: &Parameters, key: &str, expected_value: &str) {
    let mut map = HashMap::new();
    map.insert(key, expected_value);
    validate_parameters(actual, &map);
}

// ============================================================================
// Annotation Validation
// ============================================================================

/// Validate annotation count.
///
/// # Panics
///
/// Panics if annotation count doesn't match expected.
pub fn validate_annotation_count(actual: &[Annotation], expected_count: usize) {
    let actual_count = actual.len();
    assert_eq!(
        actual_count,
        expected_count,
        "Annotation count mismatch\n\
         Expected: {} annotations\n\
         Actual: {} annotations\n\
         Annotations found: {:?}",
        expected_count,
        actual_count,
        actual.iter().map(|a| &a.label).collect::<Vec<_>>()
    );
}

/// Validate element has annotation with specific label.
///
/// # Panics
///
/// Panics if annotation with label is not found.
pub fn validate_has_annotation(actual: &[Annotation], expected_label: &str) {
    let found = actual.iter().any(|a| a.label == expected_label);
    assert!(
        found,
        "Annotation validation failed\n\
         Expected annotation with label: '{}'\n\
         Actual annotations: {:?}",
        expected_label,
        actual.iter().map(|a| &a.label).collect::<Vec<_>>()
    );
}

// ============================================================================
// Text Content Validation
// ============================================================================

/// Extract all text content from TextTransforms recursively.
///
/// Walks through all transforms and concatenates text spans.
pub fn extract_all_text(transforms: &[TextTransform]) -> String {
    transforms
        .iter()
        .map(|t| t.text_content())
        .collect::<Vec<_>>()
        .join("")
}

/// Validate text content exactly matches expected.
pub fn validate_text_exact(transforms: &[TextTransform], expected: &str) {
    let actual = extract_all_text(transforms);
    assert_eq!(
        actual, expected,
        "Text content mismatch\n\
         Expected: '{}'\n\
         Actual: '{}'",
        expected, actual
    );
}

/// Validate text content contains expected substring.
pub fn validate_text_contains(transforms: &[TextTransform], needle: &str) {
    let actual = extract_all_text(transforms);
    assert!(
        actual.contains(needle),
        "Text content validation failed\n\
         Expected to contain: '{}'\n\
         Actual text: '{}'\n\
         Note: Search is case-sensitive",
        needle,
        actual
    );
}

/// Validate text content matches regex pattern.
pub fn validate_text_matches(transforms: &[TextTransform], pattern: &str) {
    use regex::Regex;

    let regex = Regex::new(pattern)
        .unwrap_or_else(|e| panic!("Invalid regex pattern '{}': {}", pattern, e));

    let actual = extract_all_text(transforms);
    assert!(
        regex.is_match(&actual),
        "Text content regex validation failed\n\
         Pattern: '{}'\n\
         Actual text: '{}'",
        pattern,
        actual
    );
}

/// Check if transforms contain formatting beyond plain text.
pub fn has_formatting(transforms: &[TextTransform]) -> bool {
    transforms
        .iter()
        .any(|t| !matches!(t, TextTransform::Identity(_)))
}

/// Validate transforms contain formatting.
pub fn validate_has_formatting(transforms: &[TextTransform], expected: bool) {
    let actual = has_formatting(transforms);
    assert_eq!(
        actual,
        expected,
        "Formatting validation failed\n\
         Expected has_formatting: {}\n\
         Actual: {}\n\
         Transforms: {} total",
        expected,
        actual,
        transforms.len()
    );
}

// ============================================================================
// Element Type Validation
// ============================================================================

/// Validate container has expected number of elements.
#[allow(dead_code)] // Will be used during container parsing
pub fn validate_element_count<T>(elements: &[T], expected_count: usize, context: &str) {
    let actual_count = elements.len();
    assert_eq!(
        actual_count, expected_count,
        "{} element count mismatch\n\
         Expected: {} elements\n\
         Actual: {} elements",
        context, expected_count, actual_count
    );
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Format element type for error messages.
#[allow(dead_code)] // Will be used when more assertions are implemented
pub fn format_element_type_name(_element: &str) -> &'static str {
    // Placeholder - will use actual element discrimination when integrated
    "Element"
}
