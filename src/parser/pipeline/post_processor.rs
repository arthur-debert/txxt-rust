//! Phase 3: Post-Processing
//!
//! This module implements the post-processing phase that handles document
//! assembly, annotation attachment, and cross-reference resolution.

use crate::ast::elements::components::parameters::Parameters;
use crate::ast::{
    base::Document,
    elements::{
        annotation::annotation_content::Annotation,
        document::document_structure::{AssemblyInfo, Meta, MetaValue, ProcessingStats},
        session::SessionContainer,
        tokens::{Token, TokenSequence},
    },
};
use crate::parser::pipeline::block_grouper::BlockGroup;

/// Phase 3 Post-Processor
///
/// Handles final document assembly, annotation processing, and
/// cross-reference resolution after the main parsing phase.
pub struct PostProcessor;

impl PostProcessor {
    /// Create a new post-processor instance
    pub fn new() -> Self {
        Self
    }

    /// Phase 3a: Wrap token tree in Session container and Document node
    ///
    /// Takes the hierarchical token structure from Phase 1c (BlockGroup) and
    /// wraps it in the proper document structure:
    /// - BlockGroup → SessionContainer (content root)
    /// - SessionContainer → Document (with metadata)
    ///
    /// This creates the basic document structure: `document.content.session[0][content].blocks`
    pub fn assemble_document(
        &self,
        block_group: BlockGroup,
        source_path: Option<String>,
    ) -> Result<Document, PostProcessError> {
        // Phase 3b: Extract and attach annotations
        let (document_annotations, content_annotations) = self.extract_annotations(&block_group)?;

        // Create assembly info with processing stats
        let stats = ProcessingStats {
            token_count: count_tokens_in_block_group(&block_group),
            annotation_count: document_annotations.len() + content_annotations.len(),
            block_count: count_blocks_in_block_group(&block_group),
            max_depth: calculate_max_depth(&block_group),
        };

        let assembly_info = AssemblyInfo {
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            source_path,
            processed_at: Some(chrono::Utc::now().to_rfc3339()),
            stats,
        };

        // Convert document-level annotations to metadata
        let meta = self.extract_metadata_from_annotations(&document_annotations)?;

        // Convert BlockGroup to SessionContainer
        // For now, we'll create a simple session container that holds the raw block group
        // Later when Phase 2 is implemented, this will contain proper AST nodes
        let session_container = SessionContainer::new(
            vec![], // TODO: Parse sessions from block_group when Phase 2 is implemented
            vec![], // TODO: Parse other blocks when Phase 2 is implemented
            Parameters::default(),
            TokenSequence::new(), // TODO: Extract tokens from block_group
        );

        // Create document with metadata from annotations
        let document = Document {
            meta,
            content: session_container,
            assembly_info,
        };

        Ok(document)
    }

    /// Post-process parsed AST into final document
    ///
    /// Performs document assembly, annotation attachment using proximity rules,
    /// and resolves cross-references between elements.
    pub fn process(&self, _ast: Document) -> Result<Document, PostProcessError> {
        // TODO: Implement full post-processing logic for when Phase 2 is complete
        // - Document metadata assembly
        // - Annotation proximity-based attachment
        // - Cross-reference resolution
        // - Final validation
        Err(PostProcessError::NotImplemented(
            "Full post-processor not yet implemented - Phase 2 parsing required".to_string(),
        ))
    }
}

impl Default for PostProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Post-processing error types
#[derive(Debug, Clone)]
pub enum PostProcessError {
    /// Feature not yet implemented
    NotImplemented(String),
    /// Cross-reference resolution failed
    UnresolvedReference(String),
    /// Annotation attachment failed
    AnnotationError(String),
    /// Document assembly failed
    AssemblyError(String),
}

impl std::fmt::Display for PostProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostProcessError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            PostProcessError::UnresolvedReference(msg) => {
                write!(f, "Unresolved reference: {}", msg)
            }
            PostProcessError::AnnotationError(msg) => write!(f, "Annotation error: {}", msg),
            PostProcessError::AssemblyError(msg) => write!(f, "Assembly error: {}", msg),
        }
    }
}

impl std::error::Error for PostProcessError {}

