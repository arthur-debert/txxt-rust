//! AST to NotationData Conversion
//!
//! This module provides the core conversion functionality from TXXT AST nodes
//! to the tree visualization format. It implements the semantic-agnostic approach
//! specified in the GitHub issue - the tool doesn't need to understand the semantics
//! of each node type, just how to extract icons, content, and children.

use super::{
    icons::{extract_content_from_node, get_node_type_name, IconConfig, DEFAULT_ICON_CONFIG},
    NotationData, TreeNode, TreeVizResult,
};
use crate::ast::elements::core::ElementNode;

/// Convert an AST node to NotationData with configurable mapping
///
/// This is the primary API function that converts any AST node to a tree
/// representation that can be serialized to JSON or rendered to string.
///
/// # Arguments
///
/// * `ast_node` - The root AST node to convert
/// * `config` - Icon and content extraction configuration
///
/// # Returns
///
/// A `NotationData` structure containing the tree representation and config
pub fn ast_to_notation_data(
    ast_node: &ElementNode,
    config: &IconConfig,
) -> TreeVizResult<NotationData> {
    let root_tree_node = convert_node_recursive(ast_node, config)?;
    Ok(NotationData::new(root_tree_node, config.clone()))
}

/// One-step convenience function for AST to tree notation string
///
/// This combines `ast_to_notation_data` and `notation_data_to_string` into
/// a single function call using the default configuration.
///
/// # Arguments
///
/// * `ast_node` - The root AST node to convert
///
/// # Returns
///
/// A formatted tree notation string ready for display
pub fn ast_to_tree_notation(ast_node: &ElementNode) -> TreeVizResult<String> {
    let config = &*DEFAULT_ICON_CONFIG;
    let notation_data = ast_to_notation_data(ast_node, config)?;
    super::renderer::notation_data_to_string(&notation_data, config)
}

/// Recursively convert an AST node and its children to TreeNode
///
/// This function handles the recursive traversal of the AST structure,
/// applying the configured icon and content extraction rules at each level.
fn convert_node_recursive(ast_node: &ElementNode, config: &IconConfig) -> TreeVizResult<TreeNode> {
    let node_type = get_node_type_name(ast_node);
    let icon = config.get_icon(&node_type);
    let content = extract_content_from_node(ast_node, config);

    let mut tree_node = TreeNode::new(icon, content, node_type.clone());

    // Add metadata if configured
    if config.include_metadata {
        tree_node.set_metadata("element_type".to_string(), node_type.clone());
        tree_node.set_metadata(
            "has_children".to_string(),
            has_children(ast_node).to_string(),
        );
    }

    // Get children using the semantic-agnostic approach
    let children = get_node_children(ast_node);
    for child in children {
        let child_tree_node = convert_node_recursive(&child, config)?;
        tree_node.add_child(child_tree_node);
    }

    Ok(tree_node)
}

