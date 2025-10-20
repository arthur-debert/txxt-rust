//! AST debugging and visualization utilities
//!
//! This module provides tools for visualizing and debugging AST structures,
//! including tree printing, structure comparison, and content inspection.

use crate::ast::{
    elements::{
        containers::{content::ContentContainerElement, ContentContainer},
        formatting::inlines::TextTransform,
        session::{session_container::SessionContainerElement, SessionContainer},
    },
    Document,
};

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
pub struct AstTreeVisualizer {
    config: TreeConfig,
}

impl Default for AstTreeVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

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

        writeln!(output, "Document").unwrap();

        // Show document metadata
        if !self.config.compact {
            if let Some(ref title) = doc.meta.title {
                writeln!(output, "├─ Title: {:?}", title).unwrap();
            }
            if !doc.meta.authors.is_empty() {
                writeln!(output, "├─ Authors: {} items", doc.meta.authors.len()).unwrap();
            }
            if !doc.meta.custom.is_empty() {
                writeln!(
                    output,
                    "├─ Custom metadata: {} items",
                    doc.meta.custom.len()
                )
                .unwrap();
            }
        }

        // Show assembly info
        if !self.config.compact {
            writeln!(output, "├─ Parser: {}", doc.assembly_info.parser_version).unwrap();
            if let Some(ref path) = doc.assembly_info.source_path {
                writeln!(output, "├─ Source: {}", path).unwrap();
            }
        }

        // Show document blocks
        writeln!(output, "├─ Blocks: {} items", doc.content.content.len()).unwrap();

        for (i, element) in doc.content.content.iter().enumerate() {
            let is_last = i == doc.content.content.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            let indent = if is_last { "   " } else { "│  " };

            self.visualize_session_element(element, &mut output, prefix, indent, 1);
        }

        output
    }

    /// Visualize a session container element
    fn visualize_session_element(
        &self,
        element: &SessionContainerElement,
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

        match element {
            SessionContainerElement::Paragraph(para) => {
                writeln!(output, "{} Paragraph", prefix).unwrap();
                if !self.config.compact {
                    let text_content = self.extract_text_from_transforms(&para.content);
                    let preview = if text_content.len() > 50 {
                        format!("{}...", &text_content[..47])
                    } else {
                        text_content
                    };
                    writeln!(output, "{}   └─ \"{preview}\"", indent).unwrap();
                }

                if self.config.show_annotations && !para.annotations.is_empty() {
                    writeln!(
                        output,
                        "{}   └─ Annotations: {}",
                        indent,
                        para.annotations.len()
                    )
                    .unwrap();
                }
            }

            SessionContainerElement::List(list) => {
                writeln!(
                    output,
                    "{} List ({:?}, {} items)",
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

                        let text_content = self.extract_text_from_transforms(&item.content);
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
                            self.visualize_content_container(
                                nested,
                                output,
                                &format!("{}{}", indent, item_indent),
                                depth + 1,
                            );
                        }
                    }
                }
            }

            SessionContainerElement::Session(session) => {
                let title_text = self.extract_text_from_transforms(&session.title.content);
                let numbering_info = if let Some(ref num) = session.title.numbering {
                    format!(" ({})", num.marker)
                } else {
                    String::new()
                };

                writeln!(
                    output,
                    "{} Session{}: \"{}\"",
                    prefix, numbering_info, title_text
                )
                .unwrap();

                if !session.content.content.is_empty() {
                    self.visualize_session_container(&session.content, output, indent, depth);
                }
            }

            SessionContainerElement::Verbatim(verbatim) => {
                let format_info = match &verbatim.verbatim_type {
                    crate::ast::elements::verbatim::VerbatimType::InFlow => {
                        " (in-flow)".to_string()
                    }
                    crate::ast::elements::verbatim::VerbatimType::Stretched => {
                        " (stretched)".to_string()
                    }
                };

                writeln!(output, "{} Verbatim{}", prefix, format_info).unwrap();

                if !self.config.compact {
                    let lines =
                        verbatim.content.ignore_lines.len() + verbatim.content.blank_lines.len();
                    let chars: usize = verbatim
                        .content
                        .ignore_lines
                        .iter()
                        .map(|line| line.content.len())
                        .sum();
                    writeln!(output, "{}   └─ {} lines, {} chars", indent, lines, chars).unwrap();

                    if self.config.show_parameters && !verbatim.parameters.map.is_empty() {
                        writeln!(
                            output,
                            "{}   └─ Parameters: {}",
                            indent,
                            verbatim.parameters.map.len()
                        )
                        .unwrap();
                    }
                }
            }

            SessionContainerElement::Definition(def) => {
                let term_text = self.extract_text_from_transforms(&def.term.content);
                writeln!(output, "{} Definition: \"{}\"", prefix, term_text).unwrap();

                if !def.content.content.is_empty() {
                    self.visualize_content_container(&def.content, output, indent, depth);
                }

                if self.config.show_parameters && !def.parameters.map.is_empty() {
                    writeln!(
                        output,
                        "{}   └─ Parameters: {}",
                        indent,
                        def.parameters.map.len()
                    )
                    .unwrap();
                }
            }

            SessionContainerElement::Annotation(annotation) => {
                writeln!(output, "{} Annotation", prefix).unwrap();

                if !self.config.compact {
                    let text_content =
                        self.extract_text_from_annotation_content(&annotation.content);
                    let preview = if text_content.len() > 50 {
                        format!("{}...", &text_content[..47])
                    } else {
                        text_content
                    };
                    writeln!(output, "{}   └─ \"{preview}\"", indent).unwrap();
                }
            }

            SessionContainerElement::BlankLine(_) => {
                if !self.config.compact {
                    writeln!(output, "{} Blank Line", prefix).unwrap();
                }
            }

            SessionContainerElement::ContentContainer(container) => {
                writeln!(
                    output,
                    "{} ContentContainer ({} elements)",
                    prefix,
                    container.content.len()
                )
                .unwrap();
                self.visualize_content_container(container, output, indent, depth);
            }

            SessionContainerElement::SessionContainer(container) => {
                writeln!(
                    output,
                    "{} SessionContainer ({} elements)",
                    prefix,
                    container.content.len()
                )
                .unwrap();
                self.visualize_session_container(container, output, indent, depth);
            }
        }
    }

    /// Visualize a session container's contents
    fn visualize_session_container(
        &self,
        container: &SessionContainer,
        output: &mut String,
        base_indent: &str,
        depth: usize,
    ) {
        for (i, element) in container.content.iter().enumerate() {
            let is_last = i == container.content.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            let indent = if is_last { "   " } else { "│  " };

            self.visualize_session_element(
                element,
                output,
                &format!("{}{}", base_indent, prefix),
                &format!("{}{}", base_indent, indent),
                depth + 1,
            );
        }
    }

    /// Visualize a content container's contents
    fn visualize_content_container(
        &self,
        container: &ContentContainer,
        output: &mut String,
        base_indent: &str,
        depth: usize,
    ) {
        for (i, element) in container.content.iter().enumerate() {
            let is_last = i == container.content.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };
            let indent = if is_last { "   " } else { "│  " };

            self.visualize_content_element(
                element,
                output,
                &format!("{}{}", base_indent, prefix),
                &format!("{}{}", base_indent, indent),
                depth + 1,
            );
        }
    }

    /// Visualize a content container element
    fn visualize_content_element(
        &self,
        element: &ContentContainerElement,
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

        match element {
            ContentContainerElement::Paragraph(para) => {
                writeln!(output, "{} Paragraph", prefix).unwrap();
                if !self.config.compact {
                    let text_content = self.extract_text_from_transforms(&para.content);
                    let preview = if text_content.len() > 50 {
                        format!("{}...", &text_content[..47])
                    } else {
                        text_content
                    };
                    writeln!(output, "{}   └─ \"{preview}\"", indent).unwrap();
                }
            }
            ContentContainerElement::List(list) => {
                writeln!(
                    output,
                    "{} List ({:?}, {} items)",
                    prefix,
                    list.decoration_type.style,
                    list.items.len()
                )
                .unwrap();
            }
            ContentContainerElement::Verbatim(_verbatim) => {
                writeln!(output, "{} Verbatim", prefix).unwrap();
            }
            ContentContainerElement::Definition(def) => {
                let term_text = self.extract_text_from_transforms(&def.term.content);
                writeln!(output, "{} Definition: \"{}\"", prefix, term_text).unwrap();
            }
            ContentContainerElement::Annotation(_annotation) => {
                writeln!(output, "{} Annotation", prefix).unwrap();
            }
            ContentContainerElement::BlankLine(_) => {
                if !self.config.compact {
                    writeln!(output, "{} Blank Line", prefix).unwrap();
                }
            }

            ContentContainerElement::Container(container) => {
                writeln!(
                    output,
                    "{} ContentContainer ({} elements)",
                    prefix,
                    container.content.len()
                )
                .unwrap();
                self.visualize_content_container(container, output, indent, depth);
            }
        }
    }

    /// Extract text content from text transforms
    pub fn extract_text_from_transforms(&self, transforms: &[TextTransform]) -> String {
        transforms
            .iter()
            .map(|transform| transform.text_content())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Extract text content from annotation content
    fn extract_text_from_annotation_content(
        &self,
        content: &crate::ast::elements::annotation::AnnotationContent,
    ) -> String {
        match content {
            crate::ast::elements::annotation::AnnotationContent::Inline(transforms) => {
                self.extract_text_from_transforms(transforms)
            }
            crate::ast::elements::annotation::AnnotationContent::Block(_) => {
                "[block annotation]".to_string()
            }
        }
    }
}

