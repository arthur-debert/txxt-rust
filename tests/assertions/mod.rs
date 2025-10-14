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

/// Assert element is a list (to be fully implemented).
///
/// **Template for implementation:** Copy `assert_paragraph()` and adapt.
#[cfg(feature = "new-ast")]
#[allow(dead_code)]
pub fn assert_list<'a>(
    _element: &'a SessionContainerElement,
    _expected: ListExpected<'a>,
) -> &'a txxt::ast::elements::list::ListBlock {
    todo!("Implement during Parser 2.1.2 - Lists")
}

/// Assert element is a definition (to be fully implemented).
///
/// **Template for implementation:** Copy `assert_paragraph()` and adapt.
#[cfg(feature = "new-ast")]
#[allow(dead_code)]
pub fn assert_definition<'a>(
    _element: &'a SessionContainerElement,
    _expected: DefinitionExpected<'a>,
) -> &'a txxt::ast::elements::definition::DefinitionBlock {
    todo!("Implement during Parser 2.1.3 - Definitions")
}

/// Assert element is a session (to be fully implemented).
///
/// **Template for implementation:** Copy `assert_paragraph()` and adapt.
#[cfg(feature = "new-ast")]
#[allow(dead_code)]
pub fn assert_session<'a>(
    _element: &'a SessionContainerElement,
    _expected: SessionExpected<'a>,
) -> &'a txxt::ast::elements::session::SessionBlock {
    todo!("Implement during Parser 2.1.5 - Sessions")
}

/// Assert element is a verbatim block (to be fully implemented).
///
/// **Template for implementation:** Copy `assert_paragraph()` and adapt.
#[cfg(feature = "new-ast")]
#[allow(dead_code)]
pub fn assert_verbatim<'a>(
    _element: &'a SessionContainerElement,
    _expected: VerbatimExpected<'a>,
) -> &'a txxt::ast::elements::verbatim::VerbatimBlock {
    todo!("Implement during verbatim parsing")
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
