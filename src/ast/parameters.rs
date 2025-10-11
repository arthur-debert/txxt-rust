//! Parameters: shared metadata system for AST nodes
//!
//! This module defines the parameter system that provides consistent metadata
//! capabilities across multiple AST node types (verbatim blocks, annotations,
//! definitions, etc.).
//!
//! # Parsing Pipeline Position
//!
//! **Phase 2.b: Parsing** (Parameter Extraction)
//!
//! Parameters are extracted during the main parsing phase when elements with
//! parameter support are processed. The parameter syntax is shared across
//! multiple element types for consistency.
//!
//! Pipeline: `Tokens` → `Block Grouping` → **`Parameter Parsing`** → `Assembly`
//!
//! ## Parameter Syntax Examples
//!
//! Verbatim blocks:
//! ```txxt
//! Hello World:
//!     def hello():
//!         print("Hello")
//! python:ref=hello-world
//! ```
//!
//! Annotations:
//! ```txxt
//! :: important:id=security-note :: This section requires careful attention
//! :: warning:severity=high,category=security :: Critical security issue
//! ```
//!
//! Definitions:
//! ```txxt  
//! Term:ref=important-term,category=glossary
//!     Definition content here
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::tokens::TokenSequence;

/// Parameter collection for AST nodes
///
/// Parameters provide a consistent metadata system across multiple AST node types.
/// They enable:
/// - Named anchors for references (ref=hello-world, id=security-note)
/// - Categorization and metadata (severity=high, category=security)  
/// - Tool-specific configuration (lang=rust, theme=dark)
/// - Custom extensions without AST changes
///
/// The parameter syntax is shared between verbatim blocks, annotations,
/// definitions, and other extensible elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameters {
    /// Key-value parameter map
    pub map: HashMap<String, String>,

    /// Raw tokens for source reconstruction
    pub tokens: TokenSequence,
}

/// Common parameter keys with semantic meaning
///
/// While parameters are arbitrary key-value pairs, certain keys have
/// conventional semantic meaning across the TXXT ecosystem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterKeys;

impl ParameterKeys {
    /// Reference identifier for creating named anchors
    /// Used by: verbatim blocks, annotations, definitions
    /// Example: `ref=hello-world` creates anchor accessible via `[#hello-world]`
    pub const REF: &'static str = "ref";

    /// Identifier (alternative to ref, often used in annotations)
    /// Used by: annotations, sessions
    /// Example: `id=security-note` creates anchor accessible via `[#security-note]`
    pub const ID: &'static str = "id";

    /// Category for grouping and organization
    /// Used by: definitions, annotations, verbatim blocks
    /// Example: `category=glossary` for definition categorization
    pub const CATEGORY: &'static str = "category";

    /// Severity level for warnings/errors
    /// Used by: annotations (warnings, errors)
    /// Example: `severity=high` for critical issues
    pub const SEVERITY: &'static str = "severity";

    /// Language hint for syntax highlighting
    /// Used by: verbatim blocks
    /// Example: `lang=rust` for code block language
    pub const LANG: &'static str = "lang";

    /// Theme/style hint
    /// Used by: verbatim blocks, annotations
    /// Example: `theme=dark` for styling preferences
    pub const THEME: &'static str = "theme";

    /// Version information
    /// Used by: annotations, verbatim blocks
    /// Example: `version=2.0` for format versioning
    pub const VERSION: &'static str = "version";
}

/// Parameter validation and processing
///
/// Provides utilities for parameter validation, type conversion,
/// and semantic interpretation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterProcessor {
    /// Whether to validate known parameter keys
    pub validate_known_keys: bool,

    /// Whether to allow arbitrary custom parameters
    pub allow_custom_parameters: bool,
}

impl Parameters {
    /// Create empty parameters
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            tokens: TokenSequence::new(),
        }
    }

    /// Create parameters from a map
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            map,
            tokens: TokenSequence::new(),
        }
    }

    /// Get parameter value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.map.get(key)
    }

    /// Set parameter value
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    /// Check if parameter exists
    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Get reference identifier (ref or id parameter)
    pub fn reference_id(&self) -> Option<&String> {
        self.get(ParameterKeys::REF)
            .or_else(|| self.get(ParameterKeys::ID))
    }

    /// Check if this element can be referenced
    pub fn is_referenceable(&self) -> bool {
        self.reference_id().is_some()
    }

    /// Get category classification
    pub fn category(&self) -> Option<&String> {
        self.get(ParameterKeys::CATEGORY)
    }

    /// Get severity level (for warnings/errors)
    pub fn severity(&self) -> Option<&String> {
        self.get(ParameterKeys::SEVERITY)
    }

    /// Get language hint (for code blocks)
    pub fn language(&self) -> Option<&String> {
        self.get(ParameterKeys::LANG)
    }

    /// Check if parameters are empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get all parameter keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    /// Get all parameter values
    pub fn values(&self) -> impl Iterator<Item = &String> {
        self.map.values()
    }

    /// Iterate over key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.map.iter()
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ParameterProcessor {
    fn default() -> Self {
        Self {
            validate_known_keys: true,
            allow_custom_parameters: true,
        }
    }
}

impl ParameterProcessor {
    /// Validate parameters according to processor configuration
    pub fn validate(&self, parameters: &Parameters) -> Result<(), ParameterError> {
        if self.validate_known_keys {
            // Validate known parameter values
            if let Some(severity) = parameters.severity() {
                match severity.as_str() {
                    "low" | "medium" | "high" | "critical" => {}
                    _ => return Err(ParameterError::InvalidSeverity(severity.clone())),
                }
            }
        }

        Ok(())
    }

    /// Process parameters and extract semantic information
    pub fn process(&self, parameters: &Parameters) -> ParameterInfo {
        ParameterInfo {
            reference_id: parameters.reference_id().cloned(),
            category: parameters.category().cloned(),
            severity: parameters.severity().cloned(),
            language: parameters.language().cloned(),
            custom: parameters.map.clone(),
        }
    }
}

/// Processed parameter information
///
/// Contains extracted semantic information from parameters for easy access
/// by tooling and processing systems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Reference identifier for named anchors
    pub reference_id: Option<String>,

    /// Category classification
    pub category: Option<String>,

    /// Severity level
    pub severity: Option<String>,

    /// Language hint
    pub language: Option<String>,

    /// All parameters (including custom ones)
    pub custom: HashMap<String, String>,
}

/// Parameter-related errors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterError {
    /// Invalid severity value
    InvalidSeverity(String),

    /// Missing required parameter
    MissingRequired(String),

    /// Invalid parameter format
    InvalidFormat(String),

    /// Custom validation error
    Custom(String),
}
