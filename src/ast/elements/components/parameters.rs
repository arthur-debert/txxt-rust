//! Parameters: shared metadata system for AST nodes
//!
//! This module defines the parameter system that provides consistent metadata
//! capabilities across multiple AST node types (verbatim blocks, annotations,
//! definitions, etc.).
//!
//! src/parser/mod.rs has the full architecture overview.
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

use crate::cst::ScannerTokenSequence;

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
    pub tokens: ScannerTokenSequence,
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
            tokens: ScannerTokenSequence::new(),
        }
    }

    /// Create parameters from a map
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self {
            map,
            tokens: ScannerTokenSequence::new(),
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
        self.get("ref").or_else(|| self.get("id"))
    }

    /// Check if this element can be referenced
    pub fn is_referenceable(&self) -> bool {
        self.reference_id().is_some()
    }

    /// Get category classification
    pub fn category(&self) -> Option<&String> {
        self.get("category")
    }

    /// Get severity level (for warnings/errors)
    pub fn severity(&self) -> Option<&String> {
        self.get("severity")
    }

    /// Get language hint (for code blocks)
    pub fn language(&self) -> Option<&String> {
        self.get("lang")
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
            // Validate common parameter values (arbitrary keys are always allowed)
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
