//! AST Assertion Framework
//!
//! Ergonomic assertion helpers for validating parsed AST elements using
//! builder-pattern Expected structs with optional fields.
//!
//! # Quick Start
//!
//! ```rust
//! use tests::assertions::{assert_paragraph, ParagraphExpected};
//!
//! let para = parse_paragraph(&source).unwrap();
//!
//! // Test only what matters - unspecified fields are not validated
//! assert_paragraph(&para, ParagraphExpected {
//!     text_contains: Some("expected content"),
//!     has_formatting: Some(true),
//!     ..Default::default()
//! });
//! ```
//!
//! # Design
//!
//! - **One function per element**: `assert_paragraph()`, `assert_list()`, etc.
//! - **Optional validation**: Only `Some()` fields are checked
//! - **Shared logic**: Common validators for parameters, annotations, text
//! - **Helpful errors**: Clear messages showing expected vs actual
//!
//! # Implementation Status
//!
//! - ✅ `assert_paragraph()` - Complete example implementation
//! - ⏳ `assert_list()` - To be implemented during Parser 2.1.2
//! - ⏳ `assert_definition()` - To be implemented during Parser 2.1.3
//! - ⏳ `assert_session()` - To be implemented during Parser 2.1.5
//! - ⏳ Other elements - Implemented when needed
//!
//! # Adding New Assertions
//!
//! When implementing a parser for a new element, add its assertion:
//!
//! 1. Expected struct already exists in `expected.rs`
//! 2. Copy `assert_paragraph()` as template
//! 3. Rename to `assert_your_element()`
//! 4. Update downcast logic for your element type
//! 5. Implement element-specific validation
//! 6. Reuse shared validators for common properties
//! 7. Add tests to `tests.rs`
//!
//! Time per element: ~30 minutes

pub mod component_assertions;
pub mod expected;
pub mod validators;

// Re-export expected structs for convenience
#[cfg(feature = "new-ast")]
pub use expected::*;

#[cfg(feature = "new-ast")]
use txxt::ast::elements::{
    containers::session::SessionContainerElement, paragraph::ParagraphBlock,
};

#[cfg(feature = "new-ast")]
use validators::{
    validate_annotation_count, validate_has_annotation, validate_has_formatting,
    validate_parameter, validate_text_contains, validate_text_exact, validate_text_matches,
};

// ============================================================================
// PARAGRAPH ASSERTION (Complete Example Implementation)
// ============================================================================

/// Assert element is a paragraph and validate specified properties.
#[allow(dead_code)] // Used in paragraph_parser_tests with new-ast feature
///
/// This is the **reference implementation** that demonstrates the pattern
/// for all element assertions. Other assertions should follow this template.
///
/// # Arguments
///
/// * `element` - The SessionContainerElement to validate
/// * `expected` - ParagraphExpected with properties to validate
///
/// # Returns
///
/// Returns `&ParagraphBlock` reference for further validation if needed.
///
/// # Panics
///
/// Panics if:
/// - Element is not a ParagraphBlock
/// - Any specified property doesn't match expected value
///
/// # Examples
///
/// ```rust
/// use tests::assertions::{assert_paragraph, ParagraphExpected};
///
/// // Minimal validation (one property)
/// assert_paragraph(&element, ParagraphExpected {
///     text_contains: Some("expected"),
///     ..Default::default()
/// });
///
/// // Comprehensive validation (many properties)
/// assert_paragraph(&element, ParagraphExpected {
///     text: Some("This is a complete paragraph."),
///     has_formatting: Some(false),
///     annotation_count: Some(0),
///     ..Default::default()
/// });
/// ```
#[cfg(feature = "new-ast")]
pub fn assert_paragraph<'a>(
    element: &'a SessionContainerElement,
    expected: ParagraphExpected<'a>,
) -> &'a ParagraphBlock {
    // Step 1: Type check and downcast
    let paragraph = match element {
        SessionContainerElement::Paragraph(p) => p,
        other => {
            panic!(
                "Element type assertion failed\n\
                 Expected: ParagraphBlock\n\
                 Actual: {:?}\n\
                 Hint: Ensure you're extracting the correct element from the container",
                element_type_name(other)
            );
        }
    };

    // Step 2: Validate each specified property

    // Text validation (exact match)
    if let Some(expected_text) = expected.text {
        validate_text_exact(&paragraph.content, expected_text);
    }

    // Text validation (contains substring)
    if let Some(needle) = expected.text_contains {
        validate_text_contains(&paragraph.content, needle);
    }

    // Text validation (regex match)
    if let Some(pattern) = expected.text_matches {
        validate_text_matches(&paragraph.content, pattern);
    }

    // Formatting validation
    if let Some(expected_formatting) = expected.has_formatting {
        validate_has_formatting(&paragraph.content, expected_formatting);
    }

    // Annotation count validation
    if let Some(expected_count) = expected.annotation_count {
        validate_annotation_count(&paragraph.annotations, expected_count);
    }

    // Specific annotation label validation
    if let Some(label) = expected.has_annotation {
        validate_has_annotation(&paragraph.annotations, label);
    }

    // Parameter validation
    if let Some((key, value)) = expected.has_parameter {
        validate_parameter(&paragraph.parameters, key, value);
    }

    // Step 3: Return reference for further use
    paragraph
}

