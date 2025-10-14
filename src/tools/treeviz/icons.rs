//! Icon Configuration for TXXT AST Tree Visualization
//!
//! This module defines the configurable icon mapping system that determines
//! which Unicode character represents each AST node type and how to extract
//! displayable content from nodes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ast::elements::core::ElementNode;

/// Configuration for icon mappings and content extraction
///
/// The configuration system allows customization of:
/// - Which icon represents each AST node type
/// - How to extract displayable content from each node type
/// - Whether to show metadata or debug information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconConfig {
    /// Node type to icon character mappings
    pub type_icons: HashMap<String, String>,

    /// Node type to content extraction function mappings
    /// The key is node type, value is the property/method to use for content
    pub content_extractors: HashMap<String, ContentExtractor>,

    /// Whether to include debug information in output
    pub show_debug_info: bool,

    /// Whether to include metadata in tree nodes
    pub include_metadata: bool,
}

/// Content extraction strategy for different node types
///
/// Defines how to get displayable text content from each AST node type.
/// The tool should be semantically agnostic - it just needs to know what
/// property to use for children, labels, and content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentExtractor {
    /// Property/method name to get display text
    pub content_property: String,

    /// Property/method name to get child nodes (for traversal)
    pub children_property: String,

    /// Optional format string for content display
    pub format_template: Option<String>,
}

impl ContentExtractor {
    /// Create a simple content extractor
    pub fn simple(content_property: &str, children_property: &str) -> Self {
        Self {
            content_property: content_property.to_string(),
            children_property: children_property.to_string(),
            format_template: None,
        }
    }

    /// Create a content extractor with formatting template
    pub fn with_format(
        content_property: &str,
        children_property: &str,
        format_template: &str,
    ) -> Self {
        Self {
            content_property: content_property.to_string(),
            children_property: children_property.to_string(),
            format_template: Some(format_template.to_string()),
        }
    }
}

impl Default for IconConfig {
    fn default() -> Self {
        DEFAULT_ICON_CONFIG.clone()
    }
}

impl IconConfig {
    /// Create a new icon configuration
    pub fn new() -> Self {
        Self {
            type_icons: HashMap::new(),
            content_extractors: HashMap::new(),
            show_debug_info: false,
            include_metadata: false,
        }
    }

    /// Get icon for a node type, falling back to default
    pub fn get_icon(&self, node_type: &str) -> String {
        self.type_icons
            .get(node_type)
            .cloned()
            .unwrap_or_else(|| "◦".to_string()) // Default: generic text icon
    }

    /// Get content extractor for a node type
    pub fn get_content_extractor(&self, node_type: &str) -> Option<&ContentExtractor> {
        self.content_extractors.get(node_type)
    }

    /// Add icon mapping
    pub fn add_icon(&mut self, node_type: String, icon: String) {
        self.type_icons.insert(node_type, icon);
    }

    /// Add content extractor
    pub fn add_extractor(&mut self, node_type: String, extractor: ContentExtractor) {
        self.content_extractors.insert(node_type, extractor);
    }
}

