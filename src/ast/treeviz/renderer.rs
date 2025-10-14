//! Tree Notation String Renderer
//!
//! This module handles the conversion of NotationData structures into the 
//! visual tree notation string format using Unicode box-drawing characters.
//!
//! The renderer produces output in this format:
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

use super::{NotationData, TreeNode, TreeVizError, TreeVizResult, icons::IconConfig};

/// Convert NotationData to formatted tree string
///
/// This is the second part of the two-step API that renders the tree data
/// structure into the visual notation format using Unicode box-drawing characters.
///
/// # Arguments
///
/// * `data` - The tree data structure to render
/// * `config` - Configuration for rendering options
///
/// # Returns
///
/// A formatted string ready for display in terminal or text output
pub fn notation_data_to_string(
    data: &NotationData, 
    config: &IconConfig
) -> TreeVizResult<String> {
    let mut output = String::new();
    render_node(&data.root, &mut output, "", true, config)?;
    Ok(output)
}

/// Recursively render a tree node and its children
///
/// This function handles the complex logic of drawing the tree structure
/// with proper Unicode box-drawing characters and indentation.
///
/// # Arguments
///
/// * `node` - The current node to render
/// * `output` - Mutable string buffer to append output to
/// * `prefix` - Current indentation prefix for this level
/// * `is_last` - Whether this node is the last child at its level
/// * `config` - Rendering configuration
fn render_node(
    node: &TreeNode,
    output: &mut String,
    prefix: &str,
    is_last: bool,
    config: &IconConfig,
) -> TreeVizResult<()> {
    // Choose the appropriate tree connector
    let connector = if is_last { "└─" } else { "├─" };
    
    // Render this node
    output.push_str(&format!("{}{} {} {}\n", prefix, connector, node.icon, node.content));
    
    // Render children if any
    if !node.children.is_empty() {
        // Calculate prefix for children
        let child_prefix = if is_last {
            format!("{}    ", prefix) // No vertical line if this is the last child
        } else {
            format!("{}│   ", prefix) // Vertical line continues
        };
        
        // Render each child
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            render_node(child, output, &child_prefix, is_last_child, config)?;
        }
    }
    
    Ok(())
}

/// Render tree with custom formatting options
///
/// Extended version of the renderer that supports additional formatting options
/// for different output contexts (terminal, HTML, etc.).
pub fn render_with_options(
    data: &NotationData,
    options: &RenderOptions,
) -> TreeVizResult<String> {
    let mut output = String::new();
    render_node_with_options(&data.root, &mut output, "", true, options)?;
    Ok(output)
}

/// Rendering options for different output contexts
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Whether to include debug information
    pub include_debug: bool,
    
    /// Whether to include metadata 
    pub include_metadata: bool,
    
    /// Custom tree drawing characters
    pub tree_chars: TreeChars,
    
    /// Maximum content length before truncation
    pub max_content_length: Option<usize>,
    
    /// Whether to colorize output (for terminal)
    pub colorize: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            include_debug: false,
            include_metadata: false,
            tree_chars: TreeChars::default(),
            max_content_length: None,
            colorize: false,
        }
    }
}

/// Customizable tree drawing characters
#[derive(Debug, Clone)]
pub struct TreeChars {
    /// Connector for non-last children
    pub branch: &'static str,
    
    /// Connector for last child
    pub last_branch: &'static str,
    
    /// Vertical line for continued indentation
    pub vertical: &'static str,
    
    /// Spacing for empty indentation
    pub space: &'static str,
}

impl Default for TreeChars {
    fn default() -> Self {
        Self {
            branch: "├─",
            last_branch: "└─",
            vertical: "│",
            space: " ",
        }
    }
}

impl TreeChars {
    /// ASCII-only tree characters for compatibility
    pub fn ascii() -> Self {
        Self {
            branch: "|-",
            last_branch: "`-",
            vertical: "|",
            space: " ",
        }
    }
    
