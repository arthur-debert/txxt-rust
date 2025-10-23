//! List Detection and Analysis Functions
//!
//! Pure functions for detecting and analyzing list sequence markers.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan (Phase 2, section 3.7).
//!
//! See: docs/proposals/progressive-quality-improvements.txxt
//! See: docs/specs/elements/list/list.txxt
//!
//! ## Sequence Marker Types
//!
//! Lists support four marker styles:
//! - Plain: `-` (dash marker)
//! - Numerical: `1.`, `2.`, `42.` (numeric markers)
//! - Alphabetical: `a.`, `b.`, `A.`, `B.` (letter markers)
//! - Roman: `i.`, `ii.`, `I.`, `II.` (roman numeral markers)
//!
//! ## Numbering Forms
//!
//! - Regular: Simple markers like `1.`, `2.`, `3.`
//! - Extended: Hierarchical markers like `1.1.`, `1.2.`, `2.1.`

use crate::cst::SequenceMarkerType;

/// Numbering style for list markers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberingStyle {
    /// Plain dash marker (-)
    Plain,
    /// Numeric markers (1., 2., 3.)
    Numerical,
    /// Alphabetic markers (a., b., c.)
    Alphabetical,
    /// Roman numeral markers (i., ii., iii.)
    Roman,
}

/// Numbering form for list markers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberingForm {
    /// Regular form (1., 2., 3.)
    Regular,
    /// Extended hierarchical form (1.1., 1.2., 2.1.)
    Extended,
}

/// List decoration type combining style and form
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListDecorationType {
    /// The numbering style (plain, numerical, alphabetical, roman)
    pub style: NumberingStyle,
    /// The numbering form (regular, extended)
    pub form: NumberingForm,
}

impl Default for ListDecorationType {
    fn default() -> Self {
        Self {
            style: NumberingStyle::Plain,
            form: NumberingForm::Regular,
        }
    }
}

/// Detect the sequence marker type from a scanner token
///
/// Classifies scanner-level SequenceMarkerType into high-level style and form.
///
/// # Arguments
/// * `marker_type` - Scanner token sequence marker type
///
/// # Returns
/// * `(NumberingStyle, NumberingForm)` - The classified style and form
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// # use txxt::cst::SequenceMarkerType;
/// let (style, form) = classify_sequence_marker(&SequenceMarkerType::Plain("-".to_string()));
/// assert_eq!(style, NumberingStyle::Plain);
/// assert_eq!(form, NumberingForm::Regular);
///
/// let (style, form) = classify_sequence_marker(&SequenceMarkerType::Numerical(1, "1.".to_string()));
/// assert_eq!(style, NumberingStyle::Numerical);
/// assert_eq!(form, NumberingForm::Regular);
/// ```
pub fn classify_sequence_marker(
    marker_type: &SequenceMarkerType,
) -> (NumberingStyle, NumberingForm) {
    match marker_type {
        SequenceMarkerType::Plain(_) => (NumberingStyle::Plain, NumberingForm::Regular),
        SequenceMarkerType::Numerical(_, _) => (NumberingStyle::Numerical, NumberingForm::Regular),
        SequenceMarkerType::Alphabetical(_, _) => {
            (NumberingStyle::Alphabetical, NumberingForm::Regular)
        }
        SequenceMarkerType::Roman(_, _) => (NumberingStyle::Roman, NumberingForm::Regular),
    }
}

