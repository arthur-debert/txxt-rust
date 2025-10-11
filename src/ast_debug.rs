//! AST debugging and visualization utilities
//!
//! This module provides tools for visualizing and debugging AST structures,
//! including tree printing, structure comparison, and content inspection.

#[cfg(feature = "new-ast")]
use crate::ast::{
    base::Document,
    blocks::Block,
    inlines::{Inline, TextTransform},
    structure::Container,
};

#[cfg(feature = "new-ast")]
use std::fmt::Write;

/// Tree visualization configuration
#[derive(Debug, Clone)]
pub struct TreeConfig {
    /// Show token information
    pub show_tokens: bool,
    /// Show annotation information  
    pub show_annotations: bool,
    /// Show parameter information
    pub show_parameters: bool,
    /// Maximum depth to display (None for unlimited)
    pub max_depth: Option<usize>,
    /// Compact mode (less verbose output)
    pub compact: bool,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            show_tokens: false,
            show_annotations: true,
            show_parameters: true,
            max_depth: None,
            compact: false,
        }
    }
}

/// Tree visualization for AST debugging
#[cfg(feature = "new-ast")]
pub struct AstTreeVisualizer {
    config: TreeConfig,
}

#[cfg(feature = "new-ast")]
impl Default for AstTreeVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "new-ast")]
impl AstTreeVisualizer {
    /// Create a new tree visualizer with default config
    pub fn new() -> Self {
        Self {
            config: TreeConfig::default(),
        }
    }

    /// Create a new tree visualizer with custom config
    pub fn with_config(config: TreeConfig) -> Self {
        Self { config }
    }

    /// Visualize a complete document as a tree
    pub fn visualize_document(&self, doc: &Document) -> String {
        let mut output = String::new();

        writeln!(output, "📄 Document").unwrap();

        // Show document metadata
        if !self.config.compact {
            if let Some(ref title) = doc.meta.title {
                writeln!(output, "├─ 📝 Title: {:?}", title).unwrap();
            }
            if !doc.meta.authors.is_empty() {
                writeln!(output, "├─ 👥 Authors: {} items", doc.meta.authors.len()).unwrap();
            }
            if !doc.meta.custom.is_empty() {
                writeln!(
                    output,
                    "├─ 🏷️ Custom metadata: {} items",
                    doc.meta.custom.len()
                )
                .unwrap();
            }
        }

        // Show assembly info
        if !self.config.compact {
            writeln!(output, "├─ 🔧 Parser: {}", doc.assembly_info.parser_version).unwrap();
            if let Some(ref path) = doc.assembly_info.source_path {
                writeln!(output, "├─ 📁 Source: {}", path).unwrap();
            }
        }

        // Show document blocks
        writeln!(output, "├─ 📦 Blocks: {} items", doc.blocks.len()).unwrap();

        for (i, block) in doc.blocks.iter().enumerate() {
            let is_last = i == doc.blocks.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            let indent = if is_last { "   " } else { "│  " };

            self.visualize_block(block, &mut output, prefix, indent, 1);
        }

        output
    }