// ============================================================================
// PLACEHOLDER ASSERTIONS (To be implemented)
// ============================================================================

// The following are placeholders showing function signatures.
// Each will be implemented when the corresponding parser element is implemented.

/// Assert element is a list and validate specified properties.
#[allow(dead_code)] // Used in list_parser_tests with new-ast feature
///
/// Follows the same pattern as `assert_paragraph()` but adapted for list-specific properties.
///
/// # Arguments
///
/// * `element` - The SessionContainerElement to validate
/// * `expected` - ListExpected with properties to validate
///
/// # Returns
///
/// Returns `&ListBlock` reference for further validation if needed.
///
/// # Panics
///
/// Panics if:
/// - Element is not a ListBlock
/// - Any specified property doesn't match expected value
///
/// # Examples
///
/// ```rust
/// use tests::assertions::{assert_list, ListExpected};
/// use txxt::ast::elements::list::NumberingStyle;
///
/// // Minimal validation
/// assert_list(&element, ListExpected {
///     item_count: Some(3),
///     ..Default::default()
/// });
///
/// // Comprehensive validation
/// assert_list(&element, ListExpected {
///     style: Some(NumberingStyle::Numerical),
///     item_count: Some(3),
///     item_text: Some(vec!["First", "Second", "Third"]),
///     ..Default::default()
/// });
/// ```
#[cfg(feature = "new-ast")]
pub fn assert_list<'a>(
    element: &'a SessionContainerElement,
    expected: ListExpected<'a>,
) -> &'a txxt::ast::elements::list::ListBlock {
    // Step 1: Type check and downcast
    let list = match element {
        SessionContainerElement::List(l) => l,
        other => {
            panic!(
                "Element type assertion failed\n\
                 Expected: ListBlock\n\
                 Actual: {:?}\n\
                 Hint: Ensure you're extracting the correct element from the container",
                element_type_name(other)
            );
        }
    };

    // Step 2: Validate each specified property

    // List style validation
    if let Some(expected_style) = expected.style {
        assert_eq!(
            list.decoration_type.style, expected_style,
            "List style mismatch\n\
             Expected: {:?}\n\
             Actual: {:?}",
            expected_style, list.decoration_type.style
        );
    }

    // Item count validation
    if let Some(expected_count) = expected.item_count {
        assert_eq!(
            list.len(),
            expected_count,
            "List item count mismatch\n\
             Expected: {} items\n\
             Actual: {} items",
            expected_count,
            list.len()
        );
    }

    // Item text validation
    if let Some(expected_texts) = expected.item_text {
        assert_eq!(
            list.len(),
            expected_texts.len(),
            "List item count mismatch for text validation\n\
             Expected: {} items\n\
             Actual: {} items",
            expected_texts.len(),
            list.len()
        );

        for (i, (item, expected_text)) in list.items.iter().zip(expected_texts.iter()).enumerate() {
            let actual_text = item.text_content();
            assert!(
                actual_text.contains(expected_text),
                "List item {} text mismatch\n\
                 Expected to contain: '{}'\n\
                 Actual: '{}'",
                i,
                expected_text,
                actual_text
            );
        }
    }

    // Nested content validation
    if let Some(expected_nested) = expected.has_nested {
        assert_eq!(
            list.len(),
            expected_nested.len(),
            "List item count mismatch for nested validation\n\
             Expected: {} items\n\
             Actual: {} items",
            expected_nested.len(),
            list.len()
        );

        for (i, (item, expected_has_nested)) in
            list.items.iter().zip(expected_nested.iter()).enumerate()
        {
            assert_eq!(
                item.has_nested_content(),
                *expected_has_nested,
                "List item {} nested content mismatch\n\
                 Expected has_nested: {}\n\
                 Actual has_nested: {}",
                i,
                expected_has_nested,
                item.has_nested_content()
            );
        }
    }

    // Annotation count validation
    if let Some(expected_count) = expected.annotation_count {
        validate_annotation_count(&list.annotations, expected_count);
    }

    // Specific annotation label validation
    if let Some(label) = expected.has_annotation {
        validate_has_annotation(&list.annotations, label);
    }

    // Step 3: Return reference for further use
    list
}

