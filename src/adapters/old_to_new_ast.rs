//! Adapter for converting from old AST to new AST structure
//!
//! This module provides a bridge between the current string-based AST and the new
//! type-safe AST during the transition period. It handles the conversion of legacy
//! AstNode structures to the rich semantic AST types defined in the new system.
//!
//! # Bridge Architecture Pattern
//!
//! The adapter follows the bridge pattern to decouple the old and new AST systems:
//! - **Legacy Parser** → `AstNode` → **Adapter** → `Document` → **New Tooling**
//! - **Gradual Migration**: Individual parser phases can be ported incrementally
//! - **Testing Support**: End-to-end validation from TXXT string to final AST
//!
//! # Conversion Strategy
//!
//! ## Phase 1: Structural Mapping
//! Maps old node types to new AST blocks based on `node_type` field:
//! - `"paragraph"` → `Block::Paragraph`
//! - `"list"` → `Block::List` with sophisticated styling
//! - `"session"` → `Block::Session` with hierarchical numbering
//! - `"container"` → `Container` with proper nesting
//!
//! ## Phase 2: Metadata Extraction  
//! Converts old attributes to new metadata systems:
//! - Node attributes → `Parameters` with semantic keys
//! - Special attributes → `AssemblyInfo` for processing metadata
//! - Content analysis → Rich `MetaValue` structures
//!
//! ## Phase 3: Token Synthesis
//! Creates synthetic tokens for language server support:
//! - Source positions from `start_line`/`end_line` → `SourceSpan`
//! - Text content → Character-precise `Token` sequences
//! - Estimated positions when exact data unavailable

#[cfg(feature = "new-ast")]
use crate::ast::{
    base::{AssemblyInfo, Document as NewDocument, Meta, MetaValue, ProcessingStats},
    blocks::{
        Block, Definition, DefinitionTerm, List, ListDecorationType, ListItem, VerbatimBlock,
        VerbatimType,
    },
    inlines::{Inline, Text, TextTransform},
    parameters::Parameters,
    structure::{
        BlankLine, Container, NumberingForm, NumberingStyle, Paragraph, Session, SessionNumbering,
        SessionTitle,
    },
    tokens::{Position, SourceSpan, Token, TokenSequence},
};

use crate::ast::{AstNode, Document as OldDocument};
use std::collections::HashMap;

/// Converts old Document to new Document structure
///
/// This is the main entry point for the adapter. It processes the entire
/// old AST tree and converts it to the new rich semantic structure.
#[cfg(feature = "new-ast")]
pub fn convert_document(old_doc: &OldDocument) -> NewDocument {
    let mut converter = AstConverter::new(&old_doc.source);

    // Convert the root node and extract blocks
    let blocks = converter.convert_node_to_blocks(&old_doc.root);

    // Extract metadata from document-level attributes and annotations
    let meta = converter.extract_document_metadata(&old_doc.root);

    // Create assembly info with conversion metadata
    let assembly_info = AssemblyInfo {
        parser_version: env!("CARGO_PKG_VERSION").to_string(),
        source_path: Some(old_doc.source.clone()),
        processed_at: Some(chrono::Utc::now().to_rfc3339()),
        stats: converter.stats,
    };

    NewDocument {
        meta,
        blocks,
        annotations: Vec::new(), // Annotations will be populated during assembly phase
        assembly_info,
    }
}

/// Internal converter state for tracking conversion process
#[cfg(feature = "new-ast")]
struct AstConverter {
    /// Source file path for token synthesis
    #[allow(dead_code)]
    source_path: String,

    /// Processing statistics
    stats: ProcessingStats,

    /// Current line number for token position estimation
    current_line: usize,
}

#[cfg(feature = "new-ast")]
impl AstConverter {
    fn new(source_path: &str) -> Self {
        Self {
            source_path: source_path.to_string(),
            stats: ProcessingStats::default(),
            current_line: 0,
        }
    }