    /// Visualize a single block
    fn visualize_block(
        &self,
        block: &Block,
        output: &mut String,
        prefix: &str,
        indent: &str,
        depth: usize,
    ) {
        if let Some(max_depth) = self.config.max_depth {
            if depth > max_depth {
                writeln!(output, "{} ... (max depth reached)", prefix).unwrap();
                return;
            }
        }

        match block {
            Block::Paragraph(para) => {
                writeln!(output, "{} 📝 Paragraph", prefix).unwrap();
                if !self.config.compact {
                    let text_content = self.extract_text_from_inlines(&para.content);
                    let preview = if text_content.len() > 50 {
                        format!("{}...", &text_content[..47])
                    } else {
                        text_content
                    };
                    writeln!(output, "{}   └─ 💭 \"{preview}\"", indent).unwrap();
                }

                if self.config.show_annotations && !para.annotations.is_empty() {
                    writeln!(
                        output,
                        "{}   └─ 🏷️ Annotations: {}",
                        indent,
                        para.annotations.len()
                    )
                    .unwrap();
                }
            }

            Block::List(list) => {
                writeln!(
                    output,
                    "{} 📋 List ({:?}, {} items)",
                    prefix,
                    list.decoration_type.style,
                    list.items.len()
                )
                .unwrap();

                if !self.config.compact {
                    for (i, item) in list.items.iter().enumerate() {
                        let is_last_item = i == list.items.len() - 1;
                        let item_prefix = if is_last_item { "└─" } else { "├─" };
                        let item_indent = if is_last_item { "   " } else { "│  " };

                        let text_content = self.extract_text_from_inlines(&item.content);
                        let preview = if text_content.len() > 30 {
                            format!("{}...", &text_content[..27])
                        } else {
                            text_content
                        };

                        writeln!(
                            output,
                            "{}{} {} \"{preview}\"",
                            indent, item_prefix, item.marker
                        )
                        .unwrap();

                        if let Some(ref nested) = item.nested {
                            self.visualize_container(
                                nested,
                                output,
                                &format!("{}{}", indent, item_indent),
                                depth + 1,
                            );
                        }
                    }
                }
            }

            Block::Session(session) => {
                let title_text = self.extract_text_from_inlines(&session.title.content);
                let numbering_info = if let Some(ref num) = session.title.numbering {
                    format!(" ({})", num.marker)
                } else {
                    String::new()
                };

                writeln!(
                    output,
                    "{} 📚 Session{}: \"{}\"",
                    prefix, numbering_info, title_text
                )
                .unwrap();

                if !session.content.content.is_empty() {
                    self.visualize_container(&session.content, output, indent, depth);
                }
            }

            Block::Container(container) => {
                writeln!(
                    output,
                    "{} 📦 Container ({} blocks)",
                    prefix,
                    container.content.len()
                )
                .unwrap();
                self.visualize_container(container, output, indent, depth);
            }

            Block::VerbatimBlock(verbatim) => {
                let format_info = verbatim
                    .format_hint
                    .as_ref()
                    .map(|f| format!(" ({})", f))
                    .unwrap_or_default();

                writeln!(output, "{} 💻 Verbatim{}", prefix, format_info).unwrap();

                if !self.config.compact {
                    let lines = verbatim.raw.lines().count();
                    let chars = verbatim.raw.len();
                    writeln!(
                        output,
                        "{}   └─ 📊 {} lines, {} chars",
                        indent, lines, chars
                    )
                    .unwrap();

                    if self.config.show_parameters && !verbatim.parameters.map.is_empty() {
                        writeln!(
                            output,
                            "{}   └─ ⚙️ Parameters: {}",
                            indent,
                            verbatim.parameters.map.len()
                        )
                        .unwrap();
                    }
                }
            }

            Block::Definition(def) => {
                let term_text = self.extract_text_from_inlines(&def.term.content);
                writeln!(output, "{} 📖 Definition: \"{}\"", prefix, term_text).unwrap();

                if !def.content.content.is_empty() {
                    self.visualize_container(&def.content, output, indent, depth);
                }

                if self.config.show_parameters && !def.parameters.map.is_empty() {
                    writeln!(
                        output,
                        "{}   └─ ⚙️ Parameters: {}",
                        indent,
                        def.parameters.map.len()
                    )
                    .unwrap();
                }
            }

            Block::BlankLine(_) => {
                if !self.config.compact {
                    writeln!(output, "{} ⬜ Blank Line", prefix).unwrap();
                }
            }
        }
    }

    /// Visualize a container's contents
    fn visualize_container(
        &self,
        container: &Container,
        output: &mut String,
        base_indent: &str,
        depth: usize,
    ) {
        for (i, block) in container.content.iter().enumerate() {
            let is_last = i == container.content.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            let indent = if is_last { "   " } else { "│  " };

            self.visualize_block(
                block,
                output,
                &format!("{}{}", base_indent, prefix),
                &format!("{}{}", base_indent, indent),
                depth + 1,
            );
        }
    }

