//! Component-Level Assertions
//!
//! These assertions validate individual components (Parameters, Labels, Annotations)
//! that are used across multiple element types.
//!
//! EVERY element assertion MUST use these to validate common fields.

use std::collections::HashMap;

use txxt::ast::{
    elements::annotation::annotation_content::Annotation,
    elements::components::parameters::Parameters,
};

// ============================================================================
// Parameters Assertions
// ============================================================================

/// Assert that Parameters struct contains ALL expected key-value pairs.
///
/// This validates that parameter tokens were collected and stored correctly.
///
/// # Arguments
///
/// * `parameters` - The Parameters struct to validate
/// * `expected` - HashMap of ALL expected key-value pairs
///
/// # Panics
///
/// Panics if:
/// - Any expected parameter is missing
/// - Any expected parameter has wrong value
/// - Parameters contains unexpected extra parameters (strict mode)
///
/// # Examples
///
/// ```rust
/// use std::collections::HashMap;
/// use tests::assertions::component_assertions::assert_parameters_exact;
///
/// let mut expected = HashMap::new();
/// expected.insert("version", "3.11");
/// expected.insert("style", "functional");
///
/// assert_parameters_exact(&element.parameters, &expected);
/// ```
#[allow(dead_code)]
pub fn assert_parameters_exact(parameters: &Parameters, expected: &HashMap<&str, &str>) {
    // Check count first
    assert_eq!(
        parameters.map.len(),
        expected.len(),
        "Parameter count mismatch\n\
         Expected {} parameters\n\
         Actual {} parameters\n\
         Expected keys: {:?}\n\
         Actual parameters: {:?}",
        expected.len(),
        parameters.map.len(),
        expected.keys().collect::<Vec<_>>(),
        parameters
    );

    // Validate each expected parameter
    for (key, expected_value) in expected {
        let actual_value = parameters.get(key);
        assert_eq!(
            actual_value.map(|s| s.as_str()),
            Some(*expected_value),
            "Parameter '{}' validation failed\n\
             Expected value: '{}'\n\
             Actual value: {:?}\n\
             All parameters: {:?}",
            key,
            expected_value,
            actual_value,
            parameters
        );
    }
}

/// Assert that Parameters struct contains at least the specified parameters.
///
/// Allows extra parameters beyond what's expected (non-strict mode).
#[allow(dead_code)]
pub fn assert_parameters_contains(parameters: &Parameters, expected: &HashMap<&str, &str>) {
    for (key, expected_value) in expected {
        let actual_value = parameters.get(key);
        assert_eq!(
            actual_value.map(|s| s.as_str()),
            Some(*expected_value),
            "Parameter '{}' validation failed\n\
             Expected value: '{}'\n\
             Actual value: {:?}",
            key,
            expected_value,
            actual_value
        );
    }
}

/// Assert that Parameters struct has a specific key-value pair.
#[allow(dead_code)]
pub fn assert_parameter(parameters: &Parameters, key: &str, expected_value: &str) {
    let actual_value = parameters.get(key);
    assert_eq!(
        actual_value.map(|s| s.as_str()),
        Some(expected_value),
        "Parameter '{}' not found or wrong value\n\
         Expected: '{}'\n\
         Actual: {:?}",
        key,
        expected_value,
        actual_value
    );
}

/// Assert that Parameters struct is NOT empty.
///
/// Use this to catch parsers that forgot to extract parameter tokens.
#[allow(dead_code)]
pub fn assert_parameters_not_empty(parameters: &Parameters) {
    assert!(
        !parameters.is_empty(),
        "Parameters is empty but should have values!\n\
         This means parameter tokens were not collected from the token stream.\n\
         Actual parameters: {:?}",
        parameters
    );
}

/// Assert that Parameters struct IS empty.
///
/// Use this when element should have no parameters.
#[allow(dead_code)]
pub fn assert_parameters_empty(parameters: &Parameters) {
    assert!(
        parameters.is_empty(),
        "Parameters should be empty but contains values!\n\
         Actual parameters: {:?}",
        parameters
    );
}

// ============================================================================
// Label Assertions
// ============================================================================

/// Assert that a label string exactly matches expected value.
///
/// Use this for validating labels in Verbatim and Annotation elements.
#[allow(dead_code)]
pub fn assert_label_exact(actual: &str, expected: &str) {
    assert_eq!(
        actual.trim(),
        expected.trim(),
        "Label validation failed\n\
         Expected: '{}'\n\
         Actual: '{}'",
        expected,
        actual
    );
}

/// Assert that a label string starts with expected prefix.
#[allow(dead_code)]
pub fn assert_label_starts_with(actual: &str, prefix: &str) {
    assert!(
        actual.starts_with(prefix),
        "Label prefix validation failed\n\
         Expected to start with: '{}'\n\
         Actual label: '{}'",
        prefix,
        actual
    );
}

/// Assert that a label is not empty.
#[allow(dead_code)]
pub fn assert_label_not_empty(actual: &str) {
    assert!(!actual.trim().is_empty(), "Label should not be empty");
}

// ============================================================================
// Annotations Assertions
// ============================================================================

/// Assert annotations vector is not empty.
#[allow(dead_code)]
pub fn assert_has_annotations(annotations: &[Annotation]) {
    assert!(
        !annotations.is_empty(),
        "Annotations vector is empty but should have values!\n\
         This means annotation tokens were not parsed.\n\
         Actual annotations: {:?}",
        annotations
    );
}

/// Assert annotations vector has exactly N elements.
#[allow(dead_code)]
pub fn assert_annotation_count_exact(annotations: &[Annotation], expected_count: usize) {
    assert_eq!(
        annotations.len(),
        expected_count,
        "Annotation count mismatch\n\
         Expected: {} annotations\n\
         Actual: {} annotations",
        expected_count,
        annotations.len()
    );
}

/// Assert annotations vector contains annotation with specific label.
#[allow(dead_code)]
pub fn assert_has_annotation_with_label(annotations: &[Annotation], label: &str) {
    let found = annotations.iter().any(|ann| ann.label == label);
    assert!(
        found,
        "Annotation with label '{}' not found\n\
         Available annotations: {:?}",
        label,
        annotations.iter().map(|a| &a.label).collect::<Vec<_>>()
    );
}