    /// Convert a single AstNode to one or more Block elements
    fn convert_node_to_blocks(&mut self, node: &AstNode) -> Vec<Block> {
        self.stats.token_count += 1; // Rough approximation

        match node.node_type.as_str() {
            "document" => {
                // Document node: convert all children
                let mut blocks = Vec::new();
                for child in &node.children {
                    blocks.extend(self.convert_node_to_blocks(child));
                }
                self.stats.block_count += blocks.len();
                blocks
            }

            "paragraph" => {
                vec![Block::Paragraph(self.convert_paragraph(node))]
            }

            "list" => {
                vec![Block::List(self.convert_list(node))]
            }

            "session" => {
                vec![Block::Session(self.convert_session(node))]
            }

            "container" => {
                vec![Block::Container(self.convert_container(node))]
            }

            "verbatim" => {
                vec![Block::VerbatimBlock(self.convert_verbatim(node))]
            }

            "definition" => {
                vec![Block::Definition(self.convert_definition(node))]
            }

            "blank_line" => {
                vec![Block::BlankLine(self.convert_blank_line(node))]
            }

            _ => {
                // Unknown node type: convert to paragraph as fallback
                vec![Block::Paragraph(self.convert_unknown_as_paragraph(node))]
            }
        }
    }

    /// Convert paragraph node
    fn convert_paragraph(&mut self, node: &AstNode) -> Paragraph {
        let content = self.convert_inline_content(node);
        let tokens = self.synthesize_tokens_for_node(node);

        Paragraph {
            content,
            annotations: Vec::new(), // Will be populated during assembly
            tokens,
        }
    }

    /// Convert list node with sophisticated styling
    fn convert_list(&mut self, node: &AstNode) -> List {
        let mut items = Vec::new();
        let mut decoration_type = ListDecorationType::default();

        // Extract decoration from attributes or first item
        if let Some(style) = node.attributes.get("list_style") {
            decoration_type.style = self.parse_numbering_style(style);
        }

        if let Some(form) = node.attributes.get("list_form") {
            decoration_type.form = self.parse_numbering_form(form);
        }

        // Convert child nodes to list items
        for child in &node.children {
            if child.node_type == "list_item" {
                items.push(self.convert_list_item(child));
            }
        }

        let tokens = self.synthesize_tokens_for_node(node);

        List {
            decoration_type,
            items,
            annotations: Vec::new(),
            tokens,
        }
    }

    /// Convert list item with marker preservation
    fn convert_list_item(&mut self, node: &AstNode) -> ListItem {
        let marker = node
            .attributes
            .get("marker")
            .unwrap_or(&"-".to_string())
            .clone();

        let content = self.convert_inline_content(node);
        let tokens = self.synthesize_tokens_for_node(node);

        // Check for nested content
        let nested = if node.children.iter().any(|child| child.node_type != "text") {
            let mut nested_blocks = Vec::new();
            for child in &node.children {
                if child.node_type != "text" {
                    nested_blocks.extend(self.convert_node_to_blocks(child));
                }
            }
            if !nested_blocks.is_empty() {
                Some(Container {
                    content: nested_blocks,
                    annotations: Vec::new(),
                })
            } else {
                None
            }
        } else {
            None
        };

        ListItem {
            marker,
            content,
            nested,
            annotations: Vec::new(),
            tokens,
        }
    }

    /// Convert session node with hierarchical numbering
    fn convert_session(&mut self, node: &AstNode) -> Session {
        // Extract title content
        let title_text = node
            .attributes
            .get("title")
            .or(node.content.as_ref())
            .unwrap_or(&"".to_string())
            .clone();

        let title_content = if !title_text.is_empty() {
            vec![Inline::TextLine(TextTransform::Identity(Text::simple(
                &title_text,
            )))]
        } else {
            Vec::new()
        };

        // Extract numbering information
        let numbering = node
            .attributes
            .get("numbering")
            .map(|marker| SessionNumbering {
                marker: marker.clone(),
                style: self.infer_numbering_style(marker),
                form: self.infer_numbering_form(marker),
            });

        let title = SessionTitle {
            content: title_content,
            numbering,
            tokens: self.synthesize_tokens_for_content(&title_text),
        };

        // Convert child content
        let mut content_blocks = Vec::new();
        for child in &node.children {
            content_blocks.extend(self.convert_node_to_blocks(child));
        }

        let content = Container {
            content: content_blocks,
            annotations: Vec::new(),
        };

        Session {
            title,
            content,
            annotations: Vec::new(),
        }
    }

    /// Convert container node
    fn convert_container(&mut self, node: &AstNode) -> Container {
        let mut content = Vec::new();

        for child in &node.children {
            content.extend(self.convert_node_to_blocks(child));
        }

        Container {
            content,
            annotations: Vec::new(),
        }
    }

