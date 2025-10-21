//! Phase 3a: Document Assembly
//!
//! Wraps the AST tree in a Session container and Document node.
//! This is the first step of Phase 3 assembly, where we take the AST
//! structure and create the final document hierarchy.
//!
//! src/parser/mod.rs has the full architecture overview.

use crate::ast::elements::{
    document::document_structure::{AssemblyInfo, Meta, ProcessingStats},
    session::SessionContainer,
};
use crate::ast::Document;
use crate::ast::ElementNode;

/// Document assembler for creating final document structure
///
/// This assembler takes AST element nodes and creates the proper
/// document hierarchy with Session containers and metadata.
pub struct DocumentAssembler;

impl Default for DocumentAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentAssembler {
    /// Create a new document assembler instance
    pub fn new() -> Self {
        Self
    }

    /// Assemble AST elements into a complete document
    ///
    /// Takes AST element nodes and wraps them in the proper document
    /// hierarchy with Session containers, metadata, and assembly info.
    pub fn assemble_document(
        &self,
        elements: Vec<ElementNode>,
        source_path: Option<String>,
    ) -> Result<Document, DocumentAssemblyError> {
        // Convert ElementNodes to SessionContainerElements
        let mut all_elements = Vec::new();

        for element in elements {
            match element {
                ElementNode::SessionBlock(session) => {
                    all_elements.push(crate::ast::elements::session::session_container::SessionContainerElement::Session(session));
                }
                ElementNode::ParagraphBlock(paragraph) => {
                    all_elements.push(crate::ast::elements::session::session_container::SessionContainerElement::Paragraph(paragraph));
                }
                ElementNode::ListBlock(list) => {
                    all_elements.push(crate::ast::elements::session::session_container::SessionContainerElement::List(list));
                }
                ElementNode::DefinitionBlock(definition) => {
                    all_elements.push(crate::ast::elements::session::session_container::SessionContainerElement::Definition(definition));
                }
                ElementNode::VerbatimBlock(verbatim) => {
                    all_elements.push(crate::ast::elements::session::session_container::SessionContainerElement::Verbatim(verbatim));
                }
                ElementNode::AnnotationBlock(annotation) => {
                    all_elements.push(crate::ast::elements::session::session_container::SessionContainerElement::Annotation(annotation));
                }
                // Handle other element types as needed
                _ => {
                    // For now, skip unsupported element types
                    continue;
                }
            }
        }

        let stats = ProcessingStats {
            token_count: 0,      // TODO: Calculate from elements
            annotation_count: 0, // TODO: Calculate from elements
            block_count: all_elements.len(),
            max_depth: 0, // TODO: Calculate from elements
        };

        let assembly_info = AssemblyInfo {
            parser_version: env!("CARGO_PKG_VERSION").to_string(),
            source_path,
            processed_at: Some(chrono::Utc::now().to_rfc3339()),
            stats,
        };

        let document = Document {
            meta: Meta::default(),
            content: SessionContainer::new(
                all_elements,
                vec![], // Empty annotations for now
                crate::ast::elements::components::parameters::Parameters::default(),
                crate::cst::ScannerTokenSequence::new(),
            ),
            assembly_info,
        };

        Ok(document)
    }
}

/// Errors that can occur during document assembly
#[derive(Debug)]
pub enum DocumentAssemblyError {
    /// Invalid document structure detected
    InvalidStructure(String),
    /// Missing required document components
    MissingComponents(String),
    /// Assembly error at specific position
    AssemblyError {
        position: crate::cst::Position,
        message: String,
    },
}

impl std::fmt::Display for DocumentAssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentAssemblyError::InvalidStructure(msg) => {
                write!(f, "Invalid document structure: {}", msg)
            }
            DocumentAssemblyError::MissingComponents(msg) => {
                write!(f, "Missing required components: {}", msg)
            }
            DocumentAssemblyError::AssemblyError { position, message } => {
                write!(f, "Assembly error at position {:?}: {}", position, message)
            }
        }
    }
}

impl std::error::Error for DocumentAssemblyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_assembler_creation() {
        let _assembler = DocumentAssembler::new();
        // Basic test to ensure assembler can be created
        // The test passes if we reach this point without panicking
    }

    #[test]
    fn test_assemble_document_placeholder() {
        let assembler = DocumentAssembler::new();
        let elements = vec![];

        // This should return a placeholder document until Phase 3a is implemented
        let result = assembler.assemble_document(elements, Some("test.txxt".to_string()));
        assert!(result.is_ok());

        let document = result.unwrap();
        assert_eq!(
            document.assembly_info.parser_version,
            env!("CARGO_PKG_VERSION")
        );
        assert_eq!(
            document.assembly_info.source_path,
            Some("test.txxt".to_string())
        );
    }
}
