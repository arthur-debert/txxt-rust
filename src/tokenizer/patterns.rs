//! Centralized regex patterns for token validation
//!
//! This module provides shared pattern constants used by both the tokenizer
//! and test suites to ensure consistency and maintainability.

/// Pattern for valid identifiers: starts with letter, can contain alphanumeric and underscores
/// but underscores can only appear in the middle of word segments
pub const IDENTIFIER_PATTERN: &str = r"[a-zA-Z][a-zA-Z0-9]*(_[a-zA-Z0-9]+)*";

/// Pattern for text tokens: alphanumeric content that may include underscores within words
pub const TEXT_PATTERN: &str = r"[a-zA-Z0-9]+(_[a-zA-Z0-9]+)*";

/// Pattern for reference marker basic validation
///
/// Note: Detailed reference type classification has been moved to
/// ast::reference_types::ReferenceClassifier for better separation of concerns.
/// These patterns are kept for any remaining basic validation needs.
pub mod ref_patterns {
    /// Basic validation: content must have at least one alphanumeric character
    pub const BASIC_CONTENT: &str = r".*[a-zA-Z0-9].*";
}

/// Pattern for annotation marker validation
pub const ANNOTATION_MARKER_PATTERN: &str = r"::";

/// Patterns for sequence markers
pub mod sequence_patterns {
    /// Dash marker: - followed by space
    pub const DASH: &str = r"- ";

    /// Numbered marker: digits followed by . and space
    pub const NUMBERED: &str = r"[0-9]+\. ";

    /// Alphabetical marker: letter followed by . or ) and space
    pub const ALPHABETICAL: &str = r"[a-zA-Z][.)] ";

    /// Roman numeral marker: roman numerals followed by . or ) and space
    pub const ROMAN: &str = r"(?i)(i{1,3}|iv|v|vi{0,3}|ix|x)[.)] ";
}

/// Inline delimiter characters
pub mod inline_delimiters {
    pub const BOLD: char = '*';
    pub const ITALIC: char = '_';
    pub const CODE: char = '`';
    pub const MATH: char = '#';
}

/// Verbatim block markers
pub mod verbatim_patterns {
    /// Start of verbatim block
    pub const START_MARKER: &str = r"```";

    /// End of verbatim block  
    pub const END_MARKER: &str = r"```";

    /// Language identifier pattern after opening ```
    pub const LANGUAGE_ID: &str = r"[a-zA-Z][a-zA-Z0-9_-]*";
}