/// Determine list decoration type from marker string
///
/// Analyzes a marker string to determine its numbering style and form.
/// This is used when creating list AST nodes from marker text.
///
/// # Arguments
/// * `marker` - The marker string (e.g., "-", "1.", "a.", "i.")
///
/// # Returns
/// * `ListDecorationType` - The determined decoration type
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// let decoration = determine_decoration_type("-");
/// assert_eq!(decoration.style, NumberingStyle::Plain);
/// assert_eq!(decoration.form, NumberingForm::Regular);
///
/// let decoration = determine_decoration_type("1.");
/// assert_eq!(decoration.style, NumberingStyle::Numerical);
///
/// let decoration = determine_decoration_type("a.");
/// assert_eq!(decoration.style, NumberingStyle::Alphabetical);
///
/// let decoration = determine_decoration_type("i.");
/// assert_eq!(decoration.style, NumberingStyle::Alphabetical); // Note: single 'i' detected as alpha
/// ```
pub fn determine_decoration_type(marker: &str) -> ListDecorationType {
    let style = if marker.starts_with('-') {
        NumberingStyle::Plain
    } else if marker.chars().next().is_some_and(|c| c.is_numeric()) {
        NumberingStyle::Numerical
    } else if marker.chars().next().is_some_and(|c| c.is_alphabetic()) {
        // This will classify both alphabetical and roman markers as Alphabetical
        // Roman detection requires more context (e.g., "ii.", "iii." vs "a.", "b.")
        NumberingStyle::Alphabetical
    } else {
        NumberingStyle::Plain // fallback
    };

    // Determine form based on marker structure
    let form = if marker.contains('.') && marker.matches('.').count() > 1 {
        // Extended form: has multiple periods like "1.1." or "2.1.3."
        NumberingForm::Extended
    } else {
        NumberingForm::Regular
    };

    ListDecorationType { style, form }
}

/// Check if a marker string represents a plain (dash) marker
///
/// # Arguments
/// * `marker` - The marker string to check
///
/// # Returns
/// * `true` if marker starts with '-', `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// assert!(is_plain_marker("-"));
/// assert!(is_plain_marker("- "));
/// assert!(!is_plain_marker("1."));
/// assert!(!is_plain_marker("a."));
/// ```
pub fn is_plain_marker(marker: &str) -> bool {
    marker.starts_with('-')
}

/// Check if a marker string represents a numerical marker
///
/// # Arguments
/// * `marker` - The marker string to check
///
/// # Returns
/// * `true` if marker starts with a digit, `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// assert!(is_numerical_marker("1."));
/// assert!(is_numerical_marker("42)"));
/// assert!(is_numerical_marker("1.1."));
/// assert!(!is_numerical_marker("-"));
/// assert!(!is_numerical_marker("a."));
/// ```
pub fn is_numerical_marker(marker: &str) -> bool {
    marker.chars().next().is_some_and(|c| c.is_numeric())
}

/// Check if a marker string represents an alphabetical marker
///
/// # Arguments
/// * `marker` - The marker string to check
///
/// # Returns
/// * `true` if marker starts with a letter, `false` otherwise
///
/// # Note
/// This will also match roman numeral markers, as they are letters.
/// Use `is_roman_marker()` for specific roman numeral detection.
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// assert!(is_alphabetical_marker("a."));
/// assert!(is_alphabetical_marker("Z)"));
/// assert!(is_alphabetical_marker("i.")); // Also matches roman
/// assert!(!is_alphabetical_marker("1."));
/// assert!(!is_alphabetical_marker("-"));
/// ```
pub fn is_alphabetical_marker(marker: &str) -> bool {
    marker.chars().next().is_some_and(|c| c.is_alphabetic())
}

/// Check if a marker string represents a roman numeral marker
///
/// Detects common roman numeral patterns (i-xiii, I-XIII).
///
/// # Arguments
/// * `marker` - The marker string to check
///
/// # Returns
/// * `true` if marker matches roman numeral pattern, `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// assert!(is_roman_marker("i."));
/// assert!(is_roman_marker("ii)"));
/// assert!(is_roman_marker("iii."));
/// assert!(is_roman_marker("IV."));
/// assert!(is_roman_marker("XIII)"));
/// assert!(!is_roman_marker("a.")); // Single letter 'a' is not roman
/// assert!(!is_roman_marker("z.")); // 'z' is not roman
/// assert!(!is_roman_marker("1."));
/// ```
pub fn is_roman_marker(marker: &str) -> bool {
    let roman_patterns = [
        "xiii", "xii", "xi", "viii", "vii", "iii", "ii", "iv", "vi", "ix", "i", "v", "x", "XIII",
        "XII", "XI", "VIII", "VII", "III", "II", "IV", "VI", "IX", "I", "V", "X",
    ];

    for pattern in &roman_patterns {
        if let Some(after_pattern) = marker.strip_prefix(pattern) {
            // Check that the pattern is followed by . or ) to ensure it's a marker
            if after_pattern.starts_with('.') || after_pattern.starts_with(')') {
                return true;
            }
        }
    }

    false
}

