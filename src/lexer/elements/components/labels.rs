//! Label element tokenization
//!
//! Implements tokenization for label elements as defined in
//! docs/specs/elements/components/labels.txxt
//!
//! Labels provide identification and metadata for elements.
//! They follow consistent identifier rules and support namespacing.

use crate::cst::{Position, SourceSpan, ScannerToken};

/// Represents a parsed label with namespace support
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    /// Full label string (e.g., "org.example.custom")
    pub value: String,
    /// Namespace segments split by '.' (e.g., ["org", "example", "custom"])
    pub namespaces: Vec<String>,
    /// Whether this label contains namespace separators
    pub is_namespaced: bool,
    /// Source span of the label
    pub span: SourceSpan,
}

/// Result of label parsing
#[derive(Debug, Clone, PartialEq)]
pub enum LabelParseResult {
    /// Valid label was parsed
    Valid(Label),
    /// Invalid label with error message
    Invalid(String),
    /// No label found at current position
    NotFound,
}

/// Validates if a character is valid at the start of a label identifier
///
/// Labels must start with a letter (a-z, A-Z)
pub fn is_valid_label_start(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// Validates if a character is valid within a label identifier
///
/// Valid characters: letters, digits, underscore, dash
/// Period is handled separately for namespace separation
pub fn is_valid_label_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Validates if a character is a namespace separator
pub fn is_namespace_separator(c: char) -> bool {
    c == '.'
}

/// Parses a label from the given input string starting at the given position
///
/// # Arguments
/// * `input` - The input string to parse
/// * `start_pos` - The starting position in the input
///
/// # Returns
/// A `LabelParseResult` indicating success, failure, or no label found
pub fn parse_label(input: &str, start_pos: Position) -> LabelParseResult {
    let chars: Vec<char> = input.chars().collect();
    let mut pos = start_pos.column;

    if pos >= chars.len() {
        return LabelParseResult::NotFound;
    }

    // Check if we start with a valid label character
    if !is_valid_label_start(chars[pos]) {
        return LabelParseResult::NotFound;
    }

    let label_start = pos;
    let mut namespaces = Vec::new();
    let mut current_segment = String::new();
    let mut has_namespace = false;

    // Parse the label character by character
    while pos < chars.len() {
        let c = chars[pos];

        if is_valid_label_char(c) {
            current_segment.push(c);
            pos += 1;
        } else if is_namespace_separator(c) {
            // Handle namespace separator
            if current_segment.is_empty() {
                return LabelParseResult::Invalid("Empty segment in namespaced label".to_string());
            }

            namespaces.push(current_segment.clone());
            current_segment.clear();
            has_namespace = true;
            pos += 1;

            // Check for consecutive periods
            if pos < chars.len() && is_namespace_separator(chars[pos]) {
                return LabelParseResult::Invalid(
                    "Consecutive periods not allowed in labels".to_string(),
                );
            }

            // Check that next character is valid after period
            if pos >= chars.len() || !is_valid_label_start(chars[pos]) {
                return LabelParseResult::Invalid(
                    "Invalid character after namespace separator".to_string(),
                );
            }
        } else {
            // End of label - any other character
            break;
        }
    }

    // Handle the final segment
    if current_segment.is_empty() {
        return LabelParseResult::Invalid("Label cannot end with namespace separator".to_string());
    }

    namespaces.push(current_segment);

    let label_end = pos;
    let value = chars[label_start..label_end].iter().collect::<String>();

    // Validate length constraints
    if value.len() > 255 {
        return LabelParseResult::Invalid(
            "Label exceeds maximum length of 255 characters".to_string(),
        );
    }

    for segment in &namespaces {
        if segment.len() > 64 {
            return LabelParseResult::Invalid(
                "Label segment exceeds maximum length of 64 characters".to_string(),
            );
        }
    }

    let span = SourceSpan {
        start: start_pos,
        end: Position {
            row: start_pos.row,
            column: label_end,
        },
    };

    LabelParseResult::Valid(Label {
        value,
        namespaces,
        is_namespaced: has_namespace,
        span,
    })
}

/// Validates a complete label string
///
/// # Arguments
/// * `label` - The label string to validate
///
/// # Returns
/// `Ok(())` if valid, `Err(String)` with error message if invalid
pub fn validate_label(label: &str) -> Result<(), String> {
    if label.is_empty() {
        return Err("Label cannot be empty".to_string());
    }

    let start_pos = Position { row: 0, column: 0 };
    match parse_label(label, start_pos) {
        LabelParseResult::Valid(parsed_label) => {
            // Check that the entire string was consumed
            if parsed_label.value != label {
                Err("Label contains invalid characters".to_string())
            } else {
                Ok(())
            }
        }
        LabelParseResult::Invalid(error) => Err(error),
        LabelParseResult::NotFound => Err("No valid label found".to_string()),
    }
}

/// Checks if a label is reserved by the txxt specification
///
/// Reserved labels and namespaces should not be used for custom purposes
pub fn is_reserved_label(label: &str) -> bool {
    // Core format labels
    let core_reserved = ["txxt", "meta", "internal", "system", "reserved"];
    if core_reserved.contains(&label) {
        return true;
    }

    // Standard media types
    let media_reserved = ["text", "image", "audio", "video", "application"];
    if media_reserved.contains(&label) {
        return true;
    }

    // Reserved namespaces
    if label.starts_with("txxt.")
        || label.starts_with("iana.")
        || label.starts_with("rfc.")
        || label.starts_with("iso.")
    {
        return true;
    }

    false
}

/// Extracts label from an identifier token if it represents a valid label
///
/// This function is used to identify labels within the token stream
pub fn extract_label_from_token(token: &ScannerToken) -> Option<Label> {
    if let ScannerToken::Identifier { content, span } = token {
        let start_pos = span.start;
        match parse_label(content, start_pos) {
            LabelParseResult::Valid(label) => Some(label),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_label_start() {
        assert!(is_valid_label_start('a'));
        assert!(is_valid_label_start('Z'));
        assert!(!is_valid_label_start('1'));
        assert!(!is_valid_label_start('_'));
        assert!(!is_valid_label_start('-'));
        assert!(!is_valid_label_start('.'));
    }

    #[test]
    fn test_is_valid_label_char() {
        assert!(is_valid_label_char('a'));
        assert!(is_valid_label_char('Z'));
        assert!(is_valid_label_char('1'));
        assert!(is_valid_label_char('_'));
        assert!(is_valid_label_char('-'));
        assert!(!is_valid_label_char('.'));
        assert!(!is_valid_label_char(' '));
        assert!(!is_valid_label_char('@'));
    }

    #[test]
    fn test_parse_simple_label() {
        let start_pos = Position { row: 0, column: 0 };
        let result = parse_label("python", start_pos);

        match result {
            LabelParseResult::Valid(label) => {
                assert_eq!(label.value, "python");
                assert_eq!(label.namespaces, vec!["python"]);
                assert!(!label.is_namespaced);
            }
            _ => panic!("Expected valid label"),
        }
    }

    #[test]
    fn test_parse_namespaced_label() {
        let start_pos = Position { row: 0, column: 0 };
        let result = parse_label("org.example.custom", start_pos);

        match result {
            LabelParseResult::Valid(label) => {
                assert_eq!(label.value, "org.example.custom");
                assert_eq!(label.namespaces, vec!["org", "example", "custom"]);
                assert!(label.is_namespaced);
            }
            _ => panic!("Expected valid label"),
        }
    }

    #[test]
    fn test_parse_invalid_labels() {
        let start_pos = Position { row: 0, column: 0 };

        // Starts with number
        assert!(matches!(
            parse_label("123invalid", start_pos),
            LabelParseResult::NotFound
        ));

        // Consecutive periods
        assert!(matches!(
            parse_label("label..name", start_pos),
            LabelParseResult::Invalid(_)
        ));

        // Ends with period
        assert!(matches!(
            parse_label("label.", start_pos),
            LabelParseResult::Invalid(_)
        ));

        // Starts with period
        assert!(matches!(
            parse_label(".label", start_pos),
            LabelParseResult::NotFound
        ));
    }

    #[test]
    fn test_validate_label() {
        assert!(validate_label("python").is_ok());
        assert!(validate_label("org.example.custom").is_ok());
        assert!(validate_label("tool_name").is_ok());
        assert!(validate_label("api-v2").is_ok());

        assert!(validate_label("").is_err());
        assert!(validate_label("123invalid").is_err());
        assert!(validate_label("label.").is_err());
        assert!(validate_label(".label").is_err());
        assert!(validate_label("label..name").is_err());
    }

    #[test]
    fn test_is_reserved_label() {
        // Core reserved labels
        assert!(is_reserved_label("txxt"));
        assert!(is_reserved_label("meta"));
        assert!(is_reserved_label("system"));

        // Media type reserved
        assert!(is_reserved_label("text"));
        assert!(is_reserved_label("image"));

        // Reserved namespaces
        assert!(is_reserved_label("txxt.internal"));
        assert!(is_reserved_label("iana.standard"));
        assert!(is_reserved_label("rfc.2119"));

        // Non-reserved
        assert!(!is_reserved_label("python"));
        assert!(!is_reserved_label("org.example"));
        assert!(!is_reserved_label("custom"));
    }

    #[test]
    fn test_extract_label_from_token() {
        let span = SourceSpan {
            start: Position { row: 0, column: 0 },
            end: Position { row: 0, column: 6 },
        };

        let token = ScannerToken::Identifier {
            content: "python".to_string(),
            span: span.clone(),
        };

        let label = extract_label_from_token(&token);
        assert!(label.is_some());

        let label = label.unwrap();
        assert_eq!(label.value, "python");
        assert_eq!(label.namespaces, vec!["python"]);

        // Test non-identifier token
        let text_token = ScannerToken::Text {
            content: "python".to_string(),
            span,
        };
        assert!(extract_label_from_token(&text_token).is_none());
    }
}