/// Helper function to count total tokens in a block group
fn count_tokens_in_block_group(block_group: &BlockGroup) -> usize {
    let mut count = block_group.tokens.len();
    for child in &block_group.children {
        count += count_tokens_in_block_group(child);
    }
    count
}

/// Helper function to count blocks (non-empty groups) in a block group
fn count_blocks_in_block_group(block_group: &BlockGroup) -> usize {
    let mut count = if block_group.tokens.is_empty() { 0 } else { 1 };
    for child in &block_group.children {
        count += count_blocks_in_block_group(child);
    }
    count
}

/// Helper function to calculate maximum nesting depth
fn calculate_max_depth(block_group: &BlockGroup) -> usize {
    let mut max_child_depth = 0;
    for child in &block_group.children {
        max_child_depth = max_child_depth.max(calculate_max_depth(child));
    }
    1 + max_child_depth
}

impl PostProcessor {
    /// Phase 3b: Extract annotations from block group and apply proximity rules
    ///
    /// Returns (document_annotations, content_annotations) where:
    /// - document_annotations: Annotations at document start (attach to document)
    /// - content_annotations: Other annotations (attach to elements or parents)
    fn extract_annotations(
        &self,
        block_group: &BlockGroup,
    ) -> Result<(Vec<Annotation>, Vec<Annotation>), PostProcessError> {
        let mut document_annotations = Vec::new();
        let mut content_annotations = Vec::new();

        // Extract annotations from root level first (document-level)
        let root_annotations = self.extract_annotations_from_tokens(&block_group.tokens)?;
        document_annotations.extend(root_annotations);

        // Extract annotations from child groups (content-level)
        for child in &block_group.children {
            let child_annotations = self.extract_annotations_recursive(child)?;
            content_annotations.extend(child_annotations);
        }

        Ok((document_annotations, content_annotations))
    }

    /// Recursively extract annotations from block groups
    fn extract_annotations_recursive(
        &self,
        block_group: &BlockGroup,
    ) -> Result<Vec<Annotation>, PostProcessError> {
        let mut annotations = Vec::new();

        // Extract from this level
        let level_annotations = self.extract_annotations_from_tokens(&block_group.tokens)?;
        annotations.extend(level_annotations);

        // Extract from children
        for child in &block_group.children {
            let child_annotations = self.extract_annotations_recursive(child)?;
            annotations.extend(child_annotations);
        }

        Ok(annotations)
    }

    /// Extract annotations from a sequence of tokens
    fn extract_annotations_from_tokens(
        &self,
        tokens: &[Token],
    ) -> Result<Vec<Annotation>, PostProcessError> {
        let mut annotations = Vec::new();
        let mut i = 0;

        while i < tokens.len() {
            if let Token::AnnotationMarker { .. } = &tokens[i] {
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
        tokens: &[Token],
        start_idx: usize,
    ) -> Result<Option<(Annotation, usize)>, PostProcessError> {
        // Look for opening :: marker
        if !matches!(&tokens[start_idx], Token::AnnotationMarker { .. }) {
            return Ok(None);
        }

        // Find closing :: marker
        let mut end_idx = start_idx + 1;
        while end_idx < tokens.len() {
            if matches!(&tokens[end_idx], Token::AnnotationMarker { .. }) {
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
            tokens: TokenSequence::new(), // TODO: Create proper TokenSequence from content_tokens
            namespace: None,              // TODO: Parse namespace
        };

        Ok(Some((annotation, end_idx + 1)))
    }

    /// Parse annotation content to extract label and parameters
    fn parse_annotation_content(
        &self,
        tokens: &[Token],
    ) -> Result<(String, Parameters), PostProcessError> {
        let mut label = String::new();
        let parameters = Parameters::default();

        for token in tokens {
            match token {
                Token::Text { content, .. } => {
                    if label.is_empty() {
                        label = content.clone();
                    }
                }
                Token::Colon { .. } => {
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
    ) -> Result<Meta, PostProcessError> {
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
    fn extract_annotation_text(&self, annotation: &Annotation) -> Result<String, PostProcessError> {
        // TODO: Extract text from annotation tokens
        // For now, return placeholder
        Ok(format!("[{}]", annotation.label))
    }
}
