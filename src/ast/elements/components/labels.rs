//! Label System for Elements
//!
//! Labels are used throughout TXXT for identification and classification.

use serde::{Deserialize, Serialize};

/// Label for elements (used in verbatim blocks, annotations, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Label {
    /// The label text (e.g., "rust", "note", "warning.critical")
    pub text: String,

    /// Whether this is a hierarchical label (contains dots)
    pub is_hierarchical: bool,
}

impl Label {
    /// Create a new label
    pub fn new(text: String) -> Self {
        let is_hierarchical = text.contains('.');
        Self {
            text,
            is_hierarchical,
        }
    }

    /// Get the label text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the top-level part of a hierarchical label
    pub fn top_level(&self) -> &str {
        if self.is_hierarchical {
            self.text.split('.').next().unwrap_or(&self.text)
        } else {
            &self.text
        }
    }

    /// Get all parts of a hierarchical label
    pub fn parts(&self) -> Vec<&str> {
        self.text.split('.').collect()
    }
}

impl From<&str> for Label {
    fn from(text: &str) -> Self {
        Self::new(text.to_string())
    }
}

impl From<String> for Label {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}
