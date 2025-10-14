//! TXXT AST Tree Visualization
//!
//! A visual notation system for representing TXXT Abstract Syntax Tree structures 
//! using monochrome Unicode characters. This module provides standardized tree
//! visualization for AST inspection, debugging, and documentation.
//!
//! # Purpose
//!
//! This notation enables clear representation of hierarchical document structure
//! and element relationships when inspecting parsed AST elements.
//!
//! # Tree Structure Format
//!
//! ```text
//! ├─ § 1.3
//! │   ├─ ⊤ The Session Title  
//! │   └─ ➔ children count
//! │       ├─ ¶ paragraph
//! │       │    └─ ↵ the text line content
//! │       └─ ☰ list
//! │            ├─ • item 1
//! │            └─ • item 2
//! ```
//!
//! # API Overview
//!
//! The module provides three main functions as specified:
//!
//! - `ast_to_notation_data()` - Convert AST to serializable tree data
//! - `notation_data_to_string()` - Render tree data to visual notation  
//! - `ast_to_tree_notation()` - One-step convenience conversion
//!
//! # Icon Reference
//!
//! Icons follow the specification in GitHub issue #46:
//!
//! ## Document Structure
//! - ⧉ document
//! - § session  
//! - Ψ session container
//! - ⊤ heading
//!
//! ## Block Elements  
//! - ¶ paragraph
//! - ☰ list
//! - • listItem
//! - 𝒱 verbatim
//! - ℣ verbatim line
//! - ≔ definition
//! - ➔ contentContainer
//!
//! ## Inline Elements
//! - ◦ text
//! - ↵ textLine
//! - 𝐼 emphasis (italic)
//! - 𝐁 strong (bold)
//! - ƒ inlineCode
//! - √ math
//!
//! ## References
//! - ⊕ reference URL
//! - / reference files
//! - † citation
//! - @ author
//! - ◫ pages
//! - ⋯ reference ToCome
//! - ∅ reference unknown
//! - ³ reference footnote
//! - # reference session
//!
//! ## Metadata & Parameters
//! - ◔ label
//! - ✗ key
//! - $ value
//! - " annotation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod converter;
pub mod icons;
pub mod renderer;

#[cfg(test)]
mod tests;

pub use converter::{ast_to_notation_data, ast_to_tree_notation};
pub use icons::{IconConfig, DEFAULT_ICON_CONFIG};
pub use renderer::notation_data_to_string;

/// Tree representation data that can be serialized to JSON
///
/// This structure captures the hierarchical tree information needed
/// for visualization while being serializable for external processing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotationData {
    /// Root node of the tree
    pub root: TreeNode,
    
    /// Configuration used for this tree representation
    pub config: IconConfig,
}

/// Individual node in the tree visualization
///
/// Each node has an icon, content text, and optional children.
/// The structure mirrors the AST hierarchy but with simplified,
/// displayable information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Unicode icon representing the node type
    pub icon: String,
    
    /// Text content for display (extracted from AST node)
    pub content: String,
    
    /// AST node type name for debugging/tooling
    pub node_type: String,
    
    /// Child nodes in document order
    pub children: Vec<TreeNode>,
    
    /// Optional metadata for debugging
    pub metadata: HashMap<String, String>,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(icon: String, content: String, node_type: String) -> Self {
        Self {
            icon,
            content,
            node_type,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a child node
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }
    
    /// Add multiple child nodes
    pub fn add_children(&mut self, children: Vec<TreeNode>) {
        self.children.extend(children);
    }
    
    /// Set metadata value
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl NotationData {
    /// Create new notation data with root node and config
    pub fn new(root: TreeNode, config: IconConfig) -> Self {
        Self { root, config }
    }
}

/// Error types for tree visualization operations
#[derive(Debug, Clone, PartialEq)]
pub enum TreeVizError {
    /// Configuration is invalid or missing required mappings
    InvalidConfig(String),
    
    /// AST node type not supported
    UnsupportedNodeType(String),
    
    /// Content extraction failed
    ContentExtractionFailed(String),
    
    /// Tree rendering failed
    RenderingFailed(String),
}

impl std::fmt::Display for TreeVizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeVizError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            TreeVizError::UnsupportedNodeType(node_type) => {
                write!(f, "Unsupported node type: {}", node_type)
            }
            TreeVizError::ContentExtractionFailed(msg) => {
                write!(f, "Content extraction failed: {}", msg)
            }
            TreeVizError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
        }
    }
}

impl std::error::Error for TreeVizError {}

/// Result type for tree visualization operations
pub type TreeVizResult<T> = Result<T, TreeVizError>;