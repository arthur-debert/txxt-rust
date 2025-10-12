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
                // Check if this is actually a session (numbered list items with content containers)
                if self.is_session_list(node) {
                    self.convert_list_as_sessions(node)
                } else {
                    vec![Block::List(self.convert_list(node))]
                }
            }

            "session" => {
                vec![Block::Session(self.convert_session(node))]
            }

            "container" | "content_container" => {
                vec![Block::Container(self.convert_container(node))]
            }

            "verbatim" => {
                // Check if this is actually a definition (verbatim with title)
                if node.attributes.contains_key("title") {
                    vec![Block::Definition(self.convert_verbatim_as_definition(node))]
                } else {
                    vec![Block::VerbatimBlock(self.convert_verbatim(node))]
                }
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

        // If no explicit style was set, infer from first item's marker
        if decoration_type.style == NumberingStyle::Plain && !items.is_empty() {
            decoration_type.style = self.infer_numbering_style(&items[0].marker);
            decoration_type.form = self.infer_numbering_form(&items[0].marker);
        }

        let tokens = self.synthesize_tokens_for_node(node);

        List {
            decoration_type,
            items,
            annotations: Vec::new(),
            tokens,
        }
    }

    /// Check if a list should be treated as sessions
    fn is_session_list(&self, node: &AstNode) -> bool {
        // Heuristic: if any list item has a numbered marker and content_container children,
        // treat the whole list as sessions
        node.children.iter().any(|child| {
            if child.node_type == "list_item" {
                // Check if it has a numbered marker
                let has_numbered_marker = child
                    .attributes
                    .get("marker")
                    .map(|marker| marker.trim().chars().next().unwrap_or('-').is_ascii_digit())
                    .unwrap_or(false);

                // Check if it has content_container children (indicating nested content)
                let has_content_container = child
                    .children
                    .iter()
                    .any(|grandchild| grandchild.node_type == "content_container");

                has_numbered_marker && has_content_container
            } else {
                false
            }
        })
    }

    /// Convert a list that should be treated as sessions
    fn convert_list_as_sessions(&mut self, node: &AstNode) -> Vec<Block> {
        let mut sessions = Vec::new();

        for child in &node.children {
            if child.node_type == "list_item" {
                sessions.push(Block::Session(self.convert_list_item_as_session(child)));
            }
        }

        sessions
    }

    /// Convert a list item as a session
    fn convert_list_item_as_session(&mut self, node: &AstNode) -> Session {
        // Extract title from the list item content
        let title_text = node.content.as_ref().unwrap_or(&"".to_string()).clone();

        let title_content = if !title_text.is_empty() {
            vec![Inline::TextLine(TextTransform::Identity(Text::simple(
                &title_text,
            )))]
        } else {
            Vec::new()
        };

        // Extract numbering from marker
        let numbering = node.attributes.get("marker").map(|marker| {
            let trimmed_marker = marker.trim();
            SessionNumbering {
                marker: trimmed_marker.to_string(),
                style: self.infer_numbering_style(trimmed_marker),
                form: self.infer_numbering_form(trimmed_marker),
            }
        });

        let title = SessionTitle {
            content: title_content,
            numbering,
            tokens: self.synthesize_tokens_for_content(&title_text),
        };

        // Convert children (especially content_container) to session content
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

    /// Convert list item with marker preservation
    fn convert_list_item(&mut self, node: &AstNode) -> ListItem {
        let marker = node
            .attributes
            .get("marker")
            .unwrap_or(&"-".to_string())
            .trim() // Remove trailing spaces from marker
            .to_string();

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

    /// Convert verbatim block that's actually a definition (has title attribute)
    fn convert_verbatim_as_definition(&mut self, node: &AstNode) -> Definition {
        let term_text = node
            .attributes
            .get("title")
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

        // Convert verbatim content as definition content
        let empty_string = String::new();
        let raw_content = node.content.as_ref().unwrap_or(&empty_string);
        let content_blocks = if !raw_content.is_empty() {
            vec![Block::Paragraph(Paragraph {
                content: vec![Inline::TextLine(TextTransform::Identity(Text::simple(
                    raw_content,
                )))],
                annotations: Vec::new(),
                tokens: self.synthesize_tokens_for_content(raw_content),
            })]
        } else {
            Vec::new()
        };

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