    /// Double-line Unicode characters for emphasis
    pub fn double_line() -> Self {
        Self {
            branch: "╠═",
            last_branch: "╚═",
            vertical: "║",
            space: " ",
        }
    }
}

/// Render node with extended options
fn render_node_with_options(
    node: &TreeNode,
    output: &mut String,
    prefix: &str,
    is_last: bool,
    options: &RenderOptions,
) -> TreeVizResult<()> {
    // Choose the appropriate tree connector
    let connector = if is_last { 
        options.tree_chars.last_branch 
    } else { 
        options.tree_chars.branch 
    };
    
    // Prepare content with optional truncation
    let content = if let Some(max_len) = options.max_content_length {
        if node.content.len() > max_len {
            format!("{}...", &node.content[..max_len.saturating_sub(3)])
        } else {
            node.content.clone()
        }
    } else {
        node.content.clone()
    };
    
    // Render this node
    output.push_str(&format!(
        "{}{} {} {}", 
        prefix, 
        connector, 
        node.icon, 
        content
    ));
    
    // Add debug information if requested
    if options.include_debug {
        output.push_str(&format!(" [{}]", node.node_type));
    }
    
    // Add metadata if requested
    if options.include_metadata && !node.metadata.is_empty() {
        let metadata_str = node.metadata
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(" ({})", metadata_str));
    }
    
    output.push('\n');
    
    // Render children if any
    if !node.children.is_empty() {
        // Calculate prefix for children
        let child_prefix = if is_last {
            format!("{}    ", prefix) // No vertical line if this is the last child
        } else {
            format!("{}{}   ", prefix, options.tree_chars.vertical) // Vertical line continues
        };
        
        // Render each child
        for (i, child) in node.children.iter().enumerate() {
            let is_last_child = i == node.children.len() - 1;
            render_node_with_options(child, output, &child_prefix, is_last_child, options)?;
        }
    }
    
    Ok(())
}

/// Convert NotationData to JSON string
///
/// Provides an alternative output format for programmatic consumption
/// or integration with other tools.
pub fn notation_data_to_json(data: &NotationData) -> TreeVizResult<String> {
    serde_json::to_string_pretty(data)
        .map_err(|e| TreeVizError::RenderingFailed(format!("JSON serialization failed: {}", e)))
}

/// Convert NotationData to compact JSON string
///
/// Single-line JSON output for machine processing
pub fn notation_data_to_compact_json(data: &NotationData) -> TreeVizResult<String> {
    serde_json::to_string(data)
        .map_err(|e| TreeVizError::RenderingFailed(format!("JSON serialization failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::treeviz::converter::create_demo_notation_data;
    
    #[test]
    fn test_basic_rendering() {
        let demo_data = create_demo_notation_data();
        let result = notation_data_to_string(&demo_data, &demo_data.config);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        
        // Verify basic structure
        assert!(output.contains("⧉ Sample Document"));
        assert!(output.contains("§ 1. Introduction"));
        assert!(output.contains("¶ This is a sample paragraph"));
        assert!(output.contains("☰ list (3 items)"));
        
        // Verify tree structure characters
        assert!(output.contains("├─"));
        assert!(output.contains("└─"));
        assert!(output.contains("│"));
    }
    
    #[test]
    fn test_json_output() {
        let demo_data = create_demo_notation_data();
        let result = notation_data_to_json(&demo_data);
        
        assert!(result.is_ok());
        let json = result.unwrap();
        
        // Verify JSON structure
        assert!(json.contains("\"root\""));
        assert!(json.contains("\"config\""));
        assert!(json.contains("\"icon\""));
        assert!(json.contains("\"content\""));
        assert!(json.contains("\"children\""));
    }
    
    #[test]
    fn test_ascii_rendering() {
        let demo_data = create_demo_notation_data();
        let options = RenderOptions {
            tree_chars: TreeChars::ascii(),
            ..Default::default()
        };
        
        let result = render_with_options(&demo_data, &options);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("|-"));
        assert!(output.contains("`-"));
        assert!(output.contains("|"));
    }
}