/// Assert element is a definition and validate specified properties.
///
/// Follows the same pattern as `assert_paragraph()` but adapted for definition-specific properties.
///
/// # Arguments
///
/// * `element` - The SessionContainerElement to validate
/// * `expected` - DefinitionExpected with properties to validate
///
/// # Returns
///
/// Returns `&DefinitionBlock` reference for further validation if needed.
///
/// # Panics
///
/// Panics if:
/// - Element is not a DefinitionBlock
/// - Any specified property doesn't match expected value
///
/// # Examples
///
/// ```rust
/// use tests::assertions::{assert_definition, DefinitionExpected};
///
/// // Minimal validation
/// assert_definition(&element, DefinitionExpected {
///     term: Some("API"),
///     ..Default::default()
/// });
///
/// // Comprehensive validation
/// assert_definition(&element, DefinitionExpected {
///     term: Some("Recursion"),
///     has_content: Some(true),
///     content_count: Some(3),
///     ..Default::default()
/// });
/// ```
#[cfg(feature = "new-ast")]
#[allow(dead_code)] // Used in definition_parser_tests with new-ast feature
pub fn assert_definition<'a>(
    element: &'a SessionContainerElement,
    expected: DefinitionExpected<'a>,
) -> &'a txxt::ast::elements::definition::DefinitionBlock {
    // Step 1: Type check and downcast
    let definition = match element {
        SessionContainerElement::Definition(d) => d,
        other => {
            panic!(
                "Element type assertion failed\n\
                 Expected: DefinitionBlock\n\
                 Actual: {:?}\n\
                 Hint: Ensure you're extracting the correct element from the container",
                element_type_name(other)
            );
        }
    };

    // Step 2: Validate each specified property

    // Term exact match validation
    if let Some(expected_term) = expected.term {
        let actual_term = definition.term_text();
        assert_eq!(
            actual_term.trim(),
            expected_term.trim(),
            "Term mismatch\n\
             Expected: '{}'\n\
             Actual: '{}'",
            expected_term,
            actual_term
        );
    }

    // Term contains substring validation
    if let Some(needle) = expected.term_contains {
        let actual_term = definition.term_text();
        assert!(
            actual_term.contains(needle),
            "Term substring validation failed\n\
             Expected to contain: '{}'\n\
             Actual term: '{}'",
            needle,
            actual_term
        );
    }

    // Content existence validation
    if let Some(expected_has_content) = expected.has_content {
        let actual_has_content = !definition.is_content_empty();
        assert_eq!(
            actual_has_content, expected_has_content,
            "Definition content existence mismatch\n\
             Expected has_content: {}\n\
             Actual has_content: {}",
            expected_has_content, actual_has_content
        );
    }

    // Content count validation
    if let Some(expected_count) = expected.content_count {
        let actual_count = definition.content.len();
        assert_eq!(
            actual_count, expected_count,
            "Definition content count mismatch\n\
             Expected: {} elements\n\
             Actual: {} elements",
            expected_count, actual_count
        );
    }

    // Content types validation
    if let Some(expected_types) = expected.content_types {
        let actual_count = definition.content.len();
        assert_eq!(
            actual_count,
            expected_types.len(),
            "Content count mismatch for type validation\n\
             Expected: {} elements\n\
             Actual: {} elements",
            expected_types.len(),
            actual_count
        );

        for (i, (_element, expected_type)) in definition
            .content
            .content
            .iter()
            .zip(expected_types.iter())
            .enumerate()
        {
            // Type validation would go here when full content parsing is implemented
            // For now, just verify expected_types list matches count
            let _ = (i, expected_type);
        }
    }

    // Annotation count validation
    if let Some(expected_count) = expected.annotation_count {
        validate_annotation_count(&definition.annotations, expected_count);
    }

    // Specific annotation label validation
    if let Some(label) = expected.has_annotation {
        validate_has_annotation(&definition.annotations, label);
    }

    // Step 3: Return reference for further use
    definition
}

