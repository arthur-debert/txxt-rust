//! Phase 3: Assembly
//!
//! This module implements the assembly phase that handles document
//! assembly, annotation attachment, and cross-reference resolution.
//!
//! # Three-Phase Pipeline Position
//!
//! **Phase 3: Assembly (Final Output)**
//!
//! The assembly phase takes the hierarchical token structure from Phase 1c (ScannerTokenTree)
//! and produces the final document structure:
//! - Phase 3a: Document Assembly - Wrap AST tree in Session container and Document node
//! - Phase 3b: Annotation Attachment - Apply proximity rules to attach annotations
//!
//! Pipeline: `Tokens` → `Token Tree` → **`Document Assembly`** → **`Final Document`**
//!
//! ## Assembly Process
//!
//! 1. **Document Wrapping**: Wrap token tree in SessionContainer and Document
//! 2. **Annotation Attachment**: Apply proximity rules to attach annotations
//! 3. **Metadata Extraction**: Convert annotations to structured metadata  
//! 4. **Document Finalization**: Add assembly info (parser version, timestamps)
//!
//! # Usage
//!
//! ```rust,ignore
//! use txxt::assembler::Assembler;
//! use txxt::lexer::pipeline::ScannerTokenTreeBuilder;
//! use txxt::lexer::tokenize;
//!
//! // Phase 1: Lexer
//! let tokens = tokenize(input_text);
//! let token_tree_builder = ScannerTokenTreeBuilder::new();
//! let token_tree = token_tree_builder.build_tree(tokens)?;
//!
//! // Phase 3: Assembly
//! let assembler = Assembler::new();
//! let document = assembler.assemble_document(token_tree, Some("source.txxt".to_string()))?;
//! ```

use crate::ast::elements::components::parameters::Parameters;
use crate::ast::{
    base::Document,
    elements::{
        annotation::annotation_content::Annotation,
        document::document_structure::{AssemblyInfo, Meta, MetaValue, ProcessingStats},
        scanner_tokens::{ScannerToken, ScannerTokenSequence},
        session::SessionContainer,
    },
};
use crate::lexer::pipeline::ScannerTokenTree;

// Pipeline modules
pub mod pipeline;

// Re-export main interfaces
pub use pipeline::{
    AnnotationAttacher, AnnotationAttachmentError, DocumentAssembler, DocumentAssemblyError,
};

/// Phase 3 Assembler
///
/// Handles document assembly, annotation processing, and
/// cross-reference resolution to produce the final document structure.
pub struct Assembler;

impl Assembler {
    /// Create a new assembler instance
    pub fn new() -> Self {
        Self
    }

    /// Phase 3a: Wrap token tree in Session container and Document node
    ///
    /// Takes the hierarchical token structure from Phase 1c (ScannerTokenTree) and
    /// wraps it in the proper document structure:
    /// - ScannerTokenTree → SessionContainer (content root)
    /// - SessionContainer → Document (with metadata)
    ///
    /// This creates the basic document structure: `document.content.session[0][content].blocks`
    pub fn assemble_document(
        &self,
        token_tree: ScannerTokenTree,
        source_path: Option<String>,
    ) -> Result<Document, AssemblyError> {
        // Phase 3b: Extract and attach annotations
        let (document_annotations, content_annotations) = self.extract_annotations(&token_tree)?;

        // Create assembly info with processing stats
        let stats = ProcessingStats {
            token_count: count_tokens_in_token_tree(&token_tree),
            annotation_count: document_annotations.len() + content_annotations.len(),
            block_count: count_blocks_in_token_tree(&token_tree),
            max_depth: calculate_max_depth(&token_tree),
        };

        let assembly_info = AssemblyInfo {
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            source_path,
            processed_at: Some(chrono::Utc::now().to_rfc3339()),
            stats,
        };

        // Convert document-level annotations to metadata
        let meta = self.extract_metadata_from_annotations(&document_annotations)?;

        // Convert ScannerTokenTree to SessionContainer
        // For now, we'll create a simple session container that holds the raw token tree
        // Later when Phase 2 is implemented, this will contain proper AST nodes
        let session_container = SessionContainer::new(
            vec![], // TODO: Parse sessions from token_tree when Phase 2 is implemented
            vec![], // TODO: Parse other blocks when Phase 2 is implemented
            Parameters::default(),
            ScannerTokenSequence::new(), // TODO: Extract tokens from token_tree
        );

        // Create document with metadata from annotations
        let document = Document {
            meta,
            content: session_container,
            assembly_info,
        };

        Ok(document)
    }