    /// Convert verbatim block
    fn convert_verbatim(&mut self, node: &AstNode) -> VerbatimBlock {
        let raw = node.content.as_ref().unwrap_or(&String::new()).clone();

        let verbatim_type = match node.attributes.get("verbatim_type") {
            Some(t) if t == "stretched" => VerbatimType::Stretched,
            _ => VerbatimType::InFlow,
        };

        let format_hint = node.attributes.get("format").cloned();
        let parameters = self.extract_parameters(&node.attributes);
        let tokens = self.synthesize_tokens_for_node(node);

        VerbatimBlock {
            raw,
            verbatim_type,
            format_hint,
            parameters,
            annotations: Vec::new(),
            tokens,
        }
    }

    /// Convert definition node
    fn convert_definition(&mut self, node: &AstNode) -> Definition {
        // Extract term from attributes or first child
        let term_text = node
            .attributes
            .get("term")
            .or_else(|| {
                node.children
                    .first()
                    .and_then(|child| child.content.as_ref())
            })
            .unwrap_or(&"".to_string())
            .clone();

        let term_content = if !term_text.is_empty() {
            vec![Inline::TextLine(TextTransform::Identity(Text::simple(
                &term_text,
            )))]
        } else {
            Vec::new()
        };

        let term = DefinitionTerm {
            content: term_content,
            tokens: self.synthesize_tokens_for_content(&term_text),
        };

        // Convert definition content (skip first child if it was the term)
        let mut content_blocks = Vec::new();
        let skip_first = node
            .children
            .first()
            .map(|child| child.content.as_ref() == Some(&term_text))
            .unwrap_or(false);

        for child in node.children.iter().skip(if skip_first { 1 } else { 0 }) {
            content_blocks.extend(self.convert_node_to_blocks(child));
        }

        let content = Container {
            content: content_blocks,
            annotations: Vec::new(),
        };

        let parameters = self.extract_parameters(&node.attributes);
        let tokens = self.synthesize_tokens_for_node(node);

        Definition {
            term,
            content,
            parameters,
            annotations: Vec::new(),
            tokens,
        }
    }

    /// Convert blank line
    fn convert_blank_line(&mut self, node: &AstNode) -> BlankLine {
        BlankLine {
            tokens: self.synthesize_tokens_for_node(node),
        }
    }

    /// Convert unknown node type as paragraph fallback
    fn convert_unknown_as_paragraph(&mut self, node: &AstNode) -> Paragraph {
        let content = self.convert_inline_content(node);
        let tokens = self.synthesize_tokens_for_node(node);

        Paragraph {
            content,
            annotations: Vec::new(),
            tokens,
        }
    }

    /// Convert node content to inline elements
    fn convert_inline_content(&mut self, node: &AstNode) -> Vec<Inline> {
        if let Some(text_content) = &node.content {
            if !text_content.is_empty() {
                return vec![Inline::TextLine(TextTransform::Identity(Text::simple(
                    text_content,
                )))];
            }
        }

        // Convert text children to inline content
        let mut inlines = Vec::new();
        for child in &node.children {
            if child.node_type == "text" {
                if let Some(text) = &child.content {
                    inlines.push(Inline::TextLine(TextTransform::Identity(Text::simple(
                        text,
                    ))));
                }
            }
        }

        if inlines.is_empty() {
            // Fallback: create empty text
            vec![Inline::TextLine(TextTransform::Identity(Text::simple("")))]
        } else {
            inlines
        }
    }

    /// Extract document metadata from root node
    fn extract_document_metadata(&self, root: &AstNode) -> Meta {
        let mut meta = Meta::default();

        // Extract title from attributes
        if let Some(title) = root.attributes.get("title") {
            meta.title = Some(MetaValue::String(title.clone()));
        }

        // Extract authors
        if let Some(authors) = root.attributes.get("authors") {
            let author_list = authors
                .split(',')
                .map(|author| MetaValue::String(author.trim().to_string()))
                .collect();
            meta.authors = author_list;
        }

        // Extract date
        if let Some(date) = root.attributes.get("date") {
            meta.date = Some(MetaValue::String(date.clone()));
        }

        // Extract custom metadata
        for (key, value) in &root.attributes {
            if !matches!(key.as_str(), "title" | "authors" | "date") {
                meta.custom
                    .insert(key.clone(), MetaValue::String(value.clone()));
            }
        }

        meta
    }

