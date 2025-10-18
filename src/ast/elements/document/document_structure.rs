//! Core document and metadata structures
//!
//! This module defines the top-level document structure and metadata handling
//! that forms the foundation of the TXXT AST.
//!
//! src/parser/mod.rs has the full architecture overview.
//!
//! ## Assembly Process
//!
//! 1. **Annotation Attachment**: Apply proximity rules to attach annotations
//! 2. **Metadata Extraction**: Convert annotations to structured metadata  
//! 3. **Document Finalization**: Add assembly info (parser version, timestamps)
//! 4. **Statistics Computation**: Calculate processing stats for tooling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ast::elements::{
    blocks::Block, components::parameters::Parameters, scanner_tokens::ScannerTokenSequence,
    session::SessionContainer,
};

/// Top-level document structure
///
/// Represents a complete TXXT document after parsing and assembly phases.
/// The document root is a SessionContainer that can hold any blocks including
/// sessions, providing the hierarchical document structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    /// Document metadata (frontmatter-style information)
    pub meta: Meta,

    /// Main document content in a SessionContainer
    /// Root container can hold sessions and any other blocks
    pub content: SessionContainer,

    /// Assembly metadata added during document processing
    pub assembly_info: AssemblyInfo,
}

/// Document metadata extracted from annotations and other sources
///
/// Follows Pandoc-style metadata structure for interoperability.
/// Can be populated from document annotations or explicit frontmatter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Meta {
    /// Document title (often from :: title :: annotation)
    pub title: Option<MetaValue>,

    /// Document authors (from :: author :: annotations)
    pub authors: Vec<MetaValue>,

    /// Publication date (from :: date :: or :: pub-date :: annotations)
    pub date: Option<MetaValue>,

    /// Custom metadata from arbitrary annotations
    /// Key is the annotation label, value is the annotation content
    pub custom: HashMap<String, MetaValue>,
}

/// Metadata values that can be various types
///
/// Supports rich metadata that can contain both simple strings and
/// complex structured content with formatting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaValue {
    /// Simple string value
    String(String),

    /// Rich text with inline formatting
    Inlines(Vec<crate::ast::elements::formatting::inlines::Inline>),

    /// Structured content (for complex metadata)
    Blocks(Vec<Block>),

    /// List of values (for multiple authors, etc.)
    List(Vec<MetaValue>),
}

/// Information added during document assembly phase
///
/// This metadata is not part of the source content but is added during
/// processing to support tooling and debugging needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssemblyInfo {
    /// Parser version that created this AST
    pub parser_version: String,

    /// Source file path (if available)
    pub source_path: Option<String>,

    /// Processing timestamp
    pub processed_at: Option<String>,

    /// Parsing/assembly statistics
    pub stats: ProcessingStats,
}

/// Statistics about the parsing and assembly process
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProcessingStats {
    /// Total number of tokens processed
    pub token_count: usize,

    /// Number of annotations processed
    pub annotation_count: usize,

    /// Number of blocks in final document
    pub block_count: usize,

    /// Maximum nesting depth encountered
    pub max_depth: usize,
}

impl Document {
    /// Create a new document with minimal information (for compatibility)
    pub fn new(source: String) -> Self {
        Self {
            meta: Meta::default(),
            content: SessionContainer::new(
                Vec::new(),
                Vec::new(),
                Parameters::default(),
                ScannerTokenSequence::new(),
            ),
            assembly_info: AssemblyInfo {
                source_path: Some(source),
                ..AssemblyInfo::default()
            },
        }
    }
}

impl Default for AssemblyInfo {
    fn default() -> Self {
        Self {
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            source_path: None,
            processed_at: None,
            stats: ProcessingStats::default(),
        }
    }
}