/// Default icon configuration based on GitHub issue #46 specification
///
/// This implements the complete icon reference from the issue, providing
/// monochrome Unicode characters for all TXXT AST element types.
pub static DEFAULT_ICON_CONFIG: std::sync::LazyLock<IconConfig> = std::sync::LazyLock::new(|| {
    let mut config = IconConfig::new();

    // Document Structure icons
    config.add_icon("Document".to_string(), "⧉".to_string());
    config.add_icon("SessionBlock".to_string(), "§".to_string());
    config.add_icon("SessionContainer".to_string(), "Ψ".to_string());
    config.add_icon("SessionTitle".to_string(), "⊤".to_string());

    // Block Elements icons
    config.add_icon("ParagraphBlock".to_string(), "¶".to_string());
    config.add_icon("ListBlock".to_string(), "☰".to_string());
    config.add_icon("ListItem".to_string(), "•".to_string());
    config.add_icon("VerbatimBlock".to_string(), "𝒱".to_string());
    config.add_icon("VerbatimLine".to_string(), "℣".to_string());
    config.add_icon("DefinitionBlock".to_string(), "≔".to_string());
    config.add_icon("ContentContainer".to_string(), "➔".to_string());

    // Inline Elements icons
    config.add_icon("TextSpan".to_string(), "◦".to_string());
    config.add_icon("TextLine".to_string(), "↵".to_string());
    config.add_icon("ItalicSpan".to_string(), "𝐼".to_string());
    config.add_icon("BoldSpan".to_string(), "𝐁".to_string());
    config.add_icon("CodeSpan".to_string(), "ƒ".to_string());
    config.add_icon("MathSpan".to_string(), "√".to_string());

    // Reference icons
    config.add_icon("ReferenceSpan".to_string(), "⊕".to_string()); // URL references
    config.add_icon("FileReference".to_string(), "/".to_string());
    config.add_icon("CitationSpan".to_string(), "†".to_string());
    config.add_icon("AuthorReference".to_string(), "@".to_string());
    config.add_icon("PageReferenceSpan".to_string(), "◫".to_string());
    config.add_icon("ReferenceToCome".to_string(), "⋯".to_string());
    config.add_icon("ReferenceUnknown".to_string(), "∅".to_string());
    config.add_icon("FootnoteReferenceSpan".to_string(), "³".to_string());
    config.add_icon("SessionReferenceSpan".to_string(), "#".to_string());

    // Metadata & Parameters icons
    config.add_icon("Label".to_string(), "◔".to_string());
    config.add_icon("ParameterKey".to_string(), "✗".to_string());
    config.add_icon("ParameterValue".to_string(), "$".to_string());
    config.add_icon("AnnotationBlock".to_string(), "\"".to_string());

    // Content extractors for common node types
    config.add_extractor(
        "ParagraphBlock".to_string(),
        ContentExtractor::simple("text_content", "lines"),
    );

    config.add_extractor(
        "SessionBlock".to_string(),
        ContentExtractor::simple("title", "content"),
    );

    config.add_extractor(
        "ListBlock".to_string(),
        ContentExtractor::with_format("style", "items", "list ({} items)"),
    );

    config.add_extractor(
        "ListItem".to_string(),
        ContentExtractor::simple("content", "children"),
    );

    config.add_extractor(
        "VerbatimBlock".to_string(),
        ContentExtractor::with_format("label", "content", "verbatim: {}"),
    );

    config.add_extractor(
        "TextLine".to_string(),
        ContentExtractor::simple("content", "spans"),
    );

    config.add_extractor(
        "TextSpan".to_string(),
        ContentExtractor::simple("text", "children"),
    );

    config.add_extractor(
        "BoldSpan".to_string(),
        ContentExtractor::with_format("text", "children", "*{}*"),
    );

    config.add_extractor(
        "ItalicSpan".to_string(),
        ContentExtractor::with_format("text", "children", "_{}_"),
    );

    config.add_extractor(
        "CodeSpan".to_string(),
        ContentExtractor::with_format("text", "children", "`{}`"),
    );

    config.add_extractor(
        "DefinitionBlock".to_string(),
        ContentExtractor::simple("term", "content"),
    );

    config.add_extractor(
        "AnnotationBlock".to_string(),
        ContentExtractor::with_format("label", "content", ":: {} ::"),
    );

    config.add_extractor(
        "ReferenceSpan".to_string(),
        ContentExtractor::with_format("target", "children", "[{}]"),
    );

    config.add_extractor(
        "CitationSpan".to_string(),
        ContentExtractor::with_format("key", "children", "[@{}]"),
    );

    config
});

/// Extract content from an ElementNode using the configured extractor
///
/// This function provides semantic-agnostic content extraction by using
/// the configured property mappings rather than hard-coded node knowledge.
pub fn extract_content_from_node(node: &ElementNode, config: &IconConfig) -> String {
    let node_type = get_node_type_name(node);

    // Get the content extractor for this node type
    if let Some(extractor) = config.get_content_extractor(&node_type) {
        // For now, we'll implement basic content extraction
        // In a real implementation, this would use reflection or trait dispatch
        extract_content_by_type(node, extractor)
    } else {
        // Fallback: try to get some basic text representation
        node_type.to_string()
    }
}