/// Assert element is a session and validate specified properties.
///
/// Follows the same pattern as `assert_paragraph()` but adapted for session-specific properties.
///
/// # Arguments
///
/// * `element` - The SessionContainerElement to validate
/// * `expected` - SessionExpected with properties to validate
///
/// # Returns
///
/// Returns `&SessionBlock` reference for further validation if needed.
///
/// # Panics
///
/// Panics if:
/// - Element is not a SessionBlock
/// - Any specified property doesn't match expected value
///
/// # Examples
///
/// ```rust
/// use tests::assertions::{assert_session, SessionExpected};
///
/// // Minimal validation
/// assert_session(&element, SessionExpected {
///     is_numbered: Some(false),
///     ..Default::default()
/// });
///
/// // Comprehensive validation
/// assert_session(&element, SessionExpected {
///     title: Some("Introduction"),
///     is_numbered: Some(true),
///     numbering: Some("1."),
///     child_count: Some(3),
///     ..Default::default()
/// });
/// ```
#[cfg(feature = "new-ast")]
#[allow(dead_code)] // Used in session_parser_tests with new-ast feature
pub fn assert_session<'a>(
    element: &'a SessionContainerElement,
    expected: SessionExpected<'a>,
) -> &'a txxt::ast::elements::session::SessionBlock {
    // Step 1: Type check and downcast
    let session = match element {
        SessionContainerElement::Session(s) => s,
        other => {
            panic!(
                "Element type assertion failed\n\
                 Expected: SessionBlock\n\
                 Actual: {:?}\n\
                 Hint: Ensure you're extracting the correct element from the container",
                element_type_name(other)
            );
        }
    };

    // Step 2: Validate each specified property

    // Title exact match validation
    if let Some(expected_title) = expected.title {
        let actual_title = session.title_text();
        assert_eq!(
            actual_title.trim(),
            expected_title.trim(),
            "Title mismatch\n\
             Expected: '{}'\n\
             Actual: '{}'",
            expected_title,
            actual_title
        );
    }

    // Title contains substring validation
    if let Some(needle) = expected.title_contains {
        let actual_title = session.title_text();
        assert!(
            actual_title.contains(needle),
            "Title substring validation failed\n\
             Expected to contain: '{}'\n\
             Actual title: '{}'",
            needle,
            actual_title
        );
    }

    // Is numbered validation
    if let Some(expected_is_numbered) = expected.is_numbered {
        let actual_is_numbered = session.has_numbering();
        assert_eq!(
            actual_is_numbered, expected_is_numbered,
            "Session numbering presence mismatch\n\
             Expected is_numbered: {}\n\
             Actual is_numbered: {}",
            expected_is_numbered, actual_is_numbered
        );
    }

    // Numbering marker exact match validation
    if let Some(expected_marker) = expected.numbering {
        let actual_marker = session.numbering_marker().unwrap_or("");
        assert_eq!(
            actual_marker.trim(),
            expected_marker.trim(),
            "Numbering marker mismatch\n\
             Expected: '{}'\n\
             Actual: '{}'",
            expected_marker,
            actual_marker
        );
    }

    // Child count validation
    if let Some(expected_count) = expected.child_count {
        let actual_count = session.content.len();
        assert_eq!(
            actual_count, expected_count,
            "Session child count mismatch\n\
             Expected: {} children\n\
             Actual: {} children",
            expected_count, actual_count
        );
    }

    // Has subsession validation
    if let Some(expected_has_subsession) = expected.has_subsession {
        let actual_has_subsession = !session.content.sessions().is_empty();
        assert_eq!(
            actual_has_subsession, expected_has_subsession,
            "Session subsession presence mismatch\n\
             Expected has_subsession: {}\n\
             Actual has_subsession: {}",
            expected_has_subsession, actual_has_subsession
        );
    }

    // Child types validation
    if let Some(expected_types) = expected.child_types {
        let actual_count = session.content.len();
        assert_eq!(
            actual_count,
            expected_types.len(),
            "Child count mismatch for type validation\n\
             Expected: {} children\n\
             Actual: {} children",
            expected_types.len(),
            actual_count
        );

        // Type validation would go here when full content parsing is implemented
        // For now, just verify expected_types list matches count
        let _ = expected_types;
    }

    // Annotation count validation
    if let Some(expected_count) = expected.annotation_count {
        validate_annotation_count(&session.annotations, expected_count);
    }

    // Step 3: Return reference for further use
    session
}