    /// Process parsed AST into final document (for future Phase 2 integration)
    ///
    /// This method will be used when Phase 2 parsing is implemented to process
    /// fully parsed AST nodes instead of raw token trees.
    pub fn process_ast(&self, _ast: Document) -> Result<Document, AssemblyError> {
        // TODO: Implement full assembly logic for when Phase 2 is complete
        // - Document metadata assembly from AST annotations
        // - Annotation proximity-based attachment to AST nodes
        // - Cross-reference resolution
        // - Final validation
        Err(AssemblyError::NotImplemented(
            "Full AST assembly not yet implemented - Phase 2 parsing required".to_string(),
        ))
    }

    /// Phase 3b: Extract annotations from block group and apply proximity rules
    ///
    /// Returns (document_annotations, content_annotations) where:
    /// - document_annotations: Annotations at document start (attach to document)
    /// - content_annotations: Other annotations (attach to elements or parents)
    fn extract_annotations(
        &self,
        token_tree: &ScannerTokenTree,
    ) -> Result<(Vec<Annotation>, Vec<Annotation>), AssemblyError> {
        let mut document_annotations = Vec::new();
        let mut content_annotations = Vec::new();

        // Extract annotations from root level first (document-level)
        let root_annotations = self.extract_annotations_from_tokens(&token_tree.tokens)?;
        document_annotations.extend(root_annotations);

        // Extract annotations from child trees (content-level)
        for child in &token_tree.children {
            let child_annotations = self.extract_annotations_recursive(child)?;
            content_annotations.extend(child_annotations);
        }

        Ok((document_annotations, content_annotations))
    }

    /// Recursively extract annotations from token trees
    fn extract_annotations_recursive(
        &self,
        token_tree: &ScannerTokenTree,
    ) -> Result<Vec<Annotation>, AssemblyError> {
        let mut annotations = Vec::new();

        // Extract from this level
        let level_annotations = self.extract_annotations_from_tokens(&token_tree.tokens)?;
        annotations.extend(level_annotations);

        // Extract from children
        for child in &token_tree.children {
            let child_annotations = self.extract_annotations_recursive(child)?;
            annotations.extend(child_annotations);
        }

        Ok(annotations)
    }

    /// Extract annotations from a sequence of tokens
    fn extract_annotations_from_tokens(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<Vec<Annotation>, AssemblyError> {
        let mut annotations = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            if let ScannerToken::TxxtMarker { .. } = &tokens[i] {
                // Found annotation start, extract the complete annotation
                if let Some((annotation, next_idx)) =
                    self.parse_annotation_from_tokens(tokens, i)?
                {
                    annotations.push(annotation);
                    i = next_idx;
                } else {
                    i += 1; // Skip malformed annotation
                }
            } else {
                i += 1;
            }
        }

        Ok(annotations)
    }

    /// Parse a complete annotation from tokens starting at the given index
    fn parse_annotation_from_tokens(
        &self,
        tokens: &[ScannerToken],
        start_idx: usize,
    ) -> Result<Option<(Annotation, usize)>, AssemblyError> {
        // Look for opening :: marker
        if !matches!(&tokens[start_idx], ScannerToken::TxxtMarker { .. }) {
            return Ok(None);
        }

        // Find closing :: marker
        let mut end_idx = start_idx + 1;
        while end_idx < tokens.len() {
            if matches!(&tokens[end_idx], ScannerToken::TxxtMarker { .. }) {
                break;
            }
            end_idx += 1;
        }

        if end_idx >= tokens.len() {
            // No closing marker found
            return Ok(None);
        }

        // Extract content between markers
        let content_tokens = &tokens[start_idx + 1..end_idx];
        let (label, parameters) = self.parse_annotation_content(content_tokens)?;

        // Create annotation
        let annotation = Annotation {
            label,
            parameters,
            content: crate::ast::elements::annotation::annotation_content::AnnotationContent::Empty, // TODO: Parse content
            tokens: ScannerTokenSequence::new(), // TODO: Create proper ScannerTokenSequence from content_tokens
            namespace: None,                     // TODO: Parse namespace
        };

        Ok(Some((annotation, end_idx + 1)))
    }

