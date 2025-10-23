//! Verbatim Boundary Detection Functions
//!
//! Pure functions for detecting and validating verbatim block boundaries.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan (Phase 2, section 3.3).
//!
//! See: docs/proposals/progressive-quality-improvements.txxt
//!
//! ## Verbatim Block Structure
//!
//! Verbatim blocks follow these rules:
//! - Start with a subject line (text ending with colon, no txxt marker)
//! - End with an annotation line at same indentation as subject line
//! - Content must be indented >= indentation wall
//!
//! ## Indentation Modes
//!
//! **Stretched Mode:** Wall at absolute column 1
//! - Subject line at any indentation
//! - First content at column 1 → stretched mode
//! - All content must be at column 1 or deeper
//!
//! **In-Flow Mode:** Wall at subject_indent + 4
//! - Subject line at any indentation
//! - First content at subject_indent + 4 or deeper → in-flow mode
//! - All content must be at wall or deeper

use crate::cst::ScannerToken;

/// Indentation level for stretched mode wall (absolute column 1)
pub const STRETCHED_MODE_WALL: usize = 1;

/// Minimum indentation increase for in-flow mode (4 spaces)
pub const IN_FLOW_MODE_INDENT_INCREASE: usize = 4;

/// Verbatim block mode based on first content line indentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbatimMode {
    /// Content at absolute column 1 (stretched mode)
    Stretched,
    /// Content indented relative to subject line (in-flow mode)
    InFlow,
}

/// Check if indentation level indicates stretched mode
///
/// Stretched mode is when content is at absolute column 1, regardless of
/// subject line indentation.
///
/// # Arguments
/// * `content_indent` - Indentation level of first content line
///
/// # Returns
/// * `true` if content_indent == STRETCHED_MODE_WALL (1)
///
/// # Examples
/// ```
/// # use txxt::syntax::verbatim_boundary::*;
/// assert!(is_stretched_mode(1));  // Column 1 = stretched
/// assert!(!is_stretched_mode(0)); // Column 0 = invalid
/// assert!(!is_stretched_mode(4)); // Column 4+ = in-flow
/// ```
pub fn is_stretched_mode(content_indent: usize) -> bool {
    content_indent == STRETCHED_MODE_WALL
}

/// Determine verbatim mode and wall from first content line
///
/// The first non-blank content line after the subject line determines:
/// - The mode (stretched vs in-flow)
/// - The wall position (where content must be aligned)
///
/// # Arguments
/// * `title_indent` - Indentation of the subject line
/// * `first_content_indent` - Indentation of first non-blank content line
///
/// # Returns
/// * `(VerbatimMode, wall_position)`
///
/// # Examples
/// ```
/// # use txxt::syntax::verbatim_boundary::*;
/// // Stretched mode: title at 0, content at 1
/// let (mode, wall) = determine_verbatim_mode(0, 1);
/// assert_eq!(mode, VerbatimMode::Stretched);
/// assert_eq!(wall, 1);
///
/// // In-flow mode: title at 0, content at 4
/// let (mode, wall) = determine_verbatim_mode(0, 4);
/// assert_eq!(mode, VerbatimMode::InFlow);
/// assert_eq!(wall, 4);
///
/// // In-flow mode: title at 8, content at 12
/// let (mode, wall) = determine_verbatim_mode(8, 12);
/// assert_eq!(mode, VerbatimMode::InFlow);
/// assert_eq!(wall, 12);
/// ```
pub fn determine_verbatim_mode(
    _title_indent: usize,
    first_content_indent: usize,
) -> (VerbatimMode, usize) {
    if is_stretched_mode(first_content_indent) {
        (VerbatimMode::Stretched, STRETCHED_MODE_WALL)
    } else {
        (VerbatimMode::InFlow, first_content_indent)
    }
}

