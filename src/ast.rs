use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple AST node for the TXXT parser
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AstNode {
    /// The type of this AST node
    pub node_type: String,

    /// Child nodes
    pub children: Vec<AstNode>,

    /// Additional attributes/parameters for this node
    pub attributes: HashMap<String, String>,

    /// Text content for leaf nodes
    pub content: Option<String>,

    /// Source location information
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
}

impl AstNode {
    /// Create a new AST node with the given type
    pub fn new(node_type: String) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            attributes: HashMap::new(),
            content: None,
            start_line: None,
            end_line: None,
        }
    }

    /// Create a new AST node with type and content
    pub fn with_content(node_type: String, content: String) -> Self {
        Self {
            node_type,
            children: Vec::new(),
            attributes: HashMap::new(),
            content: Some(content),
            start_line: None,
            end_line: None,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    /// Set an attribute
    pub fn set_attribute(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    /// Set source location
    pub fn set_location(&mut self, start_line: usize, end_line: usize) {
        self.start_line = Some(start_line);
        self.end_line = Some(end_line);
    }
}

/// Container for the entire parsed document
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Root AST node
    pub root: AstNode,

    /// Source file path
    pub source: String,
}

impl Document {
    pub fn new(source: String) -> Self {
        Self {
            root: AstNode::new("document".to_string()),
            source,
        }
    }
}
