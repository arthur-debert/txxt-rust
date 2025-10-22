//! Represents a parsed label with an optional namespace.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedLabel {
    pub name: String,
    pub namespace: Option<String>,
}

impl ParsedLabel {
    /// Parses a raw label string into a `ParsedLabel`.
    ///
    /// The namespace is everything before the last dot, and the name is everything after.
    /// If there is no dot, the entire string is the name and the namespace is `None`.
    pub fn from_raw(raw_label: &str) -> Self {
        if let Some(dot_pos) = raw_label.rfind('.') {
            let (namespace, name) = raw_label.split_at(dot_pos);
            let name = name[1..].to_string(); // Skip the dot

            if namespace.is_empty() {
                // Case: ".note"
                Self {
                    name,
                    namespace: None,
                }
            } else {
                // Case: "org.example.custom"
                Self {
                    name,
                    namespace: Some(namespace.to_string()),
                }
            }
        } else {
            // Case: "note"
            Self {
                name: raw_label.to_string(),
                namespace: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_no_namespace() {
        let parsed = ParsedLabel::from_raw("note");
        assert_eq!(
            parsed,
            ParsedLabel {
                name: "note".to_string(),
                namespace: None,
            }
        );
    }

    #[test]
    fn test_from_raw_single_level_namespace() {
        let parsed = ParsedLabel::from_raw("custom.note");
        assert_eq!(
            parsed,
            ParsedLabel {
                name: "note".to_string(),
                namespace: Some("custom".to_string()),
            }
        );
    }

    #[test]
    fn test_from_raw_multi_level_namespace() {
        let parsed = ParsedLabel::from_raw("org.example.custom");
        assert_eq!(
            parsed,
            ParsedLabel {
                name: "custom".to_string(),
                namespace: Some("org.example".to_string()),
            }
        );
    }

    #[test]
    fn test_from_raw_leading_dot() {
        let parsed = ParsedLabel::from_raw(".note");
        assert_eq!(
            parsed,
            ParsedLabel {
                name: "note".to_string(),
                namespace: None,
            }
        );
    }

    #[test]
    fn test_from_raw_trailing_dot() {
        // This is an edge case that we should clarify expected behavior for.
        // For now, let's assume the name becomes empty.
        let parsed = ParsedLabel::from_raw("note.");
        assert_eq!(
            parsed,
            ParsedLabel {
                name: "".to_string(),
                namespace: Some("note".to_string()),
            }
        );
    }

    #[test]
    fn test_from_raw_only_dot() {
        let parsed = ParsedLabel::from_raw(".");
        assert_eq!(
            parsed,
            ParsedLabel {
                name: "".to_string(),
                namespace: None,
            }
        );
    }
}