/// Extract child nodes from an AST node
///
/// This function implements the semantic-agnostic child extraction.
/// It knows the structure of each ElementNode type but doesn't need
/// to understand the semantics - just how to get the children.
fn get_node_children(node: &ElementNode) -> Vec<ElementNode> {
    match node {
        // Span elements (typically leaf nodes or simple containers)
        ElementNode::TextSpan(_) => vec![],          // Leaf node
        ElementNode::BoldSpan(_) => vec![],          // For now, treating as leaf
        ElementNode::ItalicSpan(_) => vec![],        // For now, treating as leaf
        ElementNode::CodeSpan(_) => vec![],          // Leaf node
        ElementNode::MathSpan(_) => vec![],          // Leaf node
        ElementNode::ReferenceSpan(_) => vec![],     // Leaf node
        ElementNode::CitationSpan(_) => vec![],      // Leaf node
        ElementNode::PageReferenceSpan(_) => vec![], // Leaf node
        ElementNode::SessionReferenceSpan(_) => vec![], // Leaf node
        ElementNode::FootnoteReferenceSpan(_) => vec![], // Leaf node

        // Line elements
        ElementNode::TextLine(_) => vec![], // For now, not extracting spans
        ElementNode::BlankLine(_) => vec![], // Leaf node

        // Block elements - these will have children in a real implementation
        ElementNode::ParagraphBlock(_) => vec![], // Would contain TextLines
        ElementNode::ListBlock(_) => vec![],      // Would contain ListItems
        ElementNode::DefinitionBlock(_) => vec![], // Would contain term + content
        ElementNode::VerbatimBlock(_) => vec![],  // Would contain verbatim lines
        ElementNode::SessionBlock(session) => {
            // Extract children from SessionBlock (just the content container)
            vec![ElementNode::SessionContainer(session.content.clone())]
        }
        ElementNode::AnnotationBlock(_) => vec![], // Would contain content

        // Container elements - these are the main ones with children
        ElementNode::ContentContainer(_) => vec![], // Would contain blocks
        ElementNode::SessionContainer(container) => {
            // Extract children from SessionContainer
            container.content.iter().map(|element| match element {
                crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(p) => {
                    ElementNode::ParagraphBlock(p.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::List(l) => {
                    ElementNode::ListBlock(l.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::Definition(d) => {
                    ElementNode::DefinitionBlock(d.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::Verbatim(v) => {
                    ElementNode::VerbatimBlock(v.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::Annotation(a) => {
                    ElementNode::AnnotationBlock(a.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::Session(s) => {
                    ElementNode::SessionBlock(s.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::ContentContainer(c) => {
                    ElementNode::ContentContainer(c.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::SessionContainer(s) => {
                    ElementNode::SessionContainer(s.clone())
                }
                crate::ast::elements::session::session_container::SessionContainerElement::BlankLine(b) => {
                    ElementNode::BlankLine(b.clone())
                }
            }).collect()
        }
        ElementNode::IgnoreContainer(_) => vec![], // Would contain ignore lines
    }
}

/// Check if a node has children (for metadata)
fn has_children(node: &ElementNode) -> bool {
    !get_node_children(node).is_empty()
}

/// Create a synthetic AST structure for testing
///
/// Since the parser isn't ready yet, this function creates a sample AST
/// structure that demonstrates the tree visualization capabilities.
pub fn create_synthetic_ast() -> ElementNode {
    // For now, create a simple structure using available types
    // In practice, we'd build a more complex tree with actual content

    // Create a simple session block as the root
    use crate::ast::elements::components::parameters::Parameters;
    use crate::ast::elements::session::SessionContainer;
    use crate::ast::tokens::TokenSequence;

    // Since we can't easily construct the complex nested structures without
    // the full parser infrastructure, we'll create a minimal synthetic node
    // This is a placeholder that demonstrates the tree structure concept

    ElementNode::SessionContainer(SessionContainer::new(
        vec![], // blocks - would contain various block types
        vec![], // sessions - would contain nested sessions
        Parameters::default(),
        TokenSequence::new(),
    ))
}

/// Extract tree visualization demo data
///
/// This creates a more comprehensive synthetic tree for demonstration purposes
/// without requiring the full parsing infrastructure.
pub fn create_demo_notation_data() -> NotationData {
    let config = &*DEFAULT_ICON_CONFIG;

    // Create a synthetic tree structure manually
    let mut root = TreeNode::new(
        "⧉".to_string(),
        "Sample Document".to_string(),
        "Document".to_string(),
    );

    // Add a session
    let mut session = TreeNode::new(
        "§".to_string(),
        "1. Introduction".to_string(),
        "SessionBlock".to_string(),
    );

    // Add session title
    let session_title = TreeNode::new(
        "⊤".to_string(),
        "Introduction".to_string(),
        "SessionTitle".to_string(),
    );
    session.add_child(session_title);

    // Add session container
    let mut container = TreeNode::new(
        "➔".to_string(),
        "3 children".to_string(),
        "ContentContainer".to_string(),
    );

    // Add paragraph
    let mut paragraph = TreeNode::new(
        "¶".to_string(),
        "This is a sample paragraph with formatting.".to_string(),
        "ParagraphBlock".to_string(),
    );

    // Add text line to paragraph
    let mut text_line = TreeNode::new(
        "↵".to_string(),
        "This is a sample paragraph with formatting.".to_string(),
        "TextLine".to_string(),
    );

    // Add formatted spans to text line
    text_line.add_child(TreeNode::new(
        "◦".to_string(),
        "This is a sample paragraph with ".to_string(),
        "TextSpan".to_string(),
    ));
    text_line.add_child(TreeNode::new(
        "𝐁".to_string(),
        "formatting".to_string(),
        "BoldSpan".to_string(),
    ));
    text_line.add_child(TreeNode::new(
        "◦".to_string(),
        ".".to_string(),
        "TextSpan".to_string(),
    ));

    paragraph.add_child(text_line);
    container.add_child(paragraph);

    // Add a list
    let mut list = TreeNode::new(
        "☰".to_string(),
        "list (3 items)".to_string(),
        "ListBlock".to_string(),
    );

    list.add_child(TreeNode::new(
        "•".to_string(),
        "First item".to_string(),
        "ListItem".to_string(),
    ));
    list.add_child(TreeNode::new(
        "•".to_string(),
        "Second item with code: `hello world`".to_string(),
        "ListItem".to_string(),
    ));
    list.add_child(TreeNode::new(
        "•".to_string(),
        "Third item".to_string(),
        "ListItem".to_string(),
    ));

    container.add_child(list);

    // Add verbatim block
    let verbatim = TreeNode::new(
        "𝒱".to_string(),
        "verbatim: code.example".to_string(),
        "VerbatimBlock".to_string(),
    );
    container.add_child(verbatim);

    session.add_child(container);
    root.add_child(session);

    NotationData::new(root, config.clone())
}