/// Assert element is a verbatim block and validate specified properties.
///
/// Follows the same pattern as `assert_paragraph()` but adapted for verbatim-specific properties.
///
/// # Arguments
///
/// * `element` - The SessionContainerElement to validate
/// * `expected` - VerbatimExpected with properties to validate
///
/// # Returns
///
/// Returns `&VerbatimBlock` reference for further validation if needed.
///
/// # Panics
///
/// Panics if:
/// - Element is not a VerbatimBlock
/// - Any specified property doesn't match expected value
///
/// # Examples
///
/// ```rust
/// use tests::assertions::{assert_verbatim, VerbatimExpected};
/// use txxt::ast::elements::verbatim::VerbatimType;
///
/// // Minimal validation
/// assert_verbatim(&element, VerbatimExpected {
///     label: Some("python"),
///     ..Default::default()
/// });
///
/// // Comprehensive validation
/// assert_verbatim(&element, VerbatimExpected {
///     mode: Some(VerbatimType::InFlow),
///     label: Some("sql"),
///     title: Some("Database schema"),
///     ..Default::default()
/// });
/// ```
#[cfg(feature = "new-ast")]
#[allow(dead_code)] // Used in verbatim_parser_tests with new-ast feature
pub fn assert_verbatim<'a>(
    element: &'a SessionContainerElement,
    expected: VerbatimExpected<'a>,
) -> &'a txxt::ast::elements::verbatim::VerbatimBlock {
    // Step 1: Type check and downcast
    let verbatim = match element {
        SessionContainerElement::Verbatim(v) => v,
        other => {
            panic!(
                "Element type assertion failed\n\
                 Expected: VerbatimBlock\n\
                 Actual: {:?}\n\
                 Hint: Ensure you're extracting the correct element from the container",
                element_type_name(other)
            );
        }
    };

    // Step 2: Validate each specified property

    // Mode validation
    if let Some(expected_mode) = expected.mode {
        assert_eq!(
            verbatim.verbatim_type, expected_mode,
            "Verbatim mode mismatch\n\
             Expected: {:?}\n\
             Actual: {:?}",
            expected_mode, verbatim.verbatim_type
        );
    }

    // Label exact match validation
    if let Some(expected_label) = expected.label {
        assert_eq!(
            verbatim.label.trim(),
            expected_label.trim(),
            "Label mismatch\n\
             Expected: '{}'\n\
             Actual: '{}'",
            expected_label,
            verbatim.label
        );
    }

    // Label starts with validation
    if let Some(prefix) = expected.label_starts_with {
        assert!(
            verbatim.label.starts_with(prefix),
            "Label prefix validation failed\n\
             Expected to start with: '{}'\n\
             Actual label: '{}'",
            prefix,
            verbatim.label
        );
    }

    // Title exact match validation
    if let Some(expected_title) = expected.title {
        let actual_title = verbatim.title_text();
        assert_eq!(
            actual_title.trim(),
            expected_title.trim(),
            "Title mismatch\n\
             Expected: '{}'\n\
             Actual: '{}'",
            expected_title,
            actual_title
        );
    }

    // Title contains validation
    if let Some(needle) = expected.title_contains {
        let actual_title = verbatim.title_text();
        assert!(
            actual_title.contains(needle),
            "Title substring validation failed\n\
             Expected to contain: '{}'\n\
             Actual title: '{}'",
            needle,
            actual_title
        );
    }

    // Line count validation
    if let Some(expected_count) = expected.line_count {
        let actual_count = verbatim.content.total_lines();
        assert_eq!(
            actual_count, expected_count,
            "Verbatim content line count mismatch\n\
             Expected: {} lines\n\
             Actual: {} lines",
            expected_count, actual_count
        );
    }

    // Content text validation (substring)
    if let Some(needle) = expected.content_contains {
        let actual_content = verbatim.content_text();
        assert!(
            actual_content.contains(needle),
            "Verbatim content validation failed\n\
             Expected to contain: '{}'\n\
             Actual content: '{}'",
            needle,
            actual_content
        );
    }

    // Annotation count validation
    if let Some(expected_count) = expected.annotation_count {
        validate_annotation_count(&verbatim.annotations, expected_count);
    }

    // Step 3: Return reference for further use
    verbatim
}