/// Get the type name for an ElementNode
pub fn get_node_type_name(node: &ElementNode) -> String {
    match node {
        ElementNode::TextSpan(_) => "TextSpan".to_string(),
        ElementNode::BoldSpan(_) => "BoldSpan".to_string(),
        ElementNode::ItalicSpan(_) => "ItalicSpan".to_string(),
        ElementNode::CodeSpan(_) => "CodeSpan".to_string(),
        ElementNode::MathSpan(_) => "MathSpan".to_string(),
        ElementNode::ReferenceSpan(_) => "ReferenceSpan".to_string(),
        ElementNode::CitationSpan(_) => "CitationSpan".to_string(),
        ElementNode::PageReferenceSpan(_) => "PageReferenceSpan".to_string(),
        ElementNode::SessionReferenceSpan(_) => "SessionReferenceSpan".to_string(),
        ElementNode::FootnoteReferenceSpan(_) => "FootnoteReferenceSpan".to_string(),
        ElementNode::TextLine(_) => "TextLine".to_string(),
        ElementNode::BlankLine(_) => "BlankLine".to_string(),
        ElementNode::ParagraphBlock(_) => "ParagraphBlock".to_string(),
        ElementNode::ListBlock(_) => "ListBlock".to_string(),
        ElementNode::DefinitionBlock(_) => "DefinitionBlock".to_string(),
        ElementNode::VerbatimBlock(_) => "VerbatimBlock".to_string(),
        ElementNode::SessionBlock(_) => "SessionBlock".to_string(),
        ElementNode::AnnotationBlock(_) => "AnnotationBlock".to_string(),
        ElementNode::ContentContainer(_) => "ContentContainer".to_string(),
        ElementNode::SessionContainer(_) => "SessionContainer".to_string(),
        ElementNode::IgnoreContainer(_) => "IgnoreContainer".to_string(),
    }
}

/// Extract content from a node using the specified extractor strategy
///
/// This is a simplified implementation that would be expanded with proper
/// reflection or trait dispatch in a full implementation.
fn extract_content_by_type(node: &ElementNode, extractor: &ContentExtractor) -> String {
    // For now, implement basic content extraction patterns
    // In practice, this would use the extractor.content_property to determine
    // what method/field to access on the node

    let base_content = match node {
        ElementNode::TextSpan(_) => "text content".to_string(),
        ElementNode::BoldSpan(_) => "bold text".to_string(),
        ElementNode::ItalicSpan(_) => "italic text".to_string(),
        ElementNode::CodeSpan(_) => "code text".to_string(),
        ElementNode::MathSpan(_) => "math formula".to_string(),
        ElementNode::ReferenceSpan(_) => "reference link".to_string(),
        ElementNode::CitationSpan(_) => "citation".to_string(),
        ElementNode::PageReferenceSpan(_) => "page reference".to_string(),
        ElementNode::SessionReferenceSpan(_) => "session reference".to_string(),
        ElementNode::FootnoteReferenceSpan(_) => "footnote reference".to_string(),
        ElementNode::TextLine(_) => "text line".to_string(),
        ElementNode::BlankLine(_) => "".to_string(),
        ElementNode::ParagraphBlock(_) => "paragraph content".to_string(),
        ElementNode::ListBlock(_) => "list".to_string(),
        ElementNode::DefinitionBlock(_) => "definition term".to_string(),
        ElementNode::VerbatimBlock(_) => "verbatim content".to_string(),
        ElementNode::SessionBlock(_) => "session title".to_string(),
        ElementNode::AnnotationBlock(_) => "annotation".to_string(),
        ElementNode::ContentContainer(_) => "content container".to_string(),
        ElementNode::SessionContainer(_) => "session container".to_string(),
        ElementNode::IgnoreContainer(_) => "ignore container".to_string(),
    };

    // Apply format template if provided
    if let Some(template) = &extractor.format_template {
        template.replace("{}", &base_content)
    } else {
        base_content
    }
}