    /// Parse annotation content to extract label and parameters
    fn parse_annotation_content(
        &self,
        tokens: &[ScannerToken],
    ) -> Result<(String, Parameters), AssemblyError> {
        let mut label = String::new();
        let parameters = Parameters::default();

        for token in tokens {
            match token {
                ScannerToken::Text { content, .. } => {
                    if label.is_empty() {
                        label = content.clone();
                    }
                }
                ScannerToken::Colon { .. } => {
                    // Colon separates label from parameters
                    // TODO: Parse parameters after colon
                }
                _ => {
                    // TODO: Handle other tokens (parameters, etc.)
                }
            }
        }

        Ok((label, parameters))
    }

    /// Extract metadata from document-level annotations
    fn extract_metadata_from_annotations(
        &self,
        annotations: &[Annotation],
    ) -> Result<Meta, AssemblyError> {
        let mut meta = Meta::default();

        for annotation in annotations {
            match annotation.label.as_str() {
                "title" => {
                    meta.title = Some(MetaValue::String(self.extract_annotation_text(annotation)?));
                }
                "author" => {
                    let author_text = self.extract_annotation_text(annotation)?;
                    meta.authors.push(MetaValue::String(author_text));
                }
                "date" | "pub-date" => {
                    meta.date = Some(MetaValue::String(self.extract_annotation_text(annotation)?));
                }
                _ => {
                    // Custom metadata
                    let value = MetaValue::String(self.extract_annotation_text(annotation)?);
                    meta.custom.insert(annotation.label.clone(), value);
                }
            }
        }

        Ok(meta)
    }

    /// Extract text content from an annotation
    fn extract_annotation_text(&self, annotation: &Annotation) -> Result<String, AssemblyError> {
        // TODO: Extract text from annotation tokens
        // For now, return placeholder
        Ok(format!("[{}]", annotation.label))
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Assembly error types
#[derive(Debug, Clone)]
pub enum AssemblyError {
    /// Feature not yet implemented
    NotImplemented(String),
    /// Cross-reference resolution failed
    UnresolvedReference(String),
    /// Annotation attachment failed
    AnnotationAttachmentFailed(String),
    /// Document assembly failed
    DocumentAssemblyFailed(String),
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblyError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            AssemblyError::UnresolvedReference(msg) => {
                write!(f, "Unresolved reference: {}", msg)
            }
            AssemblyError::AnnotationAttachmentFailed(msg) => {
                write!(f, "Annotation attachment failed: {}", msg)
            }
            AssemblyError::DocumentAssemblyFailed(msg) => {
                write!(f, "Document assembly failed: {}", msg)
            }
        }
    }
}

impl std::error::Error for AssemblyError {}

/// Helper function to count total tokens in a token tree
fn count_tokens_in_token_tree(token_tree: &ScannerTokenTree) -> usize {
    let mut count = token_tree.tokens.len();
    for child in &token_tree.children {
        count += count_tokens_in_token_tree(child);
    }
    count
}

/// Helper function to count blocks (non-empty trees) in a token tree
fn count_blocks_in_token_tree(token_tree: &ScannerTokenTree) -> usize {
    let mut count = if token_tree.tokens.is_empty() { 0 } else { 1 };
    for child in &token_tree.children {
        count += count_blocks_in_token_tree(child);
    }
    count
}

/// Helper function to calculate maximum nesting depth
fn calculate_max_depth(token_tree: &ScannerTokenTree) -> usize {
    let mut max_child_depth = 0;
    for child in &token_tree.children {
        max_child_depth = max_child_depth.max(calculate_max_depth(child));
    }
    1 + max_child_depth
}
