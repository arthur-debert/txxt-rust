//! Annotation Processing Functions
//!
//! Pure functions for processing and analyzing annotation elements.
//! These extracted functions improve testability and maintainability
//! per the progressive-quality-improvements plan (Phase 2, section 3.8).
//!
//! See: docs/proposals/progressive-quality-improvements.txxt
//! See: docs/specs/elements/annotation/annotation.txxt
//!
//! ## Annotation Structure
//!
//! Annotations use the `::` marker syntax:
//! - Simple: `:: label :: content`
//! - Parameterized: `:: label:param=value :: content`
//! - Block form: `:: label :\n    indented content`
//!
//! ## Label Namespaces
//!
//! Labels can use dot-separated namespaces:
//! - `def.term` - Definition namespace
//! - `org.example.custom` - Organization namespace
//! - Language markers: `rust`, `python`, `javascript` (verbatim indicators)

use crate::cst::ScannerToken;

/// Common programming language identifiers that indicate verbatim code blocks
const VERBATIM_LANGUAGE_MARKERS: &[&str] = &[
    "rust",
    "python",
    "javascript",
    "typescript",
    "java",
    "c",
    "cpp",
    "csharp",
    "go",
    "ruby",
    "php",
    "swift",
    "kotlin",
    "scala",
    "haskell",
    "ocaml",
    "lisp",
    "scheme",
    "clojure",
    "elixir",
    "erlang",
    "r",
    "julia",
    "matlab",
    "bash",
    "shell",
    "sh",
    "zsh",
    "powershell",
    "sql",
    "html",
    "css",
    "xml",
    "json",
    "yaml",
    "toml",
    "markdown",
    "latex",
    "tex",
    "code",
    "example",
    "output",
    "terminal",
    "console",
];

/// Check if a label indicates verbatim content
///
/// Verbatim annotations are typically programming language names or
/// keywords that indicate code blocks or technical content.
///
/// # Arguments
/// * `label` - The annotation label to check
///
/// # Returns
/// * `true` if label indicates verbatim content, `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert!(is_verbatim_annotation("rust"));
/// assert!(is_verbatim_annotation("python"));
/// assert!(is_verbatim_annotation("code"));
/// assert!(!is_verbatim_annotation("note"));
/// assert!(!is_verbatim_annotation("title"));
/// ```
pub fn is_verbatim_annotation(label: &str) -> bool {
    let label_lower = label.to_lowercase();
    VERBATIM_LANGUAGE_MARKERS.contains(&label_lower.as_str())
}

/// Check if a label indicates a definition annotation
///
/// Definition annotations use the `def.*` namespace to indicate
/// structured definitions or term explanations.
///
/// # Arguments
/// * `label` - The annotation label to check
///
/// # Returns
/// * `true` if label starts with "def.", `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert!(is_definition_annotation("def.term"));
/// assert!(is_definition_annotation("def.concept"));
/// assert!(!is_definition_annotation("definition"));
/// assert!(!is_definition_annotation("rust"));
/// ```
pub fn is_definition_annotation(label: &str) -> bool {
    label.starts_with("def.")
}

/// Check if a label uses a namespace
///
/// Namespaced labels contain periods that separate namespace components.
///
/// # Arguments
/// * `label` - The annotation label to check
///
/// # Returns
/// * `true` if label contains a period (is namespaced), `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert!(is_namespaced_label("def.term"));
/// assert!(is_namespaced_label("org.example.custom"));
/// assert!(!is_namespaced_label("rust"));
/// assert!(!is_namespaced_label("note"));
/// ```
pub fn is_namespaced_label(label: &str) -> bool {
    label.contains('.')
}