/// Determine if a marker uses extended (hierarchical) form
///
/// Extended form markers contain multiple periods, like "1.1." or "2.1.3."
///
/// # Arguments
/// * `marker` - The marker string to check
///
/// # Returns
/// * `true` if marker has multiple periods, `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// assert!(is_extended_form("1.1."));
/// assert!(is_extended_form("2.1.3."));
/// assert!(!is_extended_form("1."));
/// assert!(!is_extended_form("a."));
/// assert!(!is_extended_form("-"));
/// ```
pub fn is_extended_form(marker: &str) -> bool {
    marker.contains('.') && marker.matches('.').count() > 1
}

/// Infer list style from multiple markers
///
/// Determines the overall list style by examining all markers.
/// The first marker determines the primary style (per spec).
///
/// # Arguments
/// * `markers` - Slice of marker strings
///
/// # Returns
/// * `NumberingStyle` - The inferred style (based on first marker)
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// let markers = vec!["-".to_string(), "-".to_string(), "-".to_string()];
/// assert_eq!(infer_list_style(&markers), NumberingStyle::Plain);
///
/// let markers = vec!["1.".to_string(), "2.".to_string(), "3.".to_string()];
/// assert_eq!(infer_list_style(&markers), NumberingStyle::Numerical);
///
/// let markers = vec!["1.".to_string(), "a.".to_string(), "b.".to_string()];
/// assert_eq!(infer_list_style(&markers), NumberingStyle::Numerical); // First marker wins
/// ```
pub fn infer_list_style(markers: &[String]) -> NumberingStyle {
    if markers.is_empty() {
        return NumberingStyle::Plain;
    }

    // First marker determines the style per specification
    let decoration = determine_decoration_type(&markers[0]);
    decoration.style
}