/// Validate verbatim content line indentation
///
/// Content lines must be at or deeper than the wall position.
/// The wall is determined by the mode and first content line.
///
/// # Arguments
/// * `content_indent` - Indentation of the content line to validate
/// * `wall` - The wall position (from determine_verbatim_mode)
/// * `mode` - The verbatim mode
///
/// # Returns
/// * `Ok(())` if content_indent >= wall
/// * `Err(String)` with error message if invalid
///
/// # Examples
/// ```
/// # use txxt::syntax::verbatim_boundary::*;
/// // Stretched mode: wall at 1
/// assert!(validate_verbatim_content_indent(1, 1, VerbatimMode::Stretched).is_ok());
/// assert!(validate_verbatim_content_indent(5, 1, VerbatimMode::Stretched).is_ok());
/// assert!(validate_verbatim_content_indent(0, 1, VerbatimMode::Stretched).is_err());
///
/// // In-flow mode: wall at 4
/// assert!(validate_verbatim_content_indent(4, 4, VerbatimMode::InFlow).is_ok());
/// assert!(validate_verbatim_content_indent(8, 4, VerbatimMode::InFlow).is_ok());
/// assert!(validate_verbatim_content_indent(2, 4, VerbatimMode::InFlow).is_err());
/// ```
pub fn validate_verbatim_content_indent(
    content_indent: usize,
    wall: usize,
    mode: VerbatimMode,
) -> Result<(), String> {
    if content_indent >= wall {
        Ok(())
    } else {
        Err(format!(
            "Invalid verbatim content indentation: expected >= {} ({:?} mode, wall={}), got {}",
            wall, mode, wall, content_indent
        ))
    }
}

/// Check if indentation is valid for in-flow mode
///
/// In-flow mode requires content to be at least title_indent + 4.
///
/// # Arguments
/// * `title_indent` - Indentation of the subject line
/// * `content_indent` - Indentation of the content line
///
/// # Returns
/// * `true` if content_indent >= title_indent + IN_FLOW_MODE_INDENT_INCREASE
///
/// # Examples
/// ```
/// # use txxt::syntax::verbatim_boundary::*;
/// assert!(is_valid_inflow_indent(0, 4));  // 0 + 4 = 4 ✓
/// assert!(is_valid_inflow_indent(0, 8));  // 0 + 8 = 8 ✓
/// assert!(is_valid_inflow_indent(4, 8));  // 4 + 4 = 8 ✓
/// assert!(!is_valid_inflow_indent(0, 3)); // 0 + 3 < 4 ✗
/// assert!(!is_valid_inflow_indent(4, 7)); // 4 + 3 < 8 ✗
/// ```
pub fn is_valid_inflow_indent(title_indent: usize, content_indent: usize) -> bool {
    content_indent >= title_indent + IN_FLOW_MODE_INDENT_INCREASE
}