/// Extract the namespace prefix from a label
///
/// Returns the part before the last period in a namespaced label.
///
/// # Arguments
/// * `label` - The annotation label
///
/// # Returns
/// * `Some(namespace)` if label is namespaced, `None` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert_eq!(extract_label_namespace("def.term"), Some("def".to_string()));
/// assert_eq!(extract_label_namespace("org.example.custom"), Some("org.example".to_string()));
/// assert_eq!(extract_label_namespace("rust"), None);
/// ```
pub fn extract_label_namespace(label: &str) -> Option<String> {
    label.rfind('.').map(|pos| label[..pos].to_string())
}

/// Extract the base name from a label
///
/// Returns the part after the last period in a namespaced label,
/// or the entire label if not namespaced.
///
/// # Arguments
/// * `label` - The annotation label
///
/// # Returns
/// * The base name (after last period, or entire label)
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert_eq!(extract_label_basename("def.term"), "term");
/// assert_eq!(extract_label_basename("org.example.custom"), "custom");
/// assert_eq!(extract_label_basename("rust"), "rust");
/// ```
pub fn extract_label_basename(label: &str) -> &str {
    label
        .rfind('.')
        .map(|pos| &label[pos + 1..])
        .unwrap_or(label)
}

/// Extract label text from scanner tokens
///
/// Concatenates text content from a sequence of scanner tokens,
/// typically used to extract the label portion of an annotation.
///
/// # Arguments
/// * `tokens` - Scanner tokens containing the label
///
/// # Returns
/// * The concatenated label text
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// # use txxt::cst::{ScannerToken, SourceSpan, Position};
/// let tokens = vec![
///     ScannerToken::Text {
///         content: "my".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 0 },
///             end: Position { row: 0, column: 2 },
///         },
///     },
///     ScannerToken::Text {
///         content: "label".to_string(),
///         span: SourceSpan {
///             start: Position { row: 0, column: 2 },
///             end: Position { row: 0, column: 7 },
///         },
///     },
/// ];
/// assert_eq!(extract_label_from_tokens(&tokens), "mylabel");
/// ```
pub fn extract_label_from_tokens(tokens: &[ScannerToken]) -> String {
    let mut label = String::new();
    for token in tokens {
        match token {
            ScannerToken::Text { content, .. }
            | ScannerToken::Identifier { content, .. }
            | ScannerToken::QuotedString { content, .. } => {
                label.push_str(content);
            }
            ScannerToken::Colon { .. } => {
                label.push(':');
            }
            ScannerToken::Equals { .. } => {
                label.push('=');
            }
            _ => {} // Skip other token types (whitespace, markers, etc.)
        }
    }
    label
}