/// Assert element is an annotation (to be fully implemented).
///
/// **Template for implementation:** Copy `assert_paragraph()` and adapt.
#[cfg(feature = "new-ast")]
#[allow(dead_code)]
pub fn assert_annotation<'a>(
    _element: &'a SessionContainerElement,
    _expected: AnnotationExpected<'a>,
) -> &'a txxt::ast::elements::annotation::AnnotationBlock {
    todo!("Implement during annotation parsing")
}

// ============================================================================
// Helpers
// ============================================================================

/// Get element type name for error messages.
#[cfg(feature = "new-ast")]
fn element_type_name(element: &SessionContainerElement) -> &'static str {
    match element {
        SessionContainerElement::Paragraph(_) => "ParagraphBlock",
        SessionContainerElement::List(_) => "ListBlock",
        SessionContainerElement::Definition(_) => "DefinitionBlock",
        SessionContainerElement::Session(_) => "SessionBlock",
        SessionContainerElement::Verbatim(_) => "VerbatimBlock",
        SessionContainerElement::Annotation(_) => "AnnotationBlock",
        SessionContainerElement::BlankLine(_) => "BlankLine",
        SessionContainerElement::ContentContainer(_) => "ContentContainer",
        SessionContainerElement::SessionContainer(_) => "SessionContainer",
    }
}