/// Check if a line matches verbatim subject line pattern
///
/// A verbatim subject line:
/// - Does NOT start with a txxt marker (`::`)
/// - Ends with a single colon (`:`)
/// - Is followed by indented content (checked separately)
///
/// # Arguments
/// * `tokens` - Scanner tokens for the line
///
/// # Returns
/// * `true` if line matches verbatim subject pattern
///
/// # Examples
/// ```
/// # use txxt::syntax::verbatim_boundary::*;
/// # use txxt::cst::{ScannerToken, SourceSpan, Position};
/// let subject_line = vec![
///     ScannerToken::Text {
///         content: "example".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 0 },
///             end: Position { row: 0, column: 7 },
///         },
///     },
///     ScannerToken::Colon {
///         span: SourceSpan {
///             start: Position { row: 0, column: 7 },
///             end: Position { row: 0, column: 8 },
///         },
///     },
/// ];
/// assert!(is_verbatim_subject_line(&subject_line));
/// ```
pub fn is_verbatim_subject_line(tokens: &[ScannerToken]) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // Must not start with TxxtMarker (that would be an annotation)
    if matches!(tokens[0], ScannerToken::TxxtMarker { .. }) {
        return false;
    }

    // Must end with single colon (checked by is_definition_marker in line_classification)
    // For now, just check that last non-whitespace token is Colon
    let mut has_trailing_colon = false;
    for token in tokens.iter().rev() {
        match token {
            ScannerToken::Whitespace { .. } | ScannerToken::Newline { .. } => continue,
            ScannerToken::Colon { .. } => {
                has_trailing_colon = true;
                break;
            }
            _ => break,
        }
    }

    has_trailing_colon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    #[test]
    fn test_is_stretched_mode() {
        assert!(is_stretched_mode(1)); // Column 1 = stretched
        assert!(!is_stretched_mode(0)); // Column 0 = invalid
        assert!(!is_stretched_mode(4)); // Column 4 = in-flow
        assert!(!is_stretched_mode(12)); // Column 12 = in-flow
    }

    #[test]
    fn test_determine_verbatim_mode_stretched() {
        // Title at 0, content at 1 → stretched mode
        let (mode, wall) = determine_verbatim_mode(0, 1);
        assert_eq!(mode, VerbatimMode::Stretched);
        assert_eq!(wall, 1);

        // Title at 12, content at 1 → still stretched mode
        let (mode, wall) = determine_verbatim_mode(12, 1);
        assert_eq!(mode, VerbatimMode::Stretched);
        assert_eq!(wall, 1);
    }

    #[test]
    fn test_determine_verbatim_mode_inflow() {
        // Title at 0, content at 4 → in-flow mode
        let (mode, wall) = determine_verbatim_mode(0, 4);
        assert_eq!(mode, VerbatimMode::InFlow);
        assert_eq!(wall, 4);

        // Title at 8, content at 12 → in-flow mode
        let (mode, wall) = determine_verbatim_mode(8, 12);
        assert_eq!(mode, VerbatimMode::InFlow);
        assert_eq!(wall, 12);
    }

    #[test]
    fn test_validate_verbatim_content_indent_stretched() {
        // Stretched mode: wall at 1
        assert!(validate_verbatim_content_indent(1, 1, VerbatimMode::Stretched).is_ok());
        assert!(validate_verbatim_content_indent(5, 1, VerbatimMode::Stretched).is_ok());
        assert!(validate_verbatim_content_indent(0, 1, VerbatimMode::Stretched).is_err());
    }

    #[test]
    fn test_validate_verbatim_content_indent_inflow() {
        // In-flow mode: wall at 4
        assert!(validate_verbatim_content_indent(4, 4, VerbatimMode::InFlow).is_ok());
        assert!(validate_verbatim_content_indent(8, 4, VerbatimMode::InFlow).is_ok());
        assert!(validate_verbatim_content_indent(2, 4, VerbatimMode::InFlow).is_err());
    }

    #[test]
    fn test_is_valid_inflow_indent() {
        assert!(is_valid_inflow_indent(0, 4)); // 0 + 4 = 4 ✓
        assert!(is_valid_inflow_indent(0, 8)); // 0 + 8 = 8 ✓
        assert!(is_valid_inflow_indent(4, 8)); // 4 + 4 = 8 ✓
        assert!(!is_valid_inflow_indent(0, 3)); // 0 + 3 < 4 ✗
        assert!(!is_valid_inflow_indent(4, 7)); // 4 + 3 < 8 ✗
    }

    #[test]
    fn test_is_verbatim_subject_line_valid() {
        let subject = vec![
            ScannerToken::Text {
                content: "example".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 7 },
                },
            },
            ScannerToken::Colon {
                span: SourceSpan {
                    start: Position { row: 0, column: 7 },
                    end: Position { row: 0, column: 8 },
                },
            },
        ];
        assert!(is_verbatim_subject_line(&subject));
    }

    #[test]
    fn test_is_verbatim_subject_line_starts_with_marker() {
        // Starts with :: → not a verbatim subject (it's an annotation)
        let annotation = vec![
            ScannerToken::TxxtMarker {
                span: SourceSpan {
                    start: Position { row: 0, column: 0 },
                    end: Position { row: 0, column: 2 },
                },
            },
            ScannerToken::Text {
                content: "label".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 2 },
                    end: Position { row: 0, column: 7 },
                },
            },
        ];
        assert!(!is_verbatim_subject_line(&annotation));
    }

    #[test]
    fn test_is_verbatim_subject_line_no_colon() {
        // No colon → not a verbatim subject
        let no_colon = vec![ScannerToken::Text {
            content: "example".to_string(),
            span: SourceSpan {
                start: Position { row: 0, column: 0 },
                end: Position { row: 0, column: 7 },
            },
        }];
        assert!(!is_verbatim_subject_line(&no_colon));
    }
}