/// Split label text into base label and parameters
///
/// Separates a label string at the first colon, dividing it into
/// the base label part and the parameters part.
///
/// # Arguments
/// * `label_raw` - The raw label string (may include parameters)
///
/// # Returns
/// * `(base_label, parameters_str)` - Tuple of label and parameter string
///   Parameters string is empty if no colon found.
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert_eq!(
///     split_label_and_parameters("warning:severity=high"),
///     ("warning".to_string(), "severity=high".to_string())
/// );
/// assert_eq!(
///     split_label_and_parameters("note"),
///     ("note".to_string(), String::new())
/// );
/// ```
pub fn split_label_and_parameters(label_raw: &str) -> (String, String) {
    if let Some(colon_pos) = label_raw.find(':') {
        let label = label_raw[..colon_pos].to_string();
        let params = label_raw[colon_pos + 1..].to_string();
        (label, params)
    } else {
        (label_raw.to_string(), String::new())
    }
}

/// Check if a label is a document metadata annotation
///
/// Document metadata annotations include common document properties
/// like title, author, date, etc.
///
/// # Arguments
/// * `label` - The annotation label to check
///
/// # Returns
/// * `true` if label is a common document metadata field, `false` otherwise
///
/// # Examples
/// ```
/// # use txxt::syntax::annotation_processing::*;
/// assert!(is_document_metadata("title"));
/// assert!(is_document_metadata("author"));
/// assert!(is_document_metadata("pub-date"));
/// assert!(!is_document_metadata("note"));
/// assert!(!is_document_metadata("warning"));
/// ```
pub fn is_document_metadata(label: &str) -> bool {
    matches!(
        label,
        "title"
            | "author"
            | "date"
            | "pub-date"
            | "version"
            | "copyright"
            | "license"
            | "abstract"
            | "keywords"
            | "description"
            | "bibliography"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::{Position, SourceSpan};

    #[test]
    fn test_is_verbatim_annotation_languages() {
        assert!(is_verbatim_annotation("rust"));
        assert!(is_verbatim_annotation("python"));
        assert!(is_verbatim_annotation("javascript"));
        assert!(is_verbatim_annotation("code"));
    }

    #[test]
    fn test_is_verbatim_annotation_case_insensitive() {
        assert!(is_verbatim_annotation("RUST"));
        assert!(is_verbatim_annotation("Python"));
        assert!(is_verbatim_annotation("JavaScript"));
    }

    #[test]
    fn test_is_verbatim_annotation_non_languages() {
        assert!(!is_verbatim_annotation("note"));
        assert!(!is_verbatim_annotation("title"));
        assert!(!is_verbatim_annotation("warning"));
        assert!(!is_verbatim_annotation("def.term"));
    }

    #[test]
    fn test_is_definition_annotation() {
        assert!(is_definition_annotation("def.term"));
        assert!(is_definition_annotation("def.concept"));
        assert!(is_definition_annotation("def.example"));
        assert!(!is_definition_annotation("definition"));
        assert!(!is_definition_annotation("rust"));
    }

    #[test]
    fn test_is_namespaced_label() {
        assert!(is_namespaced_label("def.term"));
        assert!(is_namespaced_label("org.example.custom"));
        assert!(!is_namespaced_label("rust"));
        assert!(!is_namespaced_label("note"));
    }

    #[test]
    fn test_extract_label_namespace() {
        assert_eq!(extract_label_namespace("def.term"), Some("def".to_string()));
        assert_eq!(
            extract_label_namespace("org.example.custom"),
            Some("org.example".to_string())
        );
        assert_eq!(extract_label_namespace("rust"), None);
    }

    #[test]
    fn test_extract_label_basename() {
        assert_eq!(extract_label_basename("def.term"), "term");
        assert_eq!(extract_label_basename("org.example.custom"), "custom");
        assert_eq!(extract_label_basename("rust"), "rust");
    }

    #[test]
    fn test_extract_label_from_tokens() {
        let tokens = vec![
            ScannerToken::Text {
                content: "my".to_string(),
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
        assert_eq!(extract_label_from_tokens(&tokens), "mylabel");
    }

    #[test]
    fn test_extract_label_from_tokens_with_colon() {
        let tokens = vec![
            ScannerToken::Text {
                content: "warning".to_string(),
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
            ScannerToken::Text {
                content: "severity".to_string(),
                span: SourceSpan {
                    start: Position { row: 0, column: 8 },
                    end: Position { row: 0, column: 16 },
                },
            },
        ];
        assert_eq!(extract_label_from_tokens(&tokens), "warning:severity");
    }

    #[test]
    fn test_split_label_and_parameters() {
        assert_eq!(
            split_label_and_parameters("warning:severity=high"),
            ("warning".to_string(), "severity=high".to_string())
        );
        assert_eq!(
            split_label_and_parameters("note"),
            ("note".to_string(), String::new())
        );
        assert_eq!(
            split_label_and_parameters("meta:version=2.0,author=Jane"),
            ("meta".to_string(), "version=2.0,author=Jane".to_string())
        );
    }

    #[test]
    fn test_is_document_metadata() {
        assert!(is_document_metadata("title"));
        assert!(is_document_metadata("author"));
        assert!(is_document_metadata("pub-date"));
        assert!(is_document_metadata("version"));
        assert!(!is_document_metadata("note"));
        assert!(!is_document_metadata("warning"));
    }
}