/// AST comparison utilities for testing
pub struct AstComparator;

impl AstComparator {
    /// Compare two documents and return differences
    pub fn compare_documents(left: &Document, right: &Document) -> Vec<String> {
        let mut differences = Vec::new();

        // Compare element counts
        if left.content.content.len() != right.content.content.len() {
            differences.push(format!(
                "Element count differs: {} vs {}",
                left.content.content.len(),
                right.content.content.len()
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

        // Compare elements pairwise
        let min_elements = left.content.content.len().min(right.content.content.len());
        for i in 0..min_elements {
            let element_diffs =
                Self::compare_session_elements(&left.content.content[i], &right.content.content[i]);
            for diff in element_diffs {
                differences.push(format!("Element {}: {}", i, diff));
            }
        }

        differences
    }

    /// Compare two session container elements
    fn compare_session_elements(
        left: &SessionContainerElement,
        right: &SessionContainerElement,
    ) -> Vec<String> {
        let mut differences = Vec::new();

        match (left, right) {
            (SessionContainerElement::Paragraph(l), SessionContainerElement::Paragraph(r)) => {
                if l.content.len() != r.content.len() {
                    differences.push(format!(
                        "Paragraph transform count differs: {} vs {}",
                        l.content.len(),
                        r.content.len()
                    ));
                }
            }
            (SessionContainerElement::List(l), SessionContainerElement::List(r)) => {
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
            (SessionContainerElement::Session(l), SessionContainerElement::Session(r)) => {
                if l.title.numbering.as_ref().map(|n| &n.marker)
                    != r.title.numbering.as_ref().map(|n| &n.marker)
                {
                    differences.push("Session numbering differs".to_string());
                }
            }
            (SessionContainerElement::Verbatim(l), SessionContainerElement::Verbatim(r)) => {
                if l.content.ignore_lines != r.content.ignore_lines {
                    differences.push("Verbatim content differs".to_string());
                }
            }
            (SessionContainerElement::Definition(l), SessionContainerElement::Definition(r)) => {
                if l.term.content.len() != r.term.content.len() {
                    differences.push("Definition term differs".to_string());
                }
            }
            (l, r) => {
                let l_type = Self::session_element_type_name(l);
                let r_type = Self::session_element_type_name(r);
                if l_type != r_type {
                    differences.push(format!("Element type differs: {} vs {}", l_type, r_type));
                }
            }
        }

        differences
    }

    /// Get session element type name for comparison
    fn session_element_type_name(element: &SessionContainerElement) -> &'static str {
        match element {
            SessionContainerElement::Paragraph(_) => "Paragraph",
            SessionContainerElement::List(_) => "List",
            SessionContainerElement::Session(_) => "Session",
            SessionContainerElement::Verbatim(_) => "Verbatim",
            SessionContainerElement::Definition(_) => "Definition",
            SessionContainerElement::Annotation(_) => "Annotation",
            SessionContainerElement::BlankLine(_) => "BlankLine",
            SessionContainerElement::ContentContainer(_) => "ContentContainer",
            SessionContainerElement::SessionContainer(_) => "SessionContainer",
        }
    }
}

/// Statistics collection for AST analysis
pub struct AstStatistics {
    pub paragraph_count: usize,
    pub list_count: usize,
    pub session_count: usize,
    pub verbatim_count: usize,
    pub definition_count: usize,
    pub max_nesting_depth: usize,
    pub total_characters: usize,
}

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

        for element in &doc.content.content {
            stats.collect_from_session_element(element, 1);
        }

        stats
    }

    /// Recursively collect statistics from session elements
    fn collect_from_session_element(&mut self, element: &SessionContainerElement, depth: usize) {
        self.max_nesting_depth = self.max_nesting_depth.max(depth);

        match element {
            SessionContainerElement::Paragraph(para) => {
                self.paragraph_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_transforms(&para.content)
                    .len();
            }
            SessionContainerElement::List(list) => {
                self.list_count += 1;
                for item in &list.items {
                    self.total_characters += AstTreeVisualizer::new()
                        .extract_text_from_transforms(&item.content)
                        .len();
                    if let Some(ref nested) = item.nested {
                        self.collect_from_content_container(nested, depth + 1);
                    }
                }
            }
            SessionContainerElement::Session(session) => {
                self.session_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_transforms(&session.title.content)
                    .len();
                self.collect_from_session_container(&session.content, depth + 1);
            }
            SessionContainerElement::Verbatim(verbatim) => {
                self.verbatim_count += 1;
                self.total_characters += verbatim
                    .content
                    .ignore_lines
                    .iter()
                    .map(|line| line.content.len())
                    .sum::<usize>();
            }
            SessionContainerElement::Definition(def) => {
                self.definition_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_transforms(&def.term.content)
                    .len();
                self.collect_from_content_container(&def.content, depth + 1);
            }
            SessionContainerElement::Annotation(_) => {
                // Annotations don't count as primary content
            }
            SessionContainerElement::BlankLine(_) => {
                // Blank lines don't add to character count
            }
            SessionContainerElement::ContentContainer(container) => {
                self.collect_from_content_container(container, depth);
            }
            SessionContainerElement::SessionContainer(container) => {
                self.collect_from_session_container(container, depth);
            }
        }
    }

    /// Collect statistics from a session container
    fn collect_from_session_container(&mut self, container: &SessionContainer, depth: usize) {
        for element in &container.content {
            self.collect_from_session_element(element, depth + 1);
        }
    }

    /// Collect statistics from a content container
    fn collect_from_content_container(&mut self, container: &ContentContainer, depth: usize) {
        for element in &container.content {
            self.collect_from_content_element(element, depth + 1);
        }
    }

    /// Collect statistics from content elements
    fn collect_from_content_element(&mut self, element: &ContentContainerElement, depth: usize) {
        self.max_nesting_depth = self.max_nesting_depth.max(depth);

        match element {
            ContentContainerElement::Paragraph(para) => {
                self.paragraph_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_transforms(&para.content)
                    .len();
            }
            ContentContainerElement::List(list) => {
                self.list_count += 1;
                for item in &list.items {
                    self.total_characters += AstTreeVisualizer::new()
                        .extract_text_from_transforms(&item.content)
                        .len();
                    if let Some(ref nested) = item.nested {
                        self.collect_from_content_container(nested, depth + 1);
                    }
                }
            }
            ContentContainerElement::Verbatim(verbatim) => {
                self.verbatim_count += 1;
                self.total_characters += verbatim
                    .content
                    .ignore_lines
                    .iter()
                    .map(|line| line.content.len())
                    .sum::<usize>();
            }
            ContentContainerElement::Definition(def) => {
                self.definition_count += 1;
                self.total_characters += AstTreeVisualizer::new()
                    .extract_text_from_transforms(&def.term.content)
                    .len();
                self.collect_from_content_container(&def.content, depth + 1);
            }
            ContentContainerElement::Annotation(_) => {
                // Annotations don't count as primary content
            }
            ContentContainerElement::BlankLine(_) => {
                // Blank lines don't add to character count
            }
            ContentContainerElement::Container(container) => {
                self.collect_from_content_container(container, depth);
            }
        }
    }
}

impl std::fmt::Display for AstStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "AST Statistics:")?;
        writeln!(f, "├─ Paragraphs: {}", self.paragraph_count)?;
        writeln!(f, "├─ Lists: {}", self.list_count)?;
        writeln!(f, "├─ Sessions: {}", self.session_count)?;
        writeln!(f, "├─ Verbatim blocks: {}", self.verbatim_count)?;
        writeln!(f, "├─ Definitions: {}", self.definition_count)?;
        writeln!(f, "├─ Max nesting depth: {}", self.max_nesting_depth)?;
        writeln!(f, "└─ Total characters: {}", self.total_characters)?;
        Ok(())
    }
}