/// Calculate indentation for wrapped sequence content
///
/// Computes the indentation level for content that wraps after a sequence marker.
/// This is typically the marker end position plus one space.
///
/// # Arguments
/// * `_marker_end` - Column position where marker ends (reserved for future use)
/// * `content_start` - Column position where content starts
///
/// # Returns
/// * `usize` - The indentation level for wrapped content
///
/// # Examples
/// ```
/// # use txxt::syntax::list_detection::*;
/// // Marker "1. " ends at column 3, content starts at 3
/// assert_eq!(calculate_sequence_indent(3, 3), 3);
///
/// // Marker "- " ends at column 2, content starts at 2
/// assert_eq!(calculate_sequence_indent(2, 2), 2);
/// ```
pub fn calculate_sequence_indent(_marker_end: usize, content_start: usize) -> usize {
    // The indent for wrapped content is where the content starts
    content_start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_sequence_marker_plain() {
        let (style, form) = classify_sequence_marker(&SequenceMarkerType::Plain("-".to_string()));
        assert_eq!(style, NumberingStyle::Plain);
        assert_eq!(form, NumberingForm::Regular);
    }

    #[test]
    fn test_classify_sequence_marker_numerical() {
        let (style, form) =
            classify_sequence_marker(&SequenceMarkerType::Numerical(1, "1.".to_string()));
        assert_eq!(style, NumberingStyle::Numerical);
        assert_eq!(form, NumberingForm::Regular);
    }

    #[test]
    fn test_classify_sequence_marker_alphabetical() {
        let (style, form) =
            classify_sequence_marker(&SequenceMarkerType::Alphabetical('a', "a.".to_string()));
        assert_eq!(style, NumberingStyle::Alphabetical);
        assert_eq!(form, NumberingForm::Regular);
    }

    #[test]
    fn test_classify_sequence_marker_roman() {
        let (style, form) =
            classify_sequence_marker(&SequenceMarkerType::Roman(1, "i.".to_string()));
        assert_eq!(style, NumberingStyle::Roman);
        assert_eq!(form, NumberingForm::Regular);
    }

    #[test]
    fn test_determine_decoration_type_plain() {
        let decoration = determine_decoration_type("-");
        assert_eq!(decoration.style, NumberingStyle::Plain);
        assert_eq!(decoration.form, NumberingForm::Regular);
    }

    #[test]
    fn test_determine_decoration_type_numerical() {
        let decoration = determine_decoration_type("1.");
        assert_eq!(decoration.style, NumberingStyle::Numerical);
        assert_eq!(decoration.form, NumberingForm::Regular);
    }

    #[test]
    fn test_determine_decoration_type_alphabetical() {
        let decoration = determine_decoration_type("a.");
        assert_eq!(decoration.style, NumberingStyle::Alphabetical);
        assert_eq!(decoration.form, NumberingForm::Regular);
    }

    #[test]
    fn test_determine_decoration_type_extended_form() {
        let decoration = determine_decoration_type("1.1.");
        assert_eq!(decoration.style, NumberingStyle::Numerical);
        assert_eq!(decoration.form, NumberingForm::Extended);

        let decoration = determine_decoration_type("2.1.3.");
        assert_eq!(decoration.style, NumberingStyle::Numerical);
        assert_eq!(decoration.form, NumberingForm::Extended);
    }

    #[test]
    fn test_is_plain_marker() {
        assert!(is_plain_marker("-"));
        assert!(is_plain_marker("- "));
        assert!(!is_plain_marker("1."));
        assert!(!is_plain_marker("a."));
    }

    #[test]
    fn test_is_numerical_marker() {
        assert!(is_numerical_marker("1."));
        assert!(is_numerical_marker("42)"));
        assert!(is_numerical_marker("1.1."));
        assert!(!is_numerical_marker("-"));
        assert!(!is_numerical_marker("a."));
    }

    #[test]
    fn test_is_alphabetical_marker() {
        assert!(is_alphabetical_marker("a."));
        assert!(is_alphabetical_marker("Z)"));
        assert!(is_alphabetical_marker("i.")); // Also matches roman
        assert!(!is_alphabetical_marker("1."));
        assert!(!is_alphabetical_marker("-"));
    }

    #[test]
    fn test_is_roman_marker() {
        assert!(is_roman_marker("i."));
        assert!(is_roman_marker("ii)"));
        assert!(is_roman_marker("iii."));
        assert!(is_roman_marker("IV."));
        assert!(is_roman_marker("XIII)"));
        assert!(!is_roman_marker("a.")); // Single letter 'a' is not roman
        assert!(!is_roman_marker("z.")); // 'z' is not roman
        assert!(!is_roman_marker("1."));
    }

    #[test]
    fn test_is_extended_form() {
        assert!(is_extended_form("1.1."));
        assert!(is_extended_form("2.1.3."));
        assert!(!is_extended_form("1."));
        assert!(!is_extended_form("a."));
        assert!(!is_extended_form("-"));
    }

    #[test]
    fn test_infer_list_style() {
        let markers = vec!["-".to_string(), "-".to_string(), "-".to_string()];
        assert_eq!(infer_list_style(&markers), NumberingStyle::Plain);

        let markers = vec!["1.".to_string(), "2.".to_string(), "3.".to_string()];
        assert_eq!(infer_list_style(&markers), NumberingStyle::Numerical);

        // First marker determines style
        let markers = vec!["1.".to_string(), "a.".to_string(), "b.".to_string()];
        assert_eq!(infer_list_style(&markers), NumberingStyle::Numerical);
    }

    #[test]
    fn test_calculate_sequence_indent() {
        assert_eq!(calculate_sequence_indent(3, 3), 3);
        assert_eq!(calculate_sequence_indent(2, 2), 2);
        assert_eq!(calculate_sequence_indent(5, 5), 5);
    }
}