    /// Extract text content from inline elements
    pub fn extract_text_from_inlines(&self, inlines: &[Inline]) -> String {
        inlines
            .iter()
            .map(|inline| match inline {
                Inline::TextLine(transform) => self.extract_text_from_transform(transform),
                Inline::Link { content, .. } => self.extract_text_from_inlines(content),
                Inline::Reference { content, .. } => content
                    .as_ref()
                    .map(|c| self.extract_text_from_inlines(c))
                    .unwrap_or_else(|| "[ref]".to_string()),
                Inline::Custom { content, .. } => self.extract_text_from_inlines(content),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Extract text content from text transforms
    fn extract_text_from_transform(&self, transform: &TextTransform) -> String {
        transform.text_content()
    }
}

/// AST comparison utilities for testing
#[cfg(feature = "new-ast")]
pub struct AstComparator;

#[cfg(feature = "new-ast")]
impl AstComparator {
    /// Compare two documents and return differences
    pub fn compare_documents(left: &Document, right: &Document) -> Vec<String> {
        let mut differences = Vec::new();

        // Compare block counts
        if left.blocks.len() != right.blocks.len() {
            differences.push(format!(
                "Block count differs: {} vs {}",
                left.blocks.len(),
                right.blocks.len()
            ));
        }

        // Compare metadata
        if left.meta.title != right.meta.title {
            differences.push("Document titles differ".to_string());
        }

        if left.meta.authors.len() != right.meta.authors.len() {
            differences.push(format!(
                "Author count differs: {} vs {}",
                left.meta.authors.len(),
                right.meta.authors.len()
            ));
        }

        // Compare blocks pairwise
        let min_blocks = left.blocks.len().min(right.blocks.len());
        for i in 0..min_blocks {
            let block_diffs = Self::compare_blocks(&left.blocks[i], &right.blocks[i]);
            for diff in block_diffs {
                differences.push(format!("Block {}: {}", i, diff));
            }
        }

        differences
    }

    /// Compare two blocks
    fn compare_blocks(left: &Block, right: &Block) -> Vec<String> {
        let mut differences = Vec::new();

        match (left, right) {
            (Block::Paragraph(l), Block::Paragraph(r)) => {
                if l.content.len() != r.content.len() {
                    differences.push(format!(
                        "Paragraph inline count differs: {} vs {}",
                        l.content.len(),
                        r.content.len()
                    ));
                }
            }
            (Block::List(l), Block::List(r)) => {
                if l.items.len() != r.items.len() {
                    differences.push(format!(
                        "List item count differs: {} vs {}",
                        l.items.len(),
                        r.items.len()
                    ));
                }
                if l.decoration_type.style != r.decoration_type.style {
                    differences.push("List decoration style differs".to_string());
                }
            }
            (Block::Session(l), Block::Session(r)) => {
                if l.title.numbering.as_ref().map(|n| &n.marker)
                    != r.title.numbering.as_ref().map(|n| &n.marker)
                {
                    differences.push("Session numbering differs".to_string());
                }
            }
            (Block::VerbatimBlock(l), Block::VerbatimBlock(r)) => {
                if l.raw != r.raw {
                    differences.push("Verbatim content differs".to_string());
                }
                if l.format_hint != r.format_hint {
                    differences.push("Verbatim format hint differs".to_string());
                }
            }
            (l, r) => {
                let l_type = Self::block_type_name(l);
                let r_type = Self::block_type_name(r);
                if l_type != r_type {
                    differences.push(format!("Block type differs: {} vs {}", l_type, r_type));
                }
            }
        }

        differences
    }

    /// Get block type name for comparison
    fn block_type_name(block: &Block) -> &'static str {
        match block {
            Block::Paragraph(_) => "Paragraph",
            Block::List(_) => "List",
            Block::Session(_) => "Session",
            Block::Container(_) => "Container",
            Block::VerbatimBlock(_) => "VerbatimBlock",
            Block::Definition(_) => "Definition",
            Block::BlankLine(_) => "BlankLine",
        }
    }
}

/// Statistics collection for AST analysis
#[cfg(feature = "new-ast")]
pub struct AstStatistics {
    pub paragraph_count: usize,
    pub list_count: usize,
    pub session_count: usize,
    pub verbatim_count: usize,
    pub definition_count: usize,
    pub max_nesting_depth: usize,
    pub total_characters: usize,
}

#[cfg(feature = "new-ast")]
impl AstStatistics {
    /// Collect statistics from a document
    pub fn from_document(doc: &Document) -> Self {
        let mut stats = Self {
            paragraph_count: 0,
            list_count: 0,
            session_count: 0,
            verbatim_count: 0,
            definition_count: 0,
            max_nesting_depth: 0,
            total_characters: 0,
        };

        for block in &doc.blocks {
            stats.collect_from_block(block, 1);
        }

        stats
    }

    /// Recursively collect statistics from blocks
    fn collect_from_block(&mut self, block: &Block, depth: usize) {
        self.max_nesting_depth = self.max_nesting_depth.max(depth);

        match block {
            Block::Paragraph(para) => {
                self.paragraph_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_inlines(&para.content)
                    .len();
            }
            Block::List(list) => {
                self.list_count += 1;
                for item in &list.items {
                    self.total_characters += AstTreeVisualizer::new()
                        .extract_text_from_inlines(&item.content)
                        .len();
                    if let Some(ref nested) = item.nested {
                        self.collect_from_container(nested, depth + 1);
                    }
                }
            }
            Block::Session(session) => {
                self.session_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_inlines(&session.title.content)
                    .len();
                self.collect_from_container(&session.content, depth + 1);
            }
            Block::Container(container) => {
                self.collect_from_container(container, depth);
            }
            Block::VerbatimBlock(verbatim) => {
                self.verbatim_count += 1;
                self.total_characters += verbatim.raw.len();
            }
            Block::Definition(def) => {
                self.definition_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_inlines(&def.term.content)
                    .len();
                self.collect_from_container(&def.content, depth + 1);
            }
            Block::BlankLine(_) => {
                // Blank lines don't add to character count
            }
        }
    }

    /// Collect statistics from a container
    fn collect_from_container(&mut self, container: &Container, depth: usize) {
        for block in &container.content {
            self.collect_from_block(block, depth + 1);
        }
    }
}

#[cfg(feature = "new-ast")]
impl std::fmt::Display for AstStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "📊 AST Statistics:")?;
        writeln!(f, "├─ 📝 Paragraphs: {}", self.paragraph_count)?;
        writeln!(f, "├─ 📋 Lists: {}", self.list_count)?;
        writeln!(f, "├─ 📚 Sessions: {}", self.session_count)?;
        writeln!(f, "├─ 💻 Verbatim blocks: {}", self.verbatim_count)?;
        writeln!(f, "├─ 📖 Definitions: {}", self.definition_count)?;
        writeln!(f, "├─ 🔢 Max nesting depth: {}", self.max_nesting_depth)?;
        writeln!(f, "└─ 🔤 Total characters: {}", self.total_characters)?;
        Ok(())
    }
}