    /// Extract parameters from node attributes
    fn extract_parameters(&self, attributes: &HashMap<String, String>) -> Parameters {
        let mut params = HashMap::new();

        for (key, value) in attributes {
            // Filter out internal attributes
            if !matches!(
                key.as_str(),
                "marker"
                    | "list_style"
                    | "list_form"
                    | "verbatim_type"
                    | "format"
                    | "term"
                    | "title"
                    | "numbering"
            ) {
                params.insert(key.clone(), value.clone());
            }
        }

        Parameters {
            map: params,
            tokens: TokenSequence::new(),
        }
    }

    /// Synthesize token sequence for a node
    fn synthesize_tokens_for_node(&mut self, node: &AstNode) -> TokenSequence {
        if let Some(content) = &node.content {
            self.synthesize_tokens_for_content(content)
        } else {
            TokenSequence::new()
        }
    }

    /// Synthesize tokens for text content
    fn synthesize_tokens_for_content(&mut self, content: &str) -> TokenSequence {
        if content.is_empty() {
            return TokenSequence::new();
        }

        let start_pos = Position {
            row: self.current_line,
            column: 0,
        };

        let end_pos = Position {
            row: self.current_line,
            column: content.len(),
        };

        let span = SourceSpan {
            start: start_pos,
            end: end_pos,
        };

        let token = Token::Text {
            content: content.to_string(),
            span,
        };

        self.current_line += content.chars().filter(|&c| c == '\n').count();

        TokenSequence {
            tokens: vec![token],
        }
    }

    /// Parse numbering style from string
    fn parse_numbering_style(&self, style: &str) -> NumberingStyle {
        match style.to_lowercase().as_str() {
            "numerical" => NumberingStyle::Numerical,
            "alphabetical_lower" => NumberingStyle::AlphabeticalLower,
            "alphabetical_upper" => NumberingStyle::AlphabeticalUpper,
            "roman_lower" => NumberingStyle::RomanLower,
            "roman_upper" => NumberingStyle::RomanUpper,
            _ => NumberingStyle::Plain,
        }
    }

    /// Parse numbering form from string
    fn parse_numbering_form(&self, form: &str) -> NumberingForm {
        match form.to_lowercase().as_str() {
            "full" => NumberingForm::Full,
            _ => NumberingForm::Short,
        }
    }

    /// Infer numbering style from marker
    fn infer_numbering_style(&self, marker: &str) -> NumberingStyle {
        if marker.chars().any(|c| c.is_ascii_digit()) {
            NumberingStyle::Numerical
        } else if marker.chars().any(|c| c.is_ascii_lowercase()) {
            if marker.contains('i') || marker.contains('v') || marker.contains('x') {
                NumberingStyle::RomanLower
            } else {
                NumberingStyle::AlphabeticalLower
            }
        } else if marker.chars().any(|c| c.is_ascii_uppercase()) {
            if marker.contains('I') || marker.contains('V') || marker.contains('X') {
                NumberingStyle::RomanUpper
            } else {
                NumberingStyle::AlphabeticalUpper
            }
        } else {
            NumberingStyle::Plain
        }
    }

    /// Infer numbering form from marker
    fn infer_numbering_form(&self, marker: &str) -> NumberingForm {
        if marker.matches('.').count() > 1 {
            NumberingForm::Full
        } else {
            NumberingForm::Short
        }
    }
}

/// Test utilities for the adapter
#[cfg(all(test, feature = "new-ast"))]
mod tests {
    use super::*;

    #[test]
    fn test_simple_document_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());
        old_doc.root.add_child(AstNode::with_content(
            "paragraph".to_string(),
            "Hello world".to_string(),
        ));

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);
        assert!(matches!(new_doc.blocks[0], Block::Paragraph(_)));
    }

    #[test]
    fn test_list_conversion() {
        let mut old_doc = OldDocument::new("test.txxt".to_string());
        let mut list_node = AstNode::new("list".to_string());
        list_node.set_attribute("list_style".to_string(), "numerical".to_string());

        let mut item1 = AstNode::new("list_item".to_string());
        item1.set_attribute("marker".to_string(), "1.".to_string());
        item1.content = Some("First item".to_string());
        list_node.add_child(item1);

        old_doc.root.add_child(list_node);

        let new_doc = convert_document(&old_doc);

        assert_eq!(new_doc.blocks.len(), 1);
        if let Block::List(list) = &new_doc.blocks[0] {
            assert_eq!(list.decoration_type.style, NumberingStyle::Numerical);
            assert_eq!(list.items.len(), 1);
            assert_eq!(list.items[0].marker, "1.");
        } else {
            panic!("Expected List block");
        }
    }
}